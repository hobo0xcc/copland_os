use crate::arch::CpuId;
use crate::interrupt::{ArchInterruptFlag, Backup};
use const_default::ConstDefault;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct KernelLock {
    locked: AtomicBool,
    cpu_id: UnsafeCell<Option<CpuId>>,
    intr: UnsafeCell<bool>,
    intr_flag: UnsafeCell<ArchInterruptFlag>,
}

unsafe impl Sync for KernelLock {}
unsafe impl Send for KernelLock {}

impl KernelLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            cpu_id: UnsafeCell::new(None),
            intr: UnsafeCell::new(false),
            intr_flag: UnsafeCell::new(ArchInterruptFlag::DEFAULT),
        }
    }

    #[allow(unused_variables)]
    pub unsafe fn lock(&self) {
        if let Some(cpu_id) = *self.cpu_id.get() {
            if cpu_id == crate::arch::cpu_id() {
                return;
            }
        }

        loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                *self.intr_flag.get() = ArchInterruptFlag::save_and_off();
                break;
            }
        }

        let cpu_id: CpuId = crate::arch::cpu_id();
        *self.cpu_id.get() = Some(cpu_id);
    }

    #[allow(unused_variables)]
    pub unsafe fn unlock(&self) {
        let cpu_id: CpuId = crate::arch::cpu_id();

        if let Some(saved_cpu_id) = *self.cpu_id.get() {
            if cpu_id != saved_cpu_id {
                return;
            }
        } else {
            panic!("unlock without lock");
        }
        *self.cpu_id.get() = None;

        self.locked.store(false, Ordering::Release);
        self.intr_flag.get().as_ref().unwrap().restore();
    }

    pub unsafe fn complete_intr(&self) {
        *self.intr.get() = true;
    }

    pub unsafe fn wait_intr(&self) {
        *self.intr.get() = false;
        self.intr_flag.get().as_ref().unwrap().restore();
        assert!(crate::arch::is_interrupt_on());
        while !*self.intr.get() {}
        assert!(*self.intr.get());
        *self.intr_flag.get() = ArchInterruptFlag::save_and_off();
        assert!(!crate::arch::is_interrupt_on());
        *self.intr.get() = false;
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    intr_flag: ArchInterruptFlag,
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

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            *self.mutex.cpu_id.get() = None;
        }
        self.mutex.locked.store(false, Ordering::Release);
        self.intr_flag.restore();
    }
}

unsafe impl<T> Sync for Mutex<T> {}
unsafe impl<T> Send for Mutex<T> {}

pub struct Mutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
    cpu_id: UnsafeCell<Option<CpuId>>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
            cpu_id: UnsafeCell::new(None),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        unsafe {
            if let Some(cpu_id) = *self.cpu_id.get() {
                if cpu_id == crate::arch::cpu_id() {
                    return MutexGuard {
                        mutex: self,
                        intr_flag: ArchInterruptFlag::save_and_off(),
                    };
                }
            }
        }

        let guard = loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                break MutexGuard {
                    mutex: self,
                    intr_flag: ArchInterruptFlag::save_and_off(),
                };
            }
        };

        let cpu_id: CpuId = crate::arch::cpu_id();

        unsafe {
            *self.cpu_id.get() = Some(cpu_id);
        }

        guard
    }

    pub fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}
