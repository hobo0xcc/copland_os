use crate::arch::CpuId;
use crate::lazy::Lazy;
use core::arch::asm;

pub static mut STATE: Lazy<CpuState> = Lazy::new(|| CpuState::new());

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

        id & 0b11
    }
}
