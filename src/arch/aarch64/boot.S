1:
    mrs x1, mpidr_el1
    and x1, x1, #3
    cbz x1, 2f
    wfe
    b 1b
2:
    ldr x1, =_stack_end
    mov sp, x1
    ldr x1, =_bss_start
    ldr x2, =_bss_end
    sub x2, x2, x1
    lsr x2, x2, #3
    mov x3, #0
3:  cbz x2, 4f
    str x3, [x1]
    add x1, x1, #8
    sub x2, x2, #1
    cbnz x2, 3b
4:
    bl start
    b 1b