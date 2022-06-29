use crate::lazy::Lazy;
use alloc::alloc::alloc_zeroed;
use alloc::string::{String, ToString};
use bitflags::bitflags;
use core::alloc::Layout;
use core::arch::asm;
use core::mem::size_of;
use hashbrown::HashMap;
use log::info;

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
        const XN = 0b1 << 54;

        // AttrIndx
        const NORMAL_NON_CACHEABLE = 0b00 << 2;
        const NORMAL_CACHEABLE = 0b01 << 2;
        const DEVICE = 0b10 << 2;

        // AP
        const RO_ALL = 0b11 << 6;
        const RW_ALL = 0b01 << 6;
        const RO_EL1 = 0b10 << 6;
        const RW_EL1 = 0b00 << 6;

        // Descriptor
        const PAGE = 0b11;
        const TABLE = 0b11;
        const BLOCK = 0b01;
    }
}

pub static mut VM_MANAGER: Lazy<VMManager> = Lazy::new(|| VMManager::new());

#[repr(packed)]
pub struct Entry(usize);

pub struct PageTable {
    entries: [Entry; 512],
}

impl PageTable {
    pub fn address(&self) -> usize {
        (self as *const PageTable) as usize
    }

    pub fn entry_length(&self) -> usize {
        self.entries.len()
    }

    pub fn update_entry(&mut self, index: usize, entry: Entry) {
        self.entries[index] = entry
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

    pub fn create_table(&mut self) -> *mut PageTable {
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

    pub fn identity_mapping(&mut self, table: *mut PageTable) {
        unsafe {
            for i in 0..(*table).entry_length() {
                // (*table).update_entry(i, Entry(0x00000000000000405 | (i << 30)));
                (*table).update_entry(
                    i,
                    Entry(
                        PTE::BLOCK.bits()
                            | PTE::NORMAL_CACHEABLE.bits()
                            | PTE::AF.bits()
                            | (i << 30),
                    ),
                );
            }
        }
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
        self.identity_mapping(root_table);

        // Is root_table aligned to 2^12?
        assert_eq!(root_table as usize & 0xfff, 0);

        let mut tcr_el1: usize = 0;
        // T0SZ = 25 (The region size is 2^39)
        tcr_el1 |= 0x19;
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

            // Invalidate TLBs
            asm!("dsb sy");
            asm!("isb");

            asm!("msr ttbr0_el1, {}", in(reg)root_table);

            // Enable MMU
            asm!("msr sctlr_el1, {}", in(reg)sctlr_el1);
            asm!("isb");
        }
    }
}
