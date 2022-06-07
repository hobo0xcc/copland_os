#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

pub type CpuId = usize;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

pub fn cpu_id() -> CpuId {
    #[cfg(target_arch = "riscv64")]
    return riscv64::riscv::STATE.lock().cpuid();
    #[cfg(target_arch = "aarch64")]
    return aarch64::arm::STATE.lock().cpuid();
}

pub fn interrupt_push() {
    #[cfg(target_arch = "riscv64")]
    riscv64::riscv::STATE.lock().interrupt_push();
}

pub fn interrupt_pop() {
    #[cfg(target_arch = "riscv64")]
    riscv64::riscv::STATE.lock().interrupt_pop();
}

pub fn is_interrupt_on() -> bool {
    #[cfg(target_arch = "riscv64")]
    if cfg!(target_arch = "riscv64") {
        riscv64::riscv::STATE.lock().is_interrupt_on()
    } else {
        false
    }
}
