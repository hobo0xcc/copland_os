#![allow(dead_code)]

use crate::address::_uart0_start;
use core::fmt::{Error, Write};
use volatile_register::RW;

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
    reg: *mut [RW<u8>; 8],
}

impl Uart {
    pub fn new() -> Self {
        Self {
            reg: (_uart0_start as usize) as *mut [RW<u8>; 8],
        }
    }

    unsafe fn write_reg(&mut self, index: usize, value: u8) {
        (*self.reg)[index].write(value)
    }

    unsafe fn read_reg(&self, index: usize) -> u8 {
        (*self.reg)[index].read()
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
            if let Some(c) = self.getc() {
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
