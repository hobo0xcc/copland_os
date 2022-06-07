use crate::arch::riscv64::csr::*;
use crate::arch::riscv64::*;
use crate::device::common::virtio::block;
use crate::device::virt::uart::UART;
use crate::*;
use core::arch::global_asm;
use log::info;

global_asm!(include_str!("kernelvec.S"));

extern "C" {
    pub fn kernel_vec();
}

#[no_mangle]
pub unsafe extern "C" fn kernel_trap() {
    let scause = Csr::Scause.read();
    if scause & (1 << 63) == 0 {
        // exception
        match scause {
            12 | 13 | 15 => {
                println!("Page Fault occurred");
            }
            _ => {
                println!("Exception occurred");
            }
        }

        loop {}
    } else if scause & 0x8000000000000000 != 0 && scause & 0xff == 9 {
        let irq = plic::PLIC_MANAGER.lock().read_claim();
        if irq as usize == plic::PlicIRQ::Uart0 as usize {
            UART.lock().interrupt();
        } else if irq as usize == plic::PlicIRQ::VirtIO0 as usize {
            block::VIRTIO_BLOCK.lock().interrupt();
        } else {
            panic!("Unknown interrupt irq: {}", irq);
        }

        plic::PLIC_MANAGER.lock().send_complete(irq);
    }

    KERNEL_LOCK.complete_intr();
}
