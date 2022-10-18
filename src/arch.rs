#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub type CpuId = usize;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

pub fn cpu_id() -> CpuId {
    #[cfg(target_arch = "riscv64")]
    return unsafe { riscv64::riscv::STATE.cpuid() };
    #[cfg(target_arch = "aarch64")]
    return unsafe { aarch64::arm::STATE.cpuid() };
    #[cfg(target_arch = "x86_64")]
    return 0;
}

pub fn interrupt_off() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        riscv64::riscv::STATE.interrupt_off();
    }
}

pub fn interrupt_on() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        riscv64::riscv::STATE.interrupt_on();
    }
}

pub fn is_interrupt_on() -> bool {
    #[cfg(target_arch = "riscv64")]
    return unsafe { riscv64::riscv::STATE.is_interrupt_on() };
    #[cfg(target_arch = "aarch64")]
    return false;
    #[cfg(target_arch = "x86_64")]
    return false;
}
