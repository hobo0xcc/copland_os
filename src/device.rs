#[cfg(target_arch = "riscv64")]
#[cfg(target_board = "virt")]
pub mod virt;

#[cfg(target_arch = "aarch64")]
#[cfg(target_board = "raspi3b")]
pub mod raspi3b;
