use crate::arch::riscv64::csr::*;
use alloc::alloc::*;
use core::arch::asm;
use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref VM_MANAGER: Mutex<VMManager> = Mutex::new(VMManager::new());
}

#[repr(C)]
pub struct EntrySv39(usize);

impl EntrySv39 {
    pub fn new() -> Self {
        EntrySv39(0_usize)
    }

    pub fn from_entry(entry: Entry) -> Self {
        assert_eq!(entry.ppn & 0x3ff, 0_usize);
        EntrySv39(
            (entry.v as usize) << 0
                | (entry.r as usize) << 1
                | (entry.w as usize) << 2
                | (entry.x as usize) << 3
                | (entry.u as usize) << 4
                | entry.ppn,
        )
    }
}

pub struct Entry {
    v: bool,
    r: bool,
    w: bool,
    x: bool,
    u: bool,
    ppn: usize,
}

#[repr(C)]
pub struct PageTableSv39 {
    entries: [EntrySv39; 512],
}

pub enum PageTable {
    Sv39(*mut PageTableSv39),
}

impl PageTable {
    pub fn address(&self) -> usize {
        match *self {
            PageTable::Sv39(ref ptr) => (*ptr) as usize,
        }
    }

    pub fn entry_length(&self) -> usize {
        match *self {
            PageTable::Sv39(ref ptes) => unsafe { (**ptes).entries.len() },
        }
    }

    pub fn update_entry(&mut self, index: usize, entry: Entry) {
        match *self {
            PageTable::Sv39(ref mut ptes) => unsafe {
                (**ptes).entries[index] = EntrySv39::from_entry(entry);
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VMMode {
    Sv39,
}

impl VMMode {
    pub fn value(self) -> usize {
        match self {
            VMMode::Sv39 => 8,
        }
    }

    pub fn page_size(self) -> usize {
        match self {
            VMMode::Sv39 => 4096,
        }
    }
}

pub struct VMManager {
    mode: VMMode,
}

unsafe impl Sync for VMManager {}
unsafe impl Send for VMManager {}

impl VMManager {
    pub fn new() -> Self {
        Self { mode: VMMode::Sv39 }
    }

    pub fn map_mem_identically_as_phy(&self) -> PageTable {
        let mut page_table = self.new_page_table();
        for i in 0..page_table.entry_length() {
            page_table.update_entry(
                i,
                Entry {
                    v: true,
                    w: true,
                    r: true,
                    x: true,
                    u: false,
                    ppn: i << 28,
                },
            )
        }

        page_table
    }

    pub fn new_page_table(&self) -> PageTable {
        match self.mode {
            VMMode::Sv39 => {
                assert_eq!(size_of::<PageTableSv39>(), 4096); // the page table size must be 4096 = 8 * 2^9

                // the PTEs must be aligned with one page size (= self.mode.page_size())
                let layout =
                    Layout::from_size_align(size_of::<PageTableSv39>(), self.mode.page_size())
                        .unwrap();
                let ptes_pointer = unsafe { alloc_zeroed(layout) };

                assert_eq!((ptes_pointer as usize) & 0xfff, 0_usize); // is aligned correctly?

                PageTable::Sv39(ptes_pointer as *mut PageTableSv39)
            }
        }
    }

    pub fn change_vm_mode(&mut self, mode: VMMode) {
        self.mode = mode;
    }

    pub fn enable_paging(&self, root_page_table: PageTable) {
        Csr::Satp.write(self.mode.value() << 60 | (root_page_table.address() >> 12));
        unsafe {
            asm!("sfence.vma zero, zero");
        }
    }
}
