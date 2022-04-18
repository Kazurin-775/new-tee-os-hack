    .text
    .global ktask_enter_asm
    .global ktask_leave

ktask_enter_asm:
    # save context
    mov     [rdi + 0x00], rsp
    mov     [rdi + 0x08], rbp
    mov     [rdi + 0x10], rbx
    mov     [rdi + 0x18], r12
    mov     [rdi + 0x20], r13
    mov     [rdi + 0x28], r14
    mov     [rdi + 0x30], r15

    # load context
    mov     rsp, [rsi + 0x00]
    mov     rbp, [rsi + 0x08]
    mov     rbx, [rsi + 0x10]
    mov     r12, [rsi + 0x18]
    mov     r13, [rsi + 0x20]
    mov     r14, [rsi + 0x28]
    mov     r15, [rsi + 0x30]

    # check if GS base is non-0 (i.e. if we are in a Ktask)
    cmp     qword ptr [rsi + 0x38], 0
    jz      no_task_ctx

has_task_ctx:
    # // if we are in a Ktask, save the context pointers to GS
    mov     gs:[0x18], rdi
    mov     gs:[0x20], rsi

no_task_ctx:
    ret

ktask_leave:
    # load the context pointers from GS
    mov     rdi, gs:[0x20]
    mov     rsi, gs:[0x18]

    jmp     ktask_enter

ret_from_fork:
    # save kernel stack & load user stack
    xchg    gs:[0x10], rsp

    # return to user!
    jmp     rbx     # rip
