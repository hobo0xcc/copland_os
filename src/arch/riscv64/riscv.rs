use crate::arch::riscv64::csr::*;
use crate::arch::CpuId;
use crate::interrupt::Backup;
use crate::lazy::Lazy;
use const_default::ConstDefault;
use core::arch::asm;

pub static mut STATE: Lazy<CpuState> = Lazy::<CpuState, fn() -> CpuState>::new(|| CpuState::new());

pub struct CpuState {
    disable_counter: usize,
}

impl CpuState {
    pub fn new() -> Self {
        Self { disable_counter: 0 }
    }

    pub fn cpuid(&self) -> CpuId {
        let mut id: usize;
        unsafe {
            asm!("mv {}, tp", out(reg)id);
        }

        id
    }

    pub fn interrupt_off(&mut self) {
        if self.disable_counter >= 1 {
            self.disable_counter -= 1;
        }
        if self.disable_counter == 0 {
            Csr::Sstatus.write(Csr::Sstatus.read() & !Sstatus::SIE.mask())
        }
    }

    pub fn interrupt_on(&mut self) {
        if self.disable_counter == 0 {
            Csr::Sstatus.write(Csr::Sstatus.read() | Sstatus::SIE.mask())
        }
        self.disable_counter += 1;
    }

    pub fn is_interrupt_on(&self) -> bool {
        Csr::Sstatus.read() & Sstatus::SIE.mask() != 0
    }
}

#[derive(ConstDefault)]
pub struct InterruptFlag {
    sstatus_mask: usize,
}

impl Backup for InterruptFlag {
    fn save_and_off() -> Self {
        let sstatus_mask = Csr::Sstatus.read() & Sstatus::SIE.mask();
        // Disable interrupt
        Csr::Sstatus.write(Csr::Sstatus.read() & !sstatus_mask);
        Self { sstatus_mask }
    }

    fn restore(&self) {
        Csr::Sstatus.write(Csr::Sstatus.read() | self.sstatus_mask);
    }
}
