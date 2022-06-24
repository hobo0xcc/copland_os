use crate::arch::riscv64::csr::*;
use crate::arch::CpuId;
use crate::sync::lazy::Lazy;
use core::arch::asm;

pub static mut STATE: Lazy<CpuState> = Lazy::<CpuState, fn() -> CpuState>::new(|| CpuState::new());

pub struct CpuState {
    disable_depth: usize,
    sstatus_sie: usize,
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            disable_depth: 0,
            sstatus_sie: 0,
        }
    }

    pub fn cpuid(&self) -> CpuId {
        let mut id: usize;
        unsafe {
            asm!("mv {}, tp", out(reg)id);
        }

        id
    }

    pub fn interrupt_off(&self) {
        Csr::Sstatus.write(Csr::Sstatus.read() & !Sstatus::SIE.mask())
    }

    pub fn interrupt_on(&self) {
        Csr::Sstatus.write(Csr::Sstatus.read() | Sstatus::SIE.mask())
    }

    pub fn interrupt_push(&mut self) {
        if self.disable_depth == 0 {
            self.sstatus_sie = Csr::Sstatus.read() & Sstatus::SIE.mask();
            Csr::Sstatus.write(Csr::Sstatus.read() & !Sstatus::SIE.mask());
        }

        self.disable_depth += 1;
    }

    pub fn interrupt_pop(&mut self) {
        self.disable_depth -= 1;
        if self.disable_depth == 0 {
            Csr::Sstatus.write(Csr::Sstatus.read() | self.sstatus_sie);
            self.sstatus_sie = 0;
        }
    }

    pub fn is_interrupt_on(&self) -> bool {
        Csr::Sstatus.read() & Sstatus::SIE.mask() != 0
    }
}
