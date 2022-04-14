use crate::arch::CpuId;
use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref STATE: Mutex<CpuState> = Mutex::new(CpuState::new());
}

pub struct CpuState {}

impl CpuState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn cpuid(&self) -> CpuId {
        let mut id: CpuId;
        unsafe {
            asm!("mrs {}, mpidr_el1", out(reg)id);
        }

        id = id & 0b11;

        id
    }
}
