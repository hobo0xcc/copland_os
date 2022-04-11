#![allow(non_upper_case_globals)]

extern "C" {
    pub fn _text_start();
    pub fn _text_end();
    pub fn _rodata_start();
    pub fn _rodata_end();
    pub fn _data_start();
    pub fn _data_end();
    pub fn _bss_start();
    pub fn _bss_end();
    pub fn _stack_start();
    pub fn _stack_end();
    pub fn _heap_start();
    pub fn _heap_end();
}

pub const _mmio_base: usize = 0x3F000000;
pub const _gpfsel0: usize = _mmio_base + 0x00200000;
pub const _gpfsel1: usize = _mmio_base + 0x00200004;
pub const _gpfsel2: usize = _mmio_base + 0x00200008;
pub const _gpfsel3: usize = _mmio_base + 0x0020000c;
pub const _gpfsel4: usize = _mmio_base + 0x00200010;
pub const _gpfsel5: usize = _mmio_base + 0x00200014;
pub const _gpset0: usize = _mmio_base + 0x0020001c;
pub const _gpset1: usize = _mmio_base + 0x00200020;
pub const _gpclr0: usize = _mmio_base + 0x00200028;
pub const _gplev0: usize = _mmio_base + 0x00200034;
pub const _gplev1: usize = _mmio_base + 0x00200038;
pub const _gpeds0: usize = _mmio_base + 0x00200040;
pub const _gpeds1: usize = _mmio_base + 0x00200044;
pub const _gphen0: usize = _mmio_base + 0x00200064;
pub const _gphen1: usize = _mmio_base + 0x00200068;
pub const _gppud: usize = _mmio_base + 0x00200094;
pub const _gppudclk0: usize = _mmio_base + 0x00200098;
pub const _gppudclk1: usize = _mmio_base + 0x0020009c;
