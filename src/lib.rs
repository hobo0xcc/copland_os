#![no_std]
#![no_main]
#![feature(alloc_error_handler, once_cell, never_type, panic_info_message)]

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

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Panic: ");
    if let Some(location) = info.location() {
        println!(
            "line: {}, file: {}: {}",
            location.line(),
            location.file(),
            info.message().unwrap()
        );
    } else {
        println!("No information available");
    }
    loop {}
}
