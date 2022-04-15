    .text
    .global syscall_entry

syscall_entry:
    # save user sp & load kernel sp
    swapgs
    xchg    gs:[0], rsp

    # save clobbered registers
    # // push    rax
    push    rcx
    push    rdx
    push    rsi
    push    rdi
    push    r8
    push    r9
    push    r10
    push    r11

    # construct C ABI arguments
    mov     rcx, rax
    mov     r8, r10

    # align `rsp` to 16 bytes boundary to avoid potential errors
    # For the reason of this, see `sgx-libos/src/asm/syscall.asm`.
    sub     rsp, 8

    # jump to Rust code
    call    handle_syscall
    # the return value is stored in rax

    # restore rsp's value before alignment
    add     rsp, 8

    # restore registers
    pop     r11
    pop     r10
    pop     r9
    pop     r8
    pop     rdi
    pop     rsi
    pop     rdx
    pop     rcx
    # // pop     rax

    # save kernel sp & load user sp
    xchg    gs:[0], rsp
    swapgs

    # return to userspace
    sysretq
