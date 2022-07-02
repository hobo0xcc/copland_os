use crate::arch::PAGE_SIZE;
use crate::error::VMError;
use crate::lazy::Lazy;
use alloc::alloc::alloc_zeroed;
use alloc::string::{String, ToString};
use alloc::vec;
use bitflags::bitflags;
use core::alloc::Layout;
use core::arch::asm;
use core::mem::size_of;
use hashbrown::HashMap;
use log::info;

pub const LEVELS: usize = 3;

// https://developer.arm.com/documentation/ddi0595/2021-12/AArch64-Registers/MAIR-EL1--Memory-Attribute-Indirection-Register--EL1-
// mair_el1.attr0 = 0b0100_0100  means Normal memory, Inner/Outer Non-cacheable
// mair_el1.attr1 = 0b1111_1111  means Normal memory, Inner/Outer WB/WA/RA
// mair_el1.attr2 = 0b0000_0000  means Device-nGnRnE memory
pub const MAIR_EL1: usize = 0x000000000000FF44;

bitflags! {
    pub struct PTE: usize {
        const ATTRINDX = 0b111 << 2;
        const NS = 0b1 << 5;
        const AP_2_1 = 0b11 << 6;
        const SH_1_0 = 0b11 << 8;
        const AF = 0b1 << 10;
        const NG = 0b1 << 11;
        const NT = 0b1 << 16;
        const UXN = 0b1 << 54;
        const PXN = 0b1 << 53;

        // AttrIndx
        const NORMAL_NON_CACHEABLE = 0b000 << 2;
        const NORMAL_CACHEABLE = 0b001 << 2;
        const DEVICE = 0b010 << 2;

        // AP
        const RO_ALL = 0b11 << 6;
        const RW_ALL = 0b01 << 6;
        const RO_EL1 = 0b10 << 6;
        const RW_EL1 = 0b00 << 6;

        // Descriptor
        const PAGE = 0b11;
        const TABLE = 0b11;
        const BLOCK = 0b01;

        // Output address
        const OA = 0x7ff_ffff << 12;
    }
}

pub static mut VM_MANAGER: Lazy<VMManager> = Lazy::new(|| VMManager::new());

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Entry(pub usize);

impl Entry {
    pub fn is_valid(&self) -> bool {
        self.0 & 0b11 != 0
    }

    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    pub fn is_block(&self) -> bool {
        self.0 & 0b11 == PTE::BLOCK.bits()
    }

    pub fn as_table(&mut self) {
        self.0 = (self.0 & !(0b11_usize)) | PTE::TABLE.bits()
    }

    pub fn as_page(&mut self) {
        self.0 = (self.0 & !(0b11_usize)) | PTE::PAGE.bits();
    }

    pub fn as_block(&mut self) {
        self.0 = (self.0 & !(0b11_usize)) | PTE::BLOCK.bits();
    }

    pub fn set_oa(&mut self, oa: usize) {
        self.0 = (self.0 & !(PTE::OA.bits())) | oa
    }

    pub fn get_oa(&self) -> usize {
        self.0 & PTE::OA.bits()
    }

    pub fn set_flags(&mut self, r: bool, w: bool, x: bool, u: bool) {
        assert!(r || w);
        assert!(!(!r && w)); // invalid read write combination
                             // ARM Ref: D5.3.3
        match (r, w, u) {
            (true, true, true) => self.0 |= PTE::RW_ALL.bits(),
            (true, true, false) => self.0 |= PTE::RW_EL1.bits(),
            (true, false, true) => self.0 |= PTE::RO_ALL.bits(),
            (true, false, false) => self.0 |= PTE::RO_EL1.bits(),
            _ => unreachable!("r: {} w: {} x: {} u: {}", r, w, x, u),
        }
        match (x, u) {
            (true, true) => self.0 = self.0 & !PTE::UXN.bits(),
            (true, false) => self.0 = self.0 & !PTE::PXN.bits(),
            (false, true) => self.0 |= PTE::UXN.bits(),
            (false, false) => self.0 |= PTE::PXN.bits(),
        }
    }

