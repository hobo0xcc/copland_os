use crate::sync::lazy::Lazy;
use alloc::alloc::alloc_zeroed;
use alloc::string::{String, ToString};
use core::alloc::Layout;
use core::arch::asm;
use core::mem::size_of;
use hashbrown::HashMap;
use log::info;

pub static mut VM_MANAGER: Lazy<VMManager<'static>> = Lazy::new(|| VMManager::new());

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

pub struct VMManager<'a> {
    root_tables: HashMap<String, &'a mut PageTable>,
}

unsafe impl Sync for VMManager<'_> {}
unsafe impl Send for VMManager<'_> {}

impl VMManager<'_> {
    pub fn new() -> Self {
        Self {
            root_tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: &str) {
        assert!(!self.root_tables.contains_key(name));
        assert_eq!(size_of::<PageTable>(), 4096);

        let table = unsafe {
            let layout =
                Layout::from_size_align(size_of::<PageTable>(), size_of::<PageTable>()).unwrap();
            let table_ptr = alloc_zeroed(layout) as *mut PageTable;

            // aligned?
            assert_eq!(table_ptr as usize & 0xfff, 0);

            table_ptr.as_mut().unwrap()
        };

        self.root_tables.insert(name.to_string(), table);
    }

    pub fn identity_mapping(&mut self, name: &str) {
        let root_table = self.get_root_table_mut(name);
        for i in 0..root_table.entry_length() {
            root_table.update_entry(i, Entry(0x00000000000000405 | (i << 30)));
        }
    }

    pub fn get_root_table(&self, name: &str) -> &PageTable {
        assert!(self.root_tables.contains_key(name));
        self.root_tables.get(name).unwrap()
    }

    pub fn get_root_table_mut(&mut self, name: &str) -> &mut PageTable {
        assert!(self.root_tables.contains_key(name));
        self.root_tables.get_mut(name).unwrap()
    }

    pub fn init(&mut self) {
        info!("Initialize VM Manager");
        self.create_table("kernel");
        assert!(self.root_tables.contains_key("kernel"));
        self.identity_mapping("kernel");
        let root_table = self.get_root_table("kernel") as *const PageTable as usize;

        // Is root_table aligned to 2^12?
        assert_eq!(root_table & 0xfff, 0);

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
        let mair_el1: usize = 0x000000000000FF44;
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
