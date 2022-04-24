    .equ REGBYTES, 8
    .macro STORE reg, offset
        sd  \reg, \offset * REGBYTES(sp)
    .endm
    .macro LOAD reg, offset
        ld  \reg, \offset * REGBYTES(sp)
    .endm
    .macro TPSTORE reg, offset
        sd  \reg, \offset * REGBYTES(tp)
    .endm
    .macro TPLOAD reg, offset
        ld  \reg, \offset * REGBYTES(tp)
        sd  zero, \offset * REGBYTES(tp)
    .endm

    .text
    .global trap_entry

trap_entry:
    # TODO: the current implementation cannot handle traps originated from the
    # kernel. We should figure out a better way to handle context switching.

    # save user tp and load kernel tp
    csrrw   tp, sscratch, tp
    bnez    tp, 1f
    # double fault occurred, keep tp and sp unchanged
    csrrw   tp, sscratch, zero
    j       2f
1:
    # switch user sp and kernel sp
    TPSTORE sp, 0
    TPLOAD  sp, 1
2:
    # save context
    addi    sp, sp, -29 * REGBYTES
    STORE   ra, 0
    STORE   t0, 1
    STORE   t1, 2
    STORE   t2, 3
    STORE   t3, 4
    STORE   t4, 5
    STORE   t5, 6
    STORE   t6, 7
    STORE   a0, 8
    STORE   a1, 9
    STORE   a2, 10
    STORE   a3, 11
    STORE   a4, 12
    STORE   a5, 13
    STORE   a6, 14
    STORE   a7, 15

    # Save callee-saved registers
    # Note that only a very few number of syscalls require these registers,
    # including `fork` and `ptrace`. It is best to only save them when the
    # user actually requests these syscalls, but for the sake of simplicity,
    # we'll save all of them here.
    STORE   gp,  16
    STORE   s0,  17
    STORE   s1,  18
    STORE   s2,  19
    STORE   s3,  20
    STORE   s4,  21
    STORE   s5,  22
    STORE   s6,  23
    STORE   s7,  24
    STORE   s8,  25
    STORE   s9,  26
    STORE   s10, 27
    STORE   s11, 28

    # call Rust's trap handler
    mv      a0, sp
    call    trap_handler

    # restore context
    LOAD    ra, 0
    LOAD    t0, 1
    LOAD    t1, 2
    LOAD    t2, 3
    LOAD    t3, 4
    LOAD    t4, 5
    LOAD    t5, 6
    LOAD    t6, 7
    LOAD    a0, 8
    LOAD    a1, 9
    LOAD    a2, 10
    LOAD    a3, 11
    LOAD    a4, 12
    LOAD    a5, 13
    LOAD    a6, 14
    LOAD    a7, 15

    # We don't need to load callee-saved registers, since they are only
    # overwritten by a `ptrace`, which we won't support.
    addi    sp, sp, 29 * REGBYTES

    # switch user sp and kernel sp
    TPSTORE sp, 1
    TPLOAD  sp, 0
    # save kernel tp and load user tp
    csrrw   tp, sscratch, tp
    sret