    pub fn set_af(&mut self) {
        self.0 |= PTE::AF.bits()
    }

    pub fn set_attr(&mut self, attr: usize) {
        self.0 = (self.0 & !PTE::ATTRINDX.bits()) | attr;
    }
}

pub struct PageTable {
    entries: [Entry; 512],
}

impl PageTable {
    pub fn address(&self) -> usize {
        (self as *const PageTable) as usize
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn update_entry(&mut self, index: usize, entry: Entry) {
        assert!(index < self.size());
        self.entries[index] = entry
    }

    pub fn get_entry(&self, index: usize) -> Entry {
        assert!(index < self.size());
        self.entries[index]
    }

    pub fn identity_mapping(&mut self, level: usize, old: usize) {
        for i in 0..self.size() {
            let paddr = (i << (12 + 9 * level)) | old;
            let mut entry = Entry::default();
            entry.as_block();
            entry.set_flags(true, true, true, false);
            entry.set_attr(PTE::NORMAL_CACHEABLE.bits());
            entry.set_af();
            entry.set_oa(paddr);
            self.update_entry(i, entry);
        }
    }
}

pub struct VMManager {
    root_tables: HashMap<String, *mut PageTable>,
}

unsafe impl Sync for VMManager {}
unsafe impl Send for VMManager {}

impl VMManager {
    pub fn new() -> Self {
        Self {
            root_tables: HashMap::new(),
        }
    }

    pub fn create_table(&self) -> *mut PageTable {
        assert_eq!(size_of::<PageTable>(), 4096);

        let table = unsafe {
            let layout =
                Layout::from_size_align(size_of::<PageTable>(), size_of::<PageTable>()).unwrap();
            let table_ptr = alloc_zeroed(layout) as *mut PageTable;
            // aligned?
            assert_eq!(table_ptr as usize & 0xfff, 0);
            table_ptr
        };
        table
    }

    pub fn get_attr(&self, paddr: usize) -> usize {
        let mut attr = PTE::NORMAL_CACHEABLE.bits();
        if cfg!(target_board = "raspi3b") {
            use crate::device::raspi3b::base::*;
            if MMIO_BASE <= paddr && paddr < (MMIO_BASE + MMIO_SIZE) {
                attr = PTE::DEVICE.bits();
            }
        }
        attr
    }

    pub fn map_device_memory(&self, table: *mut PageTable) -> Result<(), VMError> {
        if cfg!(target_board = "raspi3b") {
            use crate::device::raspi3b::base::*;
            for page in (MMIO_BASE..(MMIO_BASE + MMIO_SIZE)).step_by(PAGE_SIZE) {
                assert!(page % PAGE_SIZE == 0);
                self.map_page(
                    table,
                    page,
                    page,
                    true,
                    true,
                    false,
                    false,
                    PTE::DEVICE.bits(),
                )?;
            }
        }
        Ok(())
    }

    pub fn walk(&self, name: &str, vaddr: usize) -> Result<usize, VMError> {
        use crate::*;
        let mut table = unsafe { self.get_table(name).as_mut().unwrap() };
        let vaddr_page = vaddr & !0xfff_usize;
        println!("vaddr_page: {:#x}", vaddr_page);
        let indexes = vec![
            (vaddr_page >> 12) & 0x1ff,
            (vaddr_page >> 21) & 0x1ff,
            (vaddr_page >> 30) & 0x1ff,
        ];
        for level in (1..LEVELS).rev() {
            let entry = table.get_entry(indexes[level]);
            if entry.is_invalid() {
                return Err(VMError::NotFound);
            }
            if entry.is_block() {
                let mask = (1 << (12 + 9 * level)) - 1;
                return Ok((entry.get_oa() & !mask) + (vaddr & mask));
            }
            // next page table
            table = unsafe {
                ((entry.get_oa() & PTE::OA.bits()) as *mut PageTable)
                    .as_mut()
                    .unwrap()
            };
        }
        Ok((table.get_entry(indexes[0]).get_oa() & PTE::OA.bits()) + (vaddr & 0xfff))
    }

