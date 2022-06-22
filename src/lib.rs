#![feature(alloc_error_handler, once_cell)]
#![no_std]
#![no_main]

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod device;
pub mod fs;
pub mod lazy;
pub mod lock;
pub mod logger;
pub mod print;
pub mod sandbox;
pub mod task;

use lock::KernelLock;

pub static mut KERNEL_LOCK: KernelLock = KernelLock::new();
