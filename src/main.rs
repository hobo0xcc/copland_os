#![feature(panic_info_message, start)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate copland_os;

use copland_os::kernlock::KernelLock;
use copland_os::*;
use core::arch::asm;
use lazy_static::lazy_static;
use log::info;

lazy_static! {
    pub static ref KERNEL_LOCK: KernelLock = KernelLock::new();
}

#[no_mangle]
#[cfg(target_arch = "riscv64")]
pub unsafe extern "C" fn main() -> ! {
    KERNEL_LOCK.lock();

    allocator::init_allocator();
    logger::init_logger();

    info!("Arch: RISC-V");
    info!(
        "Hart: {}",
        crate::arch::riscv64::riscv::STATE.lock().cpuid()
    );

    {
        use copland_os::arch::riscv64::*;
        vm::VM_MANAGER.lock().init();
    }

    task::TASK_MANAGER.lock().init();

    let id = task::TASK_MANAGER.lock().create_task("test");
    task::TASK_MANAGER.lock().ready_task(id);
    task::TASK_MANAGER.lock().schedule();

    println!("PRESENT DAY\n  PRESENT TIME");

    loop {}
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn main() -> ! {
    KERNEL_LOCK.lock();

    allocator::init_allocator();
    logger::init_logger();

    info!("Arch: AArch64");
    info!("Hart: {}", crate::arch::aarch64::arm::STATE.lock().cpuid());

    {
        use copland_os::arch::aarch64::*;
        vm::VM_MANAGER.lock().init();
    }

    task::TASK_MANAGER.lock().init();

    let id = task::TASK_MANAGER.lock().create_task("test");
    task::TASK_MANAGER.lock().ready_task(id);
    task::TASK_MANAGER.lock().schedule();

    println!("PRESENT DAY\n  PRESENT TIME");

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
