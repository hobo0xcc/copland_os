use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::ptr;
use dlmalloc::Allocator;
use dlmalloc::Dlmalloc;

pub struct System {
    pos: UnsafeCell<usize>,
}

impl System {
    pub const fn new() -> Self {
        Self {
            pos: UnsafeCell::new(0),
        }
    }
}

unsafe impl Send for System {}
unsafe impl Sync for System {}

unsafe impl Allocator for System {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        #[cfg(target_arch = "aarch64")]
        use crate::arch::aarch64::address::_heap_start;
        #[cfg(target_arch = "riscv64")]
        use crate::arch::riscv64::address::_heap_start;
        let prev = unsafe {
            if *self.pos.get() == 0 {
                *self.pos.get() = _heap_start as usize;
            }
            let prev = *self.pos.get();
            *self.pos.get() += size;
            prev
        };
        if prev == usize::max_value() {
            return (ptr::null_mut(), 0, 0);
        }
        (prev as *mut u8, size, 0)
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        false
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        64 * 1024
    }
}

/// An instance of a "global allocator" backed by `Dlmalloc`
///
/// This API requires the `global` feature is activated, and this type
/// implements the `GlobalAlloc` trait in the standard library.
pub struct GlobalDlmalloc;

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc<System>>::malloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        <Dlmalloc<System>>::free(&mut get(), ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc<System>>::calloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        <Dlmalloc<System>>::realloc(&mut get(), ptr, layout.size(), layout.align(), new_size)
    }
}

static mut DLMALLOC: Dlmalloc<System> = Dlmalloc::<System>::new_with_allocator(System::new());

struct Instance;

unsafe fn get() -> Instance {
    Instance
}

impl Deref for Instance {
    type Target = Dlmalloc<System>;
    fn deref(&self) -> &Dlmalloc<System> {
        unsafe { &DLMALLOC }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Dlmalloc<System> {
        unsafe { &mut DLMALLOC }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {}
}
