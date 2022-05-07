use core::arch::asm;

extern "C" {
    pub fn main();
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn start() {
    asm!(
        "
        mrs x0, midr_el1
        msr vpidr_el2, x0
        mrs x0, mpidr_el1
        msr vmpidr_el2, x0
        // msr vttbr_el2, xzr
        msr sctlr_el2, xzr
        msr sctlr_el1, xzr
        ldr x0, =(1 << 31)
        msr hcr_el2, x0

        mov x0, #0x4b1
        msr scr_el3, x0
        adr x0, main
        msr elr_el3, x0
        mov x0, #0x5
        msr spsr_el3, x0
        mov x0, sp
        msr sp_el1, x0
        eret
    "
    );
    loop {}
}
