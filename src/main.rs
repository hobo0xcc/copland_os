#![feature(panic_info_message, start)]
#![no_std]
#![no_main]
#![allow(unused_imports)]

extern crate alloc;
extern crate copland_os;

use copland_os::kernlock::KernelLock;
use copland_os::*;
use core::arch::asm;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref KERNEL_LOCK: KernelLock = KernelLock::new();
}

#[no_mangle]
#[cfg(target_arch = "riscv64")]
pub unsafe extern "C" fn main() -> ! {
    use copland_os::arch::riscv64::*;

    KERNEL_LOCK.lock();

    allocator::init_allocator();
    println!("RISC-V");
    println!("PRESENT DAY\n  PRESENT TIME");

    println!("hart: {}", riscv::STATE.lock().cpuid());

    let root_ptes = vm::VM_MANAGER.lock().map_mem_identically_as_phy();
    vm::VM_MANAGER.lock().enable_paging(root_ptes);

    println!("Hello, world!");

    loop {}
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn main() -> ! {
    use copland_os::arch::aarch64::*;

    KERNEL_LOCK.lock();

    allocator::init_allocator();
    println!("AArch64");
    println!("PRESENT DAY\n  PRESENT TIME");

    println!("hart: {}", arm::STATE.lock().cpuid());

    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn boot() -> ! {
    #[cfg(target_arch = "riscv64")]
    asm!(include_str!("arch/riscv64/boot.S"));
    #[cfg(target_arch = "aarch64")]
    asm!(include_str!("arch/aarch64/boot.S"));
    loop {}
}

#[no_mangle]
#[start]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _entry() -> ! {
    #[cfg(target_arch = "riscv64")]
    asm!("j boot");
    #[cfg(target_arch = "aarch64")]
    asm!("b boot");
    loop {}
}

#[panic_handler]
#[cfg(target_arch = "riscv64")]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Panic: ");
    if let Some(location) = info.location() {
        println!(
            "line: {}, file: {}: {}",
            location.line(),
            location.file(),
            info.message().unwrap()
        );
    } else {
        println!("No information available");
    }
    loop {}
}

#[panic_handler]
#[cfg(target_arch = "aarch64")]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
