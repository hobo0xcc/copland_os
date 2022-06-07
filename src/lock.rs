use crate::arch::CpuId;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct DummyMutexGuard<'a, T> {
    mutex: &'a DummyMutex<T>,
}

impl<T> Deref for DummyMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for DummyMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

pub struct DummyMutex<T> {
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for DummyMutex<T> {}
unsafe impl<T> Sync for DummyMutex<T> {}

impl<T> DummyMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> DummyMutexGuard<'_, T> {
        DummyMutexGuard { mutex: self }
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
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
                    return MutexGuard { mutex: self };
                }
            }
        }

        let guard = loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                break MutexGuard { mutex: self };
            }
        };

        let cpu_id: CpuId = crate::arch::cpu_id();

        unsafe {
            *self.cpu_id.get() = Some(cpu_id);
        }

        guard
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct KernelLock {
    locked: AtomicBool,
    cpu_id: UnsafeCell<Option<CpuId>>,
    intr: UnsafeCell<bool>,
}

unsafe impl Sync for KernelLock {}
unsafe impl Send for KernelLock {}

impl KernelLock {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            cpu_id: UnsafeCell::new(None),
            intr: UnsafeCell::new(false),
        }
    }

    #[allow(unused_variables)]
    pub fn lock(&self) {
        unsafe {
            if let Some(cpu_id) = *self.cpu_id.get() {
                if cpu_id == crate::arch::cpu_id() {
                    return;
                }
            }
        }

        crate::arch::interrupt_push();

        loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                break;
            }
        }

        let cpu_id: CpuId = crate::arch::cpu_id();

        unsafe {
            *self.cpu_id.get() = Some(cpu_id);
        }
    }

    #[allow(unused_variables)]
    pub fn unlock(&self) {
        let cpu_id: CpuId = crate::arch::cpu_id();

        unsafe {
            if let Some(saved_cpu_id) = *self.cpu_id.get() {
                if cpu_id != saved_cpu_id {
                    return;
                }
            } else {
                panic!("unlock without lock");
            }
            *self.cpu_id.get() = None;
        }

        self.locked.store(false, Ordering::Release);

        crate::arch::interrupt_pop();
    }

    pub unsafe fn complete_intr(&self) {
        *self.intr.get() = true;
    }

    pub unsafe fn wait_intr(&self) {
        *self.intr.get() = false;
        crate::arch::interrupt_pop();
        assert!(crate::arch::is_interrupt_on());
        while !*self.intr.get() {}
        assert!(*self.intr.get());
        crate::arch::interrupt_push();
        *self.intr.get() = false;
    }
}
