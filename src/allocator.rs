pub mod watermark;

use core::alloc::Layout;
use watermark::WaterMarkAllocator;

#[cfg(target_arch = "riscv64")]
use crate::arch::riscv64::address;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::address;

#[global_allocator]
static mut ALLOCATOR: WaterMarkAllocator = WaterMarkAllocator::empty();

pub fn init_allocator() {
    let heap_start = address::_heap_start as usize;
    let heap_end = address::_heap_end as usize;
    unsafe {
        ALLOCATOR = WaterMarkAllocator::new(heap_start, heap_end);
    }
}

#[alloc_error_handler]
fn oom_handler(_layout: Layout) -> ! {
    panic!("Out of memory!");
}
