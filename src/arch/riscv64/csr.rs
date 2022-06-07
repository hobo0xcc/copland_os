use core::arch::asm;
use core::panic;
use core::prelude::rust_2021::{derive, Debug};

macro_rules! field_info {
    ($name:ident) => {
        impl $name {
            pub fn index(&self) -> usize {
                (*self as usize).trailing_zeros() as usize
            }

            pub fn mask(&self) -> usize {
                *self as usize
            }
        }
    };
}

field_info!(Mstatus);

#[repr(usize)]
#[derive(Copy, Clone)]
pub enum Mstatus {
    MPP = 0b11 << 11,
    SPIE = 0b1 << 5,
    MPIE = 0b1 << 7,
    SIE = 0b1 << 1,
    MIE = 0b1 << 3,
    FS = 0b11 << 13,
}

field_info!(Sstatus);

#[repr(usize)]
#[derive(Copy, Clone)]
pub enum Sstatus {
    SPIE = 0b1 << 5,
    SIE = 0b1 << 1,
    FS = 0b11 << 13,
}

field_info!(Mie);

#[repr(usize)]
#[derive(Copy, Clone)]
pub enum Mie {
    MTIE = 0b1 << 7,
}

field_info!(Sie);

#[repr(usize)]
#[derive(Copy, Clone)]
pub enum Sie {
    SSIE = 0b1 << 1,
    STIE = 0b1 << 5,
    SEIE = 0b1 << 9,
}

#[derive(Debug)]
pub enum Csr {
    Misa,
    Mvendorid,
    Marchid,
    Mimpid,
    Mhartid,
    Mstatus,
    Mtvec,
    Medeleg,
    Mideleg,
    Mip,
    Mie,
    Mcounteren,
    Mcountinhibit,
    Mscratch,
    Mepc,
    Mcause,
    Mtval,

    Sstatus,
    Stvec,
    Sip,
    Sie,
    Scounteren,
    Sscratch,
    Sepc,
    Scause,
    Stval,
    Satp,
}

#[allow(unreachable_patterns)]
impl Csr {
    pub fn read(&self) -> usize {
        let mut val: usize;
        unsafe {
            match *self {
                Csr::Misa => asm!("csrr {}, misa", out(reg)val),
                Csr::Mvendorid => asm!("csrr {}, mvendorid", out(reg)val),
                Csr::Marchid => asm!("csrr {}, marchid", out(reg)val),
                Csr::Mimpid => asm!("csrr {}, mimpid", out(reg)val),
                Csr::Mhartid => asm!("csrr {}, mhartid", out(reg)val),
                Csr::Mstatus => asm!("csrr {}, mstatus", out(reg)val),
                Csr::Mtvec => asm!("csrr {}, mtvec", out(reg)val),
                Csr::Medeleg => asm!("csrr {}, medeleg", out(reg)val),
                Csr::Mideleg => asm!("csrr {}, mideleg", out(reg)val),
                Csr::Mip => asm!("csrr {}, mip", out(reg)val),
                Csr::Mie => asm!("csrr {}, mie", out(reg)val),
                Csr::Mcounteren => asm!("csrr {}, mcounteren", out(reg)val),
                Csr::Mcountinhibit => asm!("csrr {}, mcountinhibit", out(reg)val),
                Csr::Mscratch => asm!("csrr {}, mscratch", out(reg)val),
                Csr::Mepc => asm!("csrr {}, mepc", out(reg)val),
                Csr::Mcause => asm!("csrr {}, mcause", out(reg)val),
                Csr::Mtval => asm!("csrr {}, mtval", out(reg)val),
                Csr::Sstatus => asm!("csrr {}, sstatus", out(reg)val),
                Csr::Stvec => asm!("csrr {}, stvec", out(reg)val),
                Csr::Sip => asm!("csrr {}, sip", out(reg)val),
                Csr::Sie => asm!("csrr {}, sie", out(reg)val),
                Csr::Scounteren => asm!("csrr {}, scounteren", out(reg)val),
                Csr::Sscratch => asm!("csrr {}, sscratch", out(reg)val),
                Csr::Sepc => asm!("csrr {}, sepc", out(reg)val),
                Csr::Scause => asm!("csrr {}, scause", out(reg)val),
                Csr::Stval => asm!("csrr {}, stval", out(reg)val),
                Csr::Satp => asm!("csrr {}, satp", out(reg)val),
                _ => panic!("unimplemented csr: {:?}", *self),
            }
        }
        val
    }

    pub fn write(&self, val: usize) {
        unsafe {
            match *self {
                Csr::Misa => asm!("csrw misa, {}", in(reg)val),
                Csr::Mvendorid => asm!("csrw mvendorid, {}", in(reg)val),
                Csr::Marchid => asm!("csrw marchid, {}", in(reg)val),
                Csr::Mimpid => asm!("csrw mimpid, {}", in(reg)val),
                Csr::Mhartid => asm!("csrw mhartid, {}", in(reg)val),
                Csr::Mstatus => asm!("csrw mstatus, {}", in(reg)val),
                Csr::Mtvec => asm!("csrw mtvec, {}", in(reg)val),
                Csr::Medeleg => asm!("csrw medeleg, {}", in(reg)val),
                Csr::Mideleg => asm!("csrw mideleg, {}", in(reg)val),
                Csr::Mip => asm!("csrw mip, {}", in(reg)val),
                Csr::Mie => asm!("csrw mie, {}", in(reg)val),
                Csr::Mcounteren => asm!("csrw mcounteren, {}", in(reg)val),
                Csr::Mcountinhibit => asm!("csrw mcountinhibit, {}", in(reg)val),
                Csr::Mscratch => asm!("csrw mscratch, {}", in(reg)val),
                Csr::Mepc => asm!("csrw mepc, {}", in(reg)val),
                Csr::Mcause => asm!("csrw mcause, {}", in(reg)val),
                Csr::Mtval => asm!("csrw mtval, {}", in(reg)val),
                Csr::Sstatus => asm!("csrw sstatus, {}", in(reg)val),
                Csr::Stvec => asm!("csrw stvec, {}", in(reg)val),
                Csr::Sip => asm!("csrw sip, {}", in(reg)val),
                Csr::Sie => asm!("csrw sie, {}", in(reg)val),
                Csr::Scounteren => asm!("csrw scounteren, {}", in(reg)val),
                Csr::Sscratch => asm!("csrw sscratch, {}", in(reg)val),
                Csr::Sepc => asm!("csrw sepc, {}", in(reg)val),
                Csr::Scause => asm!("csrw scause, {}", in(reg)val),
                Csr::Stval => asm!("csrw stval, {}", in(reg)val),
                Csr::Satp => asm!("csrw satp, {}", in(reg)val),
                _ => panic!("unimplemented csr: {:?}", *self),
            }
        }
    }
}
