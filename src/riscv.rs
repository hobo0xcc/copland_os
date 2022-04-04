use crate::csr::*;
use core::arch::asm;

pub fn cpuid() -> usize {
    let mut id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg)id);
    }

    id
}

#[allow(unused)]
pub fn interrupt_off() {
    Csr::Sstatus.write(Csr::Sstatus.read() & !Sstatus::SIE.mask())
}

#[allow(unused)]
pub fn interrupt_on() {
    Csr::Sstatus.write(Csr::Sstatus.read() | Sstatus::SIE.mask())
}
