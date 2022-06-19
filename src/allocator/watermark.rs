use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

pub struct WaterMarkAllocator {
    current_position: UnsafeCell<usize>,
    heap_end: usize,
}

impl WaterMarkAllocator {
    pub fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            current_position: UnsafeCell::new(heap_start),
            heap_end,
        }
    }

    pub const fn empty() -> Self {
        Self {
            current_position: UnsafeCell::new(0),
            heap_end: 0,
        }
    }
}

unsafe impl GlobalAlloc for WaterMarkAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let curr = *self.current_position.get();
        let alloc_at = curr + (align - curr % align);
        let new_position = alloc_at + layout.size();
        if new_position >= self.heap_end {
            panic!("Allocaion failed: {:?}, current_position: {}", layout, curr);
        }
        *self.current_position.get() = new_position;

        alloc_at as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

unsafe impl Sync for WaterMarkAllocator {}
