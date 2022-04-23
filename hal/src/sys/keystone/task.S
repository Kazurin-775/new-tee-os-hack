    .equ REGBYTES, 8
    .macro STORE reg, offset
        sd  \reg, \offset * REGBYTES(a0)
    .endm
    .macro LOAD reg, offset
        ld  \reg, \offset * REGBYTES(a1)
        sd  zero, \offset * REGBYTES(a1)
    .endm
    .macro TPSTORE reg, offset
        sd  \reg, \offset * REGBYTES(tp)
    .endm
    .macro TPLOAD reg, offset
        ld  \reg, \offset * REGBYTES(tp)
        sd  zero, \offset * REGBYTES(tp)
    .endm
    .macro SPLOAD reg, offset
        ld  \reg, \offset * REGBYTES(sp)
    .endm

    .text
    .global ktask_enter
    .global ktask_leave
    .global ret_from_fork

ktask_enter:
    STORE   sp,   0
    STORE   ra,   1
    STORE   tp,   2
    STORE   s0,   3
    STORE   s1,   4
    STORE   s2,   5
    STORE   s3,   6
    STORE   s4,   7
    STORE   s5,   8
    STORE   s6,   9
    STORE   s7,  10
    STORE   s8,  11
    STORE   s9,  12
    STORE   s10, 13
    STORE   s11, 14

    LOAD    sp,   0
    LOAD    ra,   1
    LOAD    tp,   2
    LOAD    s0,   3
    LOAD    s1,   4
    LOAD    s2,   5
    LOAD    s3,   6
    LOAD    s4,   7
    LOAD    s5,   8
    LOAD    s6,   9
    LOAD    s7,  10
    LOAD    s8,  11
    LOAD    s9,  12
    LOAD    s10, 13
    LOAD    s11, 14

    ## if we're in a Task, save the context pointers to tp
    beqz    tp, 1f
    TPSTORE a0, 2
    TPSTORE a1, 3

1:
    ret

ktask_leave:
    # load the context pointers from tp
    TPLOAD  a0, 3
    TPLOAD  a1, 2
    # switch back to the scheduler
    j       ktask_enter

ret_from_fork:
    # write kernel tp to sscratch
    csrw    sscratch, tp

    # save kernel sp after UserspaceRegs load
    addi    t0, sp, 32 * REGBYTES
    TPSTORE t0, 1

    # load CSRs from UserspaceRegs
    SPLOAD  t0,  31
    csrw    sepc, t0

    # load all registers from UserspaceRegs
    SPLOAD  ra,   0
    SPLOAD  gp,   2
    SPLOAD  tp,   3
    SPLOAD  t0,   4
    SPLOAD  t1,   5
    SPLOAD  t2,   6
    SPLOAD  s0,   7
    SPLOAD  s1,   8
    SPLOAD  a0,   9
    SPLOAD  a1,  10
    SPLOAD  a2,  11
    SPLOAD  a3,  12
    SPLOAD  a4,  13
    SPLOAD  a5,  14
    SPLOAD  a6,  15
    SPLOAD  a7,  16
    SPLOAD  s2,  17
    SPLOAD  s3,  18
    SPLOAD  s4,  19
    SPLOAD  s5,  20
    SPLOAD  s6,  21
    SPLOAD  s7,  22
    SPLOAD  s8,  23
    SPLOAD  s9,  24
    SPLOAD  s10, 25
    SPLOAD  s11, 26
    SPLOAD  t3,  27
    SPLOAD  t4,  28
    SPLOAD  t5,  29
    SPLOAD  t6,  30
    # finally, load sp
    SPLOAD  sp,   1

    # jump to user main
    sret
