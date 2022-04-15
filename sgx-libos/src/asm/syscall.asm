    .text
    .global syscall_entry

syscall_entry:
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

    # IMPORTANT: align `rsp` to 16 bytes boundary
    # For unknown reason, the Linux ABI enforces this, or some XMM instructions
    # with `rsp` in its operands will cause a segmentation fault.
    # Since we'll push an odd number of items in the stack, we just subtract 8
    # from `rsp`.
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

    ret
