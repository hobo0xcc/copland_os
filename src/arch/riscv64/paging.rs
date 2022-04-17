use crate::arch::riscv64::csr::*;
use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref VM_MANAGER: Mutex<VMManager> = Mutex::new(VMManager::new());
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
}

pub struct VMManager {}

impl VMManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn enable_paging(&self, root_page_table: usize, mode: VMMode) {
        Csr::Satp.write(mode.value() << 60 | (root_page_table >> 12));
        unsafe {
            asm!("sfence.vma zero, zero");
        }
    }
}
