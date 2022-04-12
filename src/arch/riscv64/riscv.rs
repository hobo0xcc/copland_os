use crate::arch::riscv64::csr::*;
use crate::arch::CpuId;
use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref STATE: Mutex<CpuState> = Mutex::new(CpuState::new());
}

pub struct CpuState {
    disable_depth: usize,
    sstatus_sie: usize,
    sie: usize,
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            disable_depth: 0,
            sstatus_sie: 0,
            sie: 0,
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

    pub fn interrupt_disable(&mut self) {
        if self.disable_depth == 0 {
            self.sstatus_sie = Csr::Sstatus.read() & Sstatus::SIE.mask();
            self.sie = Csr::Sie.read();
            Csr::Sstatus.write(Csr::Sstatus.read() & !Sstatus::SIE.mask());
            Csr::Sie.write(0);
        }

        self.disable_depth += 1;
    }

    pub fn interrupt_enable(&mut self) {
        self.disable_depth -= 1;
        if self.disable_depth == 0 {
            Csr::Sstatus.write(Csr::Sstatus.read() | self.sstatus_sie);
            Csr::Sie.write(Csr::Sie.read() | self.sie);
            self.sstatus_sie = 0;
            self.sie = 0;
        }
    }
}
