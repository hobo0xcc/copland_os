#![feature(panic_info_message, start)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate copland_os;

use copland_os::*;
use core::arch::asm;

#[cfg(target_arch = "riscv64")]
pub unsafe extern "C" fn main() -> ! {
    use copland_os::arch::riscv64::*;
    if riscv::cpuid() != 0 {
        loop {}
    }
    allocator::init_allocator();
    plic::plic_init();
    plic::plic_init_hart();
    let mut uart = device::virt::uart::Uart::new();
    uart.init();
    println!("PRESENT DAY\n  PRESENT TIME");

    loop {}
}

#[cfg(target_arch = "riscv64")]
unsafe extern "C" fn pmp_init() {
    let pmpaddr0 = (!0_usize) >> 10;
    asm!("csrw pmpaddr0, {}", in(reg)pmpaddr0);
    let mut pmpcfg0 = 0_usize;
    pmpcfg0 |= 3 << 3;
    pmpcfg0 |= 1 << 2;
    pmpcfg0 |= 1 << 1;
    pmpcfg0 |= 1 << 0;
    asm!("csrw pmpcfg0, {}", in(reg)pmpcfg0);
}

#[no_mangle]
#[cfg(target_arch = "riscv64")]
pub unsafe extern "C" fn start() -> ! {
    use copland_os::arch::riscv64::csr::*;
    let mut mstatus = Csr::Mstatus.read();
    mstatus &= !Mstatus::MPP.mask();
    mstatus |= 0b01_usize << Mstatus::MPP.index(); // 0b01 -> Supervisor Mode

    mstatus |= 0b1 << Mstatus::SPIE.index(); // SPIE
    mstatus |= 0b1 << Mstatus::MPIE.index(); // MPIE
    mstatus |= 0b1 << Mstatus::SIE.index(); // SIE
    mstatus |= 0b1 << Mstatus::MIE.index(); // MIE
    mstatus |= 0b01 << Mstatus::FS.index(); // FS
    Csr::Mstatus.write(mstatus);

    let mut sstatus = Csr::Sstatus.read();
    sstatus |= 0b1 << Sstatus::SPIE.index(); // SPIE
    sstatus |= 0b1 << Sstatus::SIE.index(); // SIE
    sstatus |= 0b01 << Sstatus::FS.index(); // FS
    Csr::Sstatus.write(sstatus);

    let mepc = main as usize;
    Csr::Mepc.write(mepc);

    pmp_init();

    // disable paging
    asm!("csrw satp, zero");

    // delegate all interrupts and exceptions
    asm!("li t0, 0xffff");
    asm!("csrw mideleg, t0");
    asm!("li t0, 0xffff");
    asm!("csrw medeleg, t0");

    let mut mie = Csr::Mie.read();
    mie |= 0b1 << Mie::MTIE.index();
    Csr::Mie.write(mie);

    let mut sie = Csr::Sie.read();
    sie |= 0b1 << Sie::SEIE.index();
    Csr::Sie.write(sie);

    asm!("csrr tp, mhartid");

    Csr::Stvec.write(arch::riscv64::trap::kernel_vec as usize);

    asm!("mret");

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
