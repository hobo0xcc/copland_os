1:
    mv t0, a0
    la sp, _stack_start
    li a0, 0x4000
    csrr a1, mhartid
    addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0
2:
    la a1, _bss_start
    la a2, _bss_end
    sub a2, a2, a1
    li a3, 3
    srl a2, a2, a3
    li a3, 0
3:
    beq a2, a3, 4f
    sd a3, 0(a1)
    addi a1, a1, 8
    addi a2, a2, -1
    bne a2, a3, 3b
4:
    mv a0, t0
    call start
    j 1b
