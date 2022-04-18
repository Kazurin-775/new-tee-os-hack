    .text
    .global syscall_entry

syscall_entry:
    # switch from kernel stack to user stack
    xchg    gs:[0x10], rsp

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

    # Note: no need to align `rsp` now, since `rsp` is already aligned at
    # 16 bytes during `syscall_entry`, and we're pushing an even number of
    # items onto the stack.
    #
    # According to https://ropemporium.com/guide.html :
    #
    # > The 64 bit calling convention requires the stack to be 16-byte aligned
    # > before a `call` instruction.

    # jump to Rust code
    call    handle_syscall
    # the return value is stored in rax

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

    # switch from user stack to kernel stack
    xchg    gs:[0x10], rsp

    ret
