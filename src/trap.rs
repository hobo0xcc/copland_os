use crate::csr::*;
use crate::uart::Uart;
use crate::*;
use core::arch::global_asm;

global_asm!(include_str!("kernelvec.S"));

extern "C" {
    pub fn kernel_vec();
}

#[no_mangle]
pub unsafe extern "C" fn kernel_trap() {
    if Csr::Scause.read() & 0x8000000000000000 != 0 && Csr::Scause.read() & 0xff == 9 {
        let irq = plic::plic_claim();
        if irq as usize == plic::UART0_IRQ {
            Uart::new().interrupt();
        } else {
            panic!("Unknown interrupt irq: {}", irq);
        }

        plic::plic_complete(irq);
    }
}
