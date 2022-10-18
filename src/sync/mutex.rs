use crate::arch::{self, CpuId};
use crate::interrupt::{ArchInterruptFlag, Backup};
use const_default::ConstDefault;
use core::cell::UnsafeCell;
use core::convert::{AsMut, AsRef};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct KernelLock {
    locked: AtomicBool,
    // cpu_id: UnsafeCell<Option<CpuId>>,
    intr: AtomicBool,
    // intr_flag: UnsafeCell<ArchInterruptFlag>,
}

unsafe impl Sync for KernelLock {}
unsafe impl Send for KernelLock {}

impl KernelLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            intr: AtomicBool::new(false),
        }
    }

    #[allow(unused_variables)]
    pub unsafe fn lock(&self) {
        loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                // *self.intr_flag.get() = ArchInterruptFlag::save_and_off();
                arch::interrupt_off();
                break;
            }
        }
    }

    #[allow(unused_variables)]
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
        // self.intr_flag.get().as_ref().unwrap().restore();
        arch::interrupt_on();
    }

    pub unsafe fn complete_intr(&self) {
        self.intr.store(true, Ordering::SeqCst);
        // self.intr.get().write(true);
    }

    pub unsafe fn wait_interrupt(&self) {
        self.intr.store(false, Ordering::SeqCst);
        // self.intr_flag.get().as_ref().unwrap().restore();
        crate::arch::interrupt_on();
        assert!(crate::arch::is_interrupt_on());
        // println!("wait_intr1");
        while !self.intr.load(Ordering::SeqCst) {}
        // println!("wait_intr2");
        assert!(self.intr.load(Ordering::SeqCst));
        // *self.intr_flag.get() = ArchInterruptFlag::save_and_off();
        crate::arch::interrupt_off();
        assert!(!crate::arch::is_interrupt_on());
        self.intr.store(false, Ordering::SeqCst);
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    // intr_flag: ArchInterruptFlag,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> AsRef<T> for MutexGuard<'_, T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T> AsMut<T> for MutexGuard<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        arch::interrupt_on();
        // self.intr_flag.restore();
    }
}

pub struct Mutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
    // cpu_id: UnsafeCell<Option<CpuId>>,
}

unsafe impl<T> Sync for Mutex<T> {}
unsafe impl<T> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
            // cpu_id: UnsafeCell::new(None),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        let guard = loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                arch::interrupt_off();
                break MutexGuard {
                    mutex: self,
                    // intr_flag: ArchInterruptFlag::save_and_off(),
                };
            }
        };

        guard
    }

    pub fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}
