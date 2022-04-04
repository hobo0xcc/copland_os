use crate::address;
use crate::*;

pub const UART0_IRQ: usize = 10;

// https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc#memory-map
// https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/plic.c

unsafe fn priority(source: usize) -> *mut u32 {
    (address::_plic_start as *mut u32).add(source)
}

// Return a pointer to a word in which an enable bit of the interrupt source is contained.
// The enable bit can be accessed with the formula: *ptr >> (source % 32) & 1.
unsafe fn s_mode_enable(hart: usize, source: usize) -> *mut u32 {
    let enable_base = (address::_plic_start as usize) + 0x2000;
    ((enable_base + hart * 0x100 + 0x80) as *mut u32).add(source / 32)
}

unsafe fn s_mode_threshold(hart: usize) -> *mut u32 {
    let threshold_base = (address::_plic_start as usize) + 0x200000;
    ((threshold_base + hart * 0x2000 + 0x1000) as *mut u32)
}

unsafe fn s_mode_claim(hart: usize) -> *mut u32 {
    let claim_base = (address::_plic_start as usize) + 0x200000;
    ((claim_base + hart * 0x2000 + 0x1000) as *mut u32).add(1)
}

pub unsafe fn plic_init() {
    priority(UART0_IRQ).write_volatile(1);
}

pub unsafe fn plic_init_hart() {
    let hart = riscv::cpuid();
    s_mode_enable(hart, UART0_IRQ).write_volatile(1 << (UART0_IRQ % 32));
    s_mode_threshold(hart).write_volatile(0);
}

pub unsafe fn plic_claim() -> u32 {
    let hart = riscv::cpuid();
    s_mode_claim(hart).read_volatile()
}

pub unsafe fn plic_complete(irq: u32) {
    let hart = riscv::cpuid();
    s_mode_claim(hart).write_volatile(irq);
}
