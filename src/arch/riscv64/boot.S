    mv t0, a0
    la sp, _stack_start
    li a0, 0x4000
    csrr a1, mhartid
    addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0
    mv a0, t0
    call start
