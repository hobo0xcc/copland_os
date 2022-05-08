use crate::arch::riscv64::*;
use lazy_static::lazy_static;
use spin::Mutex;

pub const UART0_IRQ: usize = 10;

lazy_static! {
    pub static ref PLIC_MANAGER: Mutex<PLICManager> = Mutex::new(PLICManager::new());
}

// https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc#memory-map
// https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/plic.c

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
pub enum PlicIRQ {
    Uart0 = 10,
}

pub struct PLICManager {}

unsafe impl Sync for PLICManager {}
unsafe impl Send for PLICManager {}

impl PLICManager {
    pub fn new() -> Self {
        Self {}
    }

    fn enable(&self, hart: usize, source: PlicIRQ) {
        let enable_base = (address::_plic_start as usize) + 0x2000;
        unsafe {
            ((enable_base + hart * 0x100 + 0x80) as *mut u32)
                .add(source as usize / 32)
                .write_volatile(1 << (source as usize % 32));
        }
    }

    fn update_threshold(&self, hart: usize, threshold: u32) {
        let threshold_base = (address::_plic_start as usize) + 0x200000;
        unsafe {
            ((threshold_base + hart * 0x2000 + 0x1000) as *mut u32).write_volatile(threshold);
        }
    }

    fn update_priority(&self, source: usize, priority: u32) {
        unsafe {
            (address::_plic_start as *mut u32)
                .add(source)
                .write_volatile(priority);
        }
    }

    fn claim_address(&self, hart: usize) -> *mut u32 {
        let claim_base = (address::_plic_start as usize) + 0x200000;
        unsafe { ((claim_base + hart * 0x2000 + 0x1000) as *mut u32).add(1) }
    }

    pub fn init_irq(&mut self, source: PlicIRQ) {
        let hart = riscv::STATE.lock().cpuid();
        self.update_priority(source as usize, 1);
        self.enable(hart, source);
        self.update_threshold(hart, 0);
    }

    pub fn read_claim(&self) -> u32 {
        let hart = riscv::STATE.lock().cpuid();
        unsafe { self.claim_address(hart).read_volatile() }
    }

    pub unsafe fn send_complete(&self, irq: u32) {
        let hart = riscv::STATE.lock().cpuid();
        self.claim_address(hart).write_volatile(irq);
    }
}
