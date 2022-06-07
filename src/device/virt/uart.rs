#![allow(dead_code)]

use crate::arch::riscv64::address;
use crate::lock::Mutex;
use core::fmt::{Error, Write};
use lazy_static::lazy_static;
use volatile::Volatile;

lazy_static! {
    pub static ref UART: Mutex<Uart> = unsafe {
        let mut uart = Uart::new();
        uart.init();
        Mutex::new(uart)
    };
}

// https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#UART_Registers

const THR: usize = 0;
const RBR: usize = 0;
const DLL: usize = 0;
const IER: usize = 1;
const DLH: usize = 1;
const IIR: usize = 2;
const FCR: usize = 2;
const LCR: usize = 3;
const MCR: usize = 4;
const LSR: usize = 5;
const MSR: usize = 6;
const SR: usize = 7;

pub struct Uart {
    regs: &'static mut [Volatile<u8>; 8],
}

impl Uart {
    pub fn new() -> Self {
        Self {
            regs: unsafe { &mut *((address::_uart0_start as usize) as *mut [Volatile<u8>; 8]) },
        }
    }

    unsafe fn write_reg(&mut self, index: usize, value: u8) {
        self.regs[index].write(value)
    }

    unsafe fn read_reg(&self, index: usize) -> u8 {
        self.regs[index].read()
    }

    pub unsafe fn init(&mut self) {
        self.write_reg(IER, 0);
        self.write_reg(LCR, 0b1 << 7);
        self.write_reg(DLL, 0x01);
        self.write_reg(DLH, 0x00);
        self.write_reg(LCR, 0b11);
        self.write_reg(FCR, 0b111);
        self.write_reg(IER, 0b11);
    }

    pub unsafe fn putc(&mut self, c: u8) {
        while (self.read_reg(LSR) & 0b1 << 5) == 0 {}
        self.write_reg(THR, c);
    }

    pub unsafe fn getc(&mut self) -> Option<u8> {
        if self.read_reg(LSR) & 1 != 0 {
            Some(self.read_reg(RBR))
        } else {
            None
        }
    }

    pub unsafe fn interrupt(&mut self) {
        loop {
            if let Some(_c) = self.getc() {
                // do something
            } else {
                break;
            }
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.chars() {
            unsafe {
                self.putc(c as u8);
            }
        }

        Ok(())
    }
}
