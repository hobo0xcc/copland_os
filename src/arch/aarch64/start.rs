use core::arch::asm;

extern "C" {
    pub fn main();
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn start() {
    asm!(
        "
        mov x0, #0x4b1
        msr scr_el3, x0
        adr x0, main
        msr elr_el3, x0
        mov x0, #0x5
        msr spsr_el3, x0
        mov x0, sp
        msr sp_el1, x0

        mov x0, 0
        msr sctlr_el1, x0
        ldr x0, =(1 << 31)
        msr hcr_el2, x0
        eret
    "
    );

    /*
    #define MODE_AARCH64_EL2H 0x9
    mov   x0, #0x4b1    // RW=1, SMD=1, RES=1, NS=1
    msr   scr_el3, x0
    adr   x0, start_el2
    msr   elr_el3, x0
    mov   x0, #MODE_AARCH64_EL2H
    msr   spsr_el3, x0
    eret
    */
    loop {}
}
