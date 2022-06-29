use crate::arch::riscv64::csr::*;
use crate::arch::riscv64::task::{trampoline, TRAMPOLINE};
use crate::error::VMError;
use crate::lazy::Lazy;
use alloc::alloc::*;
use alloc::string::{String, ToString};
use alloc::vec;
use bitflags::bitflags;
use core::arch::asm;
use hashbrown::HashMap;
use log::info;

pub const LEVELS: usize = 3;
pub const PAGE_SIZE: usize = 4096;

pub static mut VM_MANAGER: Lazy<VMManager> =
    Lazy::<VMManager, fn() -> VMManager>::new(|| VMManager::new());

bitflags! {
    struct PTE: usize {
        const V = 0b1 << 0;
        const R = 0b1 << 1;
        const W = 0b1 << 2;
        const X = 0b1 << 3;
        const U = 0b1 << 4;
        const PPN = 0xfff_ffff_ffff << 10;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Entry(usize);

impl Entry {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn is_valid(&self) -> bool {
        (self.0 & PTE::V.bits()) != 0
    }

    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    pub fn is_readable(&self) -> bool {
        (self.0 & PTE::R.bits()) != 0
    }

    pub fn is_writable(&self) -> bool {
        (self.0 & PTE::W.bits()) != 0
    }

    pub fn is_executable(&self) -> bool {
        (self.0 & PTE::X.bits()) != 0
    }

    pub fn is_user_accessible(&self) -> bool {
        (self.0 & PTE::U.bits()) != 0
    }

    pub fn is_leaf(&self) -> bool {
        (self.0 & PTE::R.bits()) != 0 || (self.0 & PTE::X.bits()) != 0
    }

    pub fn is_next_ptr(&self) -> bool {
        !self.is_leaf()
    }

    pub fn as_next_ptr(&mut self) {
        self.set_flags(true, false, false, false, false);
    }

    pub fn set_flags(&mut self, v: bool, r: bool, w: bool, x: bool, u: bool) {
        if v {
            self.0 |= PTE::V.bits();
        }
        if r {
            self.0 |= PTE::R.bits();
        }
        if w {
            self.0 |= PTE::W.bits();
        }
        if x {
            self.0 |= PTE::X.bits();
        }
        if u {
            self.0 |= PTE::U.bits();
        }
    }

    pub fn set_ppn(&mut self, ppn: usize) {
        self.0 |= ppn;
    }

    pub fn get_ppn(&self) -> usize {
        self.0 & PTE::PPN.bits()
    }
}

#[repr(C)]
pub struct PageTable {
    entries: [Entry; 512],
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

    pub fn identity_mapping(&self, table: *mut PageTable, level: usize, old_ppn: usize) {
        for i in 0..512 {
            unsafe {
                let entry = &mut (*table).entries[i];
                entry.set_ppn((i << (10 + 9 * level)) | old_ppn);
                entry.set_flags(true, true, true, true, false);
            }
        }
    }

    pub fn map_page(
        &self,
        mut table: *mut PageTable,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
        u: bool,
    ) -> Result<(), VMError> {
        let vpn = vec![
            (vaddr >> 12) & 0x1ff,
            (vaddr >> 21) & 0x1ff,
            (vaddr >> 30) & 0x1ff,
        ];
        unsafe {
            for level in (1..LEVELS).rev() {
                let entry = (*table).entries[vpn[level]];
                if entry.is_leaf() || entry.is_invalid() {
                    let new_table = self.create_table();
                    let ppn = if entry.is_invalid() {
                        0
                    } else {
                        entry.get_ppn()
                    };
                    self.identity_mapping(new_table, level - 1, ppn);
                    let mut new_entry = Entry::new();
                    new_entry.as_next_ptr();
                    new_entry.set_ppn((new_table as usize) >> 2);
                    (*table).entries[vpn[level]] = new_entry;
                    table = new_table;
                } else {
                    let new_table = ((*table).entries[vpn[level]].get_ppn() << 2) as *mut PageTable;
                    table = new_table;
                }
            }
            let mut new_entry = Entry::new();
            new_entry.set_flags(true, r, w, x, u);
            new_entry.set_ppn(paddr >> 2);
            (*table).entries[vpn[0]] = new_entry;
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
        self.map_page(table, paddr, vaddr, r, w, x, u)?;
        Ok(())
    }

    pub fn create_table(&self) -> *mut PageTable {
        unsafe {
            let layout = Layout::from_size_align(0x1000, 0x1000).unwrap();
            alloc_zeroed(layout) as *mut PageTable
        }
    }

    pub fn get_table(&self, name: &str) -> *mut PageTable {
        **self.root_tables.get(name).as_mut().unwrap()
    }

    pub fn set_table(&mut self, name: String, table: *mut PageTable) {
        self.root_tables.insert(name, table);
    }

    pub fn make_satp(&self, name: &str) -> usize {
        // Sv39
        (8 << 60) | (self.get_table(name) as usize >> 12)
    }

    pub fn init(&mut self) {
        info!("Initialize VM Manager");
        let root_table = self.create_table();
        self.set_table("kernel".to_string(), root_table);
        unsafe {
            let root_table = self.get_table("kernel");
            self.identity_mapping(root_table, 2, 0);
            self.map(
                "kernel",
                trampoline as usize,
                TRAMPOLINE,
                true,
                true,
                true,
                false,
            )
            .unwrap();
            Csr::Satp.write(self.make_satp("kernel"));
            asm!("sfence.vma zero, zero");
        }
    }
}
