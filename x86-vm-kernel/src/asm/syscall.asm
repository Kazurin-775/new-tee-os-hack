    .text
    .global syscall_entry

syscall_entry:
    # save user sp & load kernel sp
    swapgs
    xchg    gs:[0], rsp

    # save clobbered registers
    push    rax
    push    rcx
    push    rdx
    push    rsi
    push    rdi
    push    r8
    push    r9
    push    r10
    push    r11

    # save callee-saved registers
    # for more details, see the Keystone part
    push    rbx
    push    rbp
    push    r12
    push    r13
    push    r14
    push    r15
    pushf

    # XMM registers are not saved (yet)

    # jump to Rust code
    mov     rdi, rsp
    call    handle_syscall
    # the return value is stored in rax

    #; pop callee-saved from the stack without loading them
    #  for more details, see the Keystone part
    add     rsp, 7 * 8

    # restore registers
    pop     r11
    pop     r10
    pop     r9
    pop     r8
    pop     rdi
    pop     rsi
    pop     rdx
    pop     rcx
    pop     rax

    # save kernel sp & load user sp
    xchg    gs:[0], rsp
    swapgs

    # return to userspace
    sysretq
