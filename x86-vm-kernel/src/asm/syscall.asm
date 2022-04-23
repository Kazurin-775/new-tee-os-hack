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
    # From: (rdi rsi rdx rax r10 r8  r9   )
    # To:   (rdi rsi rdx rcx r8  r9  stack)
    mov     rcx, rax
    push    r9
    mov     r9, r8
    mov     r8, r10

    # jump to Rust code
    call    handle_syscall
    # the return value is stored in rax

    # // pop r9 from the stack
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
