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
        self.mutex.locked.store(false, Ordering::Release);
        #[cfg(target_arch = "riscv64")]
        crate::arch::riscv64::riscv::STATE.lock().interrupt_enable();
    }
}

pub struct Mutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        #[cfg(target_arch = "riscv64")]
        crate::arch::riscv64::riscv::STATE
            .lock()
            .interrupt_disable();

        loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                break MutexGuard { mutex: self };
            }
        }
    }
}

pub struct KernelLock {
    locked: AtomicBool,
    cpu_id: UnsafeCell<CpuId>,
}

unsafe impl Sync for KernelLock {}

impl KernelLock {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            cpu_id: UnsafeCell::new(0),
        }
    }

    #[allow(unused_variables)]
    pub fn lock(&self) {
        #[cfg(target_arch = "riscv64")]
        crate::arch::riscv64::riscv::STATE
            .lock()
            .interrupt_disable();

        loop {
            if let Ok(_) =
                self.locked
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                break;
            }
        }

        let cpu_id: CpuId = 0;

        #[cfg(target_arch = "riscv64")]
        let cpu_id = crate::arch::riscv64::riscv::STATE.lock().cpuid();

        unsafe {
            *self.cpu_id.get() = cpu_id;
        }
    }

    #[allow(unused_variables)]
    pub fn unlock(&self) {
        let cpu_id: CpuId = 0;

        #[cfg(target_arch = "riscv64")]
        let cpu_id = crate::arch::riscv64::riscv::STATE.lock().cpuid();

        #[cfg(target_arch = "aarch64")]
        let cpu_id = crate::arch::aarch64::arm::STATE.lock().cpuid();

        // unlock from other cpus is refused.
        unsafe {
            if cpu_id != *self.cpu_id.get() {
                return;
            }
        }

        self.locked.store(false, Ordering::Release);

        #[cfg(target_arch = "riscv64")]
        crate::arch::riscv64::riscv::STATE.lock().interrupt_enable();
    }
}
