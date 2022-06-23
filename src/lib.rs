#![feature(alloc_error_handler, once_cell)]
#![no_std]
#![no_main]

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod device;
pub mod fs;
pub mod logger;
pub mod print;
pub mod sandbox;
pub mod sync;
pub mod task;

use sync::mutex::KernelLock;

pub static mut KERNEL_LOCK: KernelLock = KernelLock::new();
