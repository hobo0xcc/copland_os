#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

pub type CpuId = usize;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

pub fn cpu_id() -> CpuId {
    #[cfg(target_arch = "riscv64")]
    return unsafe { riscv64::riscv::STATE.cpuid() };
    #[cfg(target_arch = "aarch64")]
    return unsafe { aarch64::arm::STATE.cpuid() };
}

pub fn is_interrupt_on() -> bool {
    #[cfg(target_arch = "riscv64")]
    return unsafe { riscv64::riscv::STATE.is_interrupt_on() };
    #[cfg(target_arch = "aarch64")]
    false
}
