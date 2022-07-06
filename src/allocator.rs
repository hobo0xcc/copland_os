#![allow(unused_imports)]

pub mod dlmalloc;
pub mod watermark;

use core::alloc::Layout;
use log::info;

#[cfg(allocator = "WaterMark")]
use watermark::WaterMarkAllocator;

#[cfg(allocator = "Dlmalloc")]
use self::dlmalloc::GlobalDlmalloc;

#[global_allocator]
#[cfg(allocator = "WaterMark")]
static mut ALLOCATOR: WaterMarkAllocator = WaterMarkAllocator::empty();

#[global_allocator]
#[cfg(allocator = "Dlmalloc")]
static mut ALLOCATOR: GlobalDlmalloc = GlobalDlmalloc;

#[cfg(allocator = "WaterMark")]
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
pub fn init_allocator() {
    info!("Initialize Dlmalloc allocator");
}

#[alloc_error_handler]
#[cfg(not(target_board = "uefi"))]
fn oom_handler(_layout: Layout) -> ! {
    panic!("Out of memory!");
}
