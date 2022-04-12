use crate::arch::CpuId;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

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