    pub fn map_page(
        &self,
        table: *mut PageTable,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
        u: bool,
        attr: usize,
    ) -> Result<(), VMError> {
        let mut table = unsafe { table.as_mut().unwrap() };
        let indexes = vec![
            (vaddr >> 12) & 0x1ff,
            (vaddr >> 21) & 0x1ff,
            (vaddr >> 30) & 0x1ff,
        ];
        unsafe {
            for level in (1..LEVELS).rev() {
                let entry = table.get_entry(indexes[level]);
                if entry.is_block() || entry.is_invalid() {
                    let new_table = self.create_table();
                    let old = if entry.is_invalid() {
                        0
                    } else {
                        entry.get_oa()
                    };
                    let child = level - 1;
                    new_table.as_mut().unwrap().identity_mapping(child, old);
                    let mut new_entry = Entry::default();
                    new_entry.as_table();
                    new_entry.set_oa(new_table as usize);
                    table.update_entry(indexes[level], new_entry);
                    table = new_table.as_mut().unwrap();
                } else {
                    let new_table = (table.get_entry(indexes[level]).get_oa()) as *mut PageTable;
                    table = new_table.as_mut().unwrap();
                }
            }
            let mut new_entry = Entry::default();
            new_entry.as_page();
            new_entry.set_flags(r, w, x, u);
            new_entry.set_oa(paddr);
            new_entry.set_attr(attr);
            new_entry.set_af();
            table.update_entry(indexes[0], new_entry);
        }
        Ok(())
    }

    pub fn map(
        &mut self,
        name: &str,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
        u: bool,
    ) -> Result<(), VMError> {
        assert!(paddr & 0xfff == 0);
        assert!(vaddr & 0xfff == 0);
        let table = self.get_table(name);
        self.map_page(table, paddr, vaddr, r, w, x, u, self.get_attr(paddr))?;
        Ok(())
    }

    pub fn get_table(&self, name: &str) -> *mut PageTable {
        *self.root_tables.get(name).unwrap()
    }

    pub fn set_table(&mut self, name: &str, table: *mut PageTable) {
        self.root_tables.insert(name.to_string(), table);
    }

    pub fn init(&mut self) {
        info!("Initialize VM Manager");
        let root_table = self.create_table();
        self.set_table("kernel", root_table);
        assert!(self.root_tables.contains_key("kernel"));
        unsafe {
            root_table.as_mut().unwrap().identity_mapping(3, 0);
        }
        self.map_device_memory(root_table).unwrap();

        // Is root_table aligned to 2^12?
        assert_eq!(root_table as usize & 0xfff, 0);

        let mut tcr_el1: usize = 0;
        // T0SZ = 25 (The region size is 2^39)
        tcr_el1 |= 0x19;
        // T1SZ = 25
        tcr_el1 |= 0x19 << 16;
        // IRGN0: Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable
        tcr_el1 |= 0b1 << 8;
        // ORGN0: Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable
        tcr_el1 |= 0b1 << 10;
        // SH0: Inner Shareable
        tcr_el1 |= 0b11 << 12;
        // EPD1: A TLB miss on an address that is translated using TTBR1_EL1 generates a Translation fault
        tcr_el1 |= 0b1 << 23;

        let mair_el1: usize = MAIR_EL1;
        let mut sctlr_el1: usize = 0;
        // M=1  Enable the stage 1 MMU
        sctlr_el1 |= 0b1 << 0;
        // C=1  Enable data and unified caches
        sctlr_el1 |= 0b1 << 2;
        // I=1  Enable instruction fetches to allocate into unified caches
        sctlr_el1 |= 0b1 << 12;

        unsafe {
            asm!("msr tcr_el1, {}", in(reg)tcr_el1);
            asm!("msr mair_el1, {}", in(reg)mair_el1);

            asm!("dsb sy");
            asm!("isb");
            // Invalidate TLB
            asm!("tlbi vmalle1");

            asm!("msr ttbr0_el1, {}", in(reg)root_table);

            // Enable MMU
            asm!("msr sctlr_el1, {}", in(reg)sctlr_el1);
            asm!("isb");
        }
    }
}
