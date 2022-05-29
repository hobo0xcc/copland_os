#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

pub type CpuId = usize;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
