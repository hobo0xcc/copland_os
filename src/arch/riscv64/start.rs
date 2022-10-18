use core::arch::asm;

extern "C" {
    pub fn main();
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
    use crate::arch::riscv64::csr::*;
    use crate::*;

    let mut mstatus = Csr::Mstatus.read();
    mstatus &= !Mstatus::MPP.mask();
    mstatus |= 0b01_usize << Mstatus::MPP.index(); // 0b01 -> Supervisor Mode

    mstatus |= 0b1 << Mstatus::SPIE.index();
    mstatus |= 0b1 << Mstatus::MPIE.index();
    mstatus |= 0b1 << Mstatus::SIE.index();
    mstatus |= 0b1 << Mstatus::MIE.index();
    mstatus |= 0b01 << Mstatus::FS.index();
    Csr::Mstatus.write(mstatus);

    let mut sstatus = Csr::Sstatus.read();
    sstatus |= 0b1 << Sstatus::SPIE.index();
    sstatus |= 0b1 << Sstatus::SIE.index();
    sstatus |= 0b01 << Sstatus::FS.index();
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

    // let mut mie = Csr::Mie.read();
    // mie |= Mie::MTIE.mask();
    // Csr::Mie.write(mie);

    let mut sie = Csr::Sie.read();
    sie |= Sie::SEIE.mask();
    sie |= Sie::STIE.mask();
    sie |= Sie::SSIE.mask();
    Csr::Sie.write(sie);

    asm!("csrr tp, mhartid");

    Csr::Stvec.write(arch::riscv64::trap::kernel_vec as usize);

    asm!("mret");

    loop {}
}
