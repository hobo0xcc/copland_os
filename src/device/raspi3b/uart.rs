#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::arch::aarch64::address::*;
use crate::lock::Mutex;
use core::arch::asm;
use core::fmt::{Error, Write};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref UART: Mutex<MiniUart> = unsafe {
        let uart = MiniUart::new();
        uart.init();
        Mutex::new(uart)
    };
}

// https://github.com/bztsrc/raspi3-tutorial/blob/master/03_uart1/uart.c

const AUX_ENABLE: usize = 0x00215004;
const AUX_MU_IO: usize = 0x00215040;
const AUX_MU_IER: usize = 0x00215044;
const AUX_MU_IIR: usize = 0x00215048;
const AUX_MU_LCR: usize = 0x0021504C;
const AUX_MU_MCR: usize = 0x00215050;
const AUX_MU_LSR: usize = 0x00215054;
const AUX_MU_MSR: usize = 0x00215058;
const AUX_MU_SCRATCH: usize = 0x0021505C;
const AUX_MU_CNTL: usize = 0x00215060;
const AUX_MU_STAT: usize = 0x00215064;
const AUX_MU_BAUD: usize = 0x00215068;

pub struct MiniUart {}

impl MiniUart {
    pub fn new() -> Self {
        Self {}
    }

    unsafe fn write_reg(&self, offset: usize, value: u32) {
        ((_mmio_base + offset) as *mut u32).write_volatile(value);
    }

    unsafe fn read_reg(&self, offset: usize) -> u32 {
        ((_mmio_base + offset) as *mut u32).read_volatile()
    }

    pub unsafe fn init(&self) {
        self.write_reg(AUX_ENABLE, self.read_reg(AUX_ENABLE) | 1);
        self.write_reg(AUX_MU_CNTL, 0);
        self.write_reg(AUX_MU_LCR, 3);
        self.write_reg(AUX_MU_MCR, 0);
        self.write_reg(AUX_MU_IER, 0);
        self.write_reg(AUX_MU_IIR, 0xc6);
        self.write_reg(AUX_MU_BAUD, 270);

        let mut r = (_gpfsel1 as *mut u32).read_volatile();
        r &= !((7 << 12) | (7 << 15));
        r |= (2 << 12) | (2 << 15);
        (_gpfsel1 as *mut u32).write_volatile(r);
        (_gppud as *mut u32).write_volatile(0);

        r = 150;
        while r != 0 {
            r -= 1;
            asm!("nop");
        }

        (_gppudclk0 as *mut u32).write_volatile((1 << 14) | (1 << 15));

        r = 150;
        while r != 0 {
            r -= 1;
            asm!("nop");
        }

        (_gppudclk0 as *mut u32).write_volatile(0);
        self.write_reg(AUX_MU_CNTL, 3);
    }

    pub unsafe fn putc(&self, c: u8) {
        while (self.read_reg(AUX_MU_LSR) & 0b1 << 5) == 0 {}

        self.write_reg(AUX_MU_IO, c as u32);
    }
}

impl Write for MiniUart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.chars() {
            unsafe {
                self.putc(c as u8);
            }
        }

        Ok(())
    }
}
