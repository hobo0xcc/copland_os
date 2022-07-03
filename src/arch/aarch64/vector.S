.macro SWITCH_HANDLER handler type
    sub	sp,  sp,  #16 * 18

    stp	x0,  x1,  [sp, #16 * 0]
    stp	x2,  x3,  [sp, #16 * 1]
    stp	x4,  x5,  [sp, #16 * 2]
    stp	x6,  x7,  [sp, #16 * 3]
    stp	x8,  x9,  [sp, #16 * 4]
    stp	x10, x11, [sp, #16 * 5]
    stp	x12, x13, [sp, #16 * 6]
    stp	x14, x15, [sp, #16 * 7]
    stp	x16, x17, [sp, #16 * 8]
    stp	x18, x19, [sp, #16 * 9]
    stp	x20, x21, [sp, #16 * 10]
    stp	x22, x23, [sp, #16 * 11]
    stp	x24, x25, [sp, #16 * 12]
    stp	x26, x27, [sp, #16 * 13]
    stp	x28, x29, [sp, #16 * 14]

    mov     x0, #\type
    mrs     x1, esr_el1
    mrs     x2, elr_el1
    mrs     x3, spsr_el1
    mrs     x4, far_el1

    stp lr, x0, [sp, #16 * 15]
    stp x1, x2, [sp, #16 * 16]
    stp x3, x4, [sp, #16 * 17]

    bl \handler

    b restore_and_return
.endm

restore_and_return:
    ldp lr, x0, [sp, #16 * 15]
    ldp x1, x2, [sp, #16 * 16]
    ldp x3, x4, [sp, #16 * 17]

    msr     esr_el1, x1
    msr     elr_el1, x2
    msr     spsr_el1, x3
    msr     far_el1, x4

    ldp	x0,  x1,  [sp, #16 * 0]
    ldp	x2,  x3,  [sp, #16 * 1]
    ldp	x4,  x5,  [sp, #16 * 2]
    ldp	x6,  x7,  [sp, #16 * 3]
    ldp	x8,  x9,  [sp, #16 * 4]
    ldp	x10, x11, [sp, #16 * 5]
    ldp	x12, x13, [sp, #16 * 6]
    ldp	x14, x15, [sp, #16 * 7]
    ldp	x16, x17, [sp, #16 * 8]
    ldp	x18, x19, [sp, #16 * 9]
    ldp	x20, x21, [sp, #16 * 10]
    ldp	x22, x23, [sp, #16 * 11]
    ldp	x24, x25, [sp, #16 * 12]
    ldp	x26, x27, [sp, #16 * 13]
    ldp	x28, x29, [sp, #16 * 14]

    add sp, sp, #16 * 17

    eret

.globl vector
.balign 0x800
vector:
curr_el_sp0_sync:        // The exception handler for a synchronous 
                         // exception from the current EL using SP0.
    SWITCH_HANDLER kernel_exception, 0
.balign 0x80
curr_el_sp0_irq:         // The exception handler for an IRQ exception
                         // from the current EL using SP0.
    SWITCH_HANDLER kernel_exception, 1
.balign 0x80
curr_el_sp0_fiq:         // The exception handler for an FIQ exception
                         // from the current EL using SP0.
    SWITCH_HANDLER kernel_exception, 2
.balign 0x80
curr_el_sp0_serror:      // The exception handler for a System Error 
                         // exception from the current EL using SP0.
    SWITCH_HANDLER kernel_exception, 3
.balign 0x80
curr_el_spx_sync:        // The exception handler for a synchrous 
                         // exception from the current EL using the
                         // current SP.
    SWITCH_HANDLER kernel_exception, 0
.balign 0x80
curr_el_spx_irq:         // The exception handler for an IRQ exception from 
                         // the current EL using the current SP.
    SWITCH_HANDLER kernel_exception, 1

.balign 0x80
curr_el_spx_fiq:         // The exception handler for an FIQ from 
                         // the current EL using the current SP.
    SWITCH_HANDLER kernel_exception, 2

.balign 0x80
curr_el_spx_serror:      // The exception handler for a System Error 
                         // exception from the current EL using the
                         // current SP.
    SWITCH_HANDLER kernel_exception, 3

.balign 0x80
lower_el_aarch64_sync:   // The exception handler for a synchronous 
                         // exception from a lower EL (AArch64).

.balign 0x80
lower_el_aarch64_irq:    // The exception handler for an IRQ from a lower EL
                         // (AArch64).

.balign 0x80
lower_el_aarch64_fiq:    // The exception handler for an FIQ from a lower EL
                         // (AArch64).

.balign 0x80
lower_el_aarch64_serror: // The exception handler for a System Error 
                         // exception from a lower EL(AArch64).

.balign 0x80
lower_el_aarch32_sync:   // The exception handler for a synchronous 
                         // exception from a lower EL(AArch32).
.balign 0x80
lower_el_aarch32_irq:    // The exception handler for an IRQ exception 
                         // from a lower EL (AArch32).
.balign 0x80
lower_el_aarch32_fiq:    // The exception handler for an FIQ exception from 
                         // a lower EL (AArch32).
.balign 0x80
lower_el_aarch32_serror: // The exception handler for a System Error
                         // exception from a lower EL(AArch32).
