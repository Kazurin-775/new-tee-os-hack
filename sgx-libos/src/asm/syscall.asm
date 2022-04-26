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

    # IMPORTANT: align `rsp` to 16 bytes boundary
    #
    # According to https://ropemporium.com/guide.html :
    #
    # > The 64 bit calling convention requires the stack to be 16-byte aligned
    # > before a `call` instruction.
    #
    # Since we'll push an odd number of items in the stack, we just subtract 8
    # from `rsp`.
    sub     rsp, 8

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

    # // pop r9 and the alignment from the stack
    add     rsp, 16

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
