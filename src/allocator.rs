#![allow(unused_imports)]

pub mod dlmalloc;
pub mod watermark;

use core::alloc::Layout;
use log::info;

#[cfg(allocator = "WaterMark")]
#[cfg(not(target_arch = "x86_64"))]
use watermark::WaterMarkAllocator;

#[cfg(allocator = "Dlmalloc")]
#[cfg(not(target_arch = "x86_64"))]
use self::dlmalloc::GlobalDlmalloc;

#[cfg(target_arch = "x86_64")]
use uefi::prelude::BootServices;

#[global_allocator]
#[cfg(allocator = "WaterMark")]
#[cfg(not(target_arch = "x86_64"))]
static mut ALLOCATOR: WaterMarkAllocator = WaterMarkAllocator::empty();

#[global_allocator]
#[cfg(allocator = "Dlmalloc")]
#[cfg(not(target_arch = "x86_64"))]
static mut ALLOCATOR: GlobalDlmalloc = GlobalDlmalloc;

#[cfg(allocator = "WaterMark")]
#[cfg(not(target_arch = "x86_64"))]
pub fn init_allocator() {
    #[cfg(target_arch = "aarch64")]
    use crate::arch::aarch64::address;
    #[cfg(target_arch = "riscv64")]
    use crate::arch::riscv64::address;

    info!("Initialize WaterMark allocator");

    let heap_start = address::_heap_start as usize;
    let heap_end = address::_heap_end as usize;
    unsafe {
        ALLOCATOR = WaterMarkAllocator::new(heap_start, heap_end);
    }
}

#[cfg(allocator = "Dlmalloc")]
#[cfg(not(target_arch = "x86_64"))]
pub fn init_allocator() {
    info!("Initialize Dlmalloc allocator");
}

#[cfg(target_arch = "x86_64")]
pub fn init_allocator(boot_services: &BootServices) {
    unsafe { uefi::alloc::init(boot_services) };
}

#[alloc_error_handler]
fn oom_handler(_layout: Layout) -> ! {
    panic!("Out of memory!");
}
