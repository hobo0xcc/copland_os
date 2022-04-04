pub mod watermark;

use crate::arch::riscv64::address::_heap_end;
use crate::arch::riscv64::address::_heap_start;
use core::alloc::Layout;
use watermark::WaterMarkAllocator;

#[global_allocator]
static mut ALLOCATOR: WaterMarkAllocator = WaterMarkAllocator::empty();

pub fn init_allocator() {
    let heap_start = _heap_start as usize;
    let heap_end = _heap_end as usize;
    unsafe {
        ALLOCATOR = WaterMarkAllocator::new(heap_start, heap_end);
    }
}

#[alloc_error_handler]
fn oom_handler(_layout: Layout) -> ! {
    panic!("Out of memory!");
}
