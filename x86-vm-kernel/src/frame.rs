use hal::{arch::x86_vm::gdt, task::UserspaceRegs};

#[repr(C)]
pub struct SyscallFrame {
    // the fields are stored in reversed order
    pub rflags: usize,
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

impl SyscallFrame {
    pub fn to_child_regs(&self) -> UserspaceRegs {
        UserspaceRegs {
            rax: 0, // fork() returns 0 to a child process
            rbx: self.rbx,
            rcx: self.rcx,
            rdx: self.rdx,
            rsi: self.rsi,
            rdi: self.rdi,
            rbp: self.rbp,
            r8: self.r8,
            r9: self.r9,
            r10: self.r10,
            r11: self.r11,
            r12: self.r12,
            r13: self.r13,
            r14: self.r14,
            r15: self.r15,
            rip: self.rcx, // `syscall` convention
            cs: gdt::USER_CODE_SEL.0 as usize,
            rflags: self.rflags,
            rsp: hal::task::current().lock().tls.foreign_sp,
            ss: gdt::USER_DATA_SEL.0 as usize,
            // TODO: process user GS
        }
    }
}
