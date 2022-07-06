#![allow(dead_code)]

#[cfg(target_arch = "riscv64")]
use crate::arch::riscv64::address;
#[cfg(target_arch = "riscv64")]
use volatile::Volatile;

#[cfg(target_arch = "x86_64")]
use x86_64::instructions::port::Port;

use crate::lazy::Lazy;
use core::fmt::{Error, Write};

pub static mut UART: Lazy<Uart> = Lazy::<Uart, fn() -> Uart>::new(|| unsafe {
    let mut uart = Uart::new();
    uart.init();
    uart
});

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

#[cfg(target_arch = "x86_64")]
const SERIAL_PORT: u16 = 0x3F8;

pub struct Uart {
    #[cfg(not(target_arch = "x86_64"))]
    regs: &'static mut [Volatile<u8>; 8],
    #[cfg(target_arch = "x86_64")]
    ports: [Port<u8>; 8],
}

impl Uart {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "x86_64"))]
            regs: unsafe { &mut *((address::_uart0_start as usize) as *mut [Volatile<u8>; 8]) },
            #[cfg(target_arch = "x86_64")]
            ports: [
                Port::new(SERIAL_PORT + 0),
                Port::new(SERIAL_PORT + 1),
                Port::new(SERIAL_PORT + 2),
                Port::new(SERIAL_PORT + 3),
                Port::new(SERIAL_PORT + 4),
                Port::new(SERIAL_PORT + 5),
                Port::new(SERIAL_PORT + 6),
                Port::new(SERIAL_PORT + 7),
            ],
        }
    }

    unsafe fn write_reg(&mut self, index: usize, value: u8) {
        #[cfg(not(target_arch = "x86_64"))]
        self.regs[index].write(value);
        #[cfg(target_arch = "x86_64")]
        self.ports[index].write(value);
    }

    unsafe fn read_reg(&mut self, index: usize) -> u8 {
        #[cfg(not(target_arch = "x86_64"))]
        return self.regs[index].read();
        #[cfg(target_arch = "x86_64")]
        return self.ports[index].read();
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
