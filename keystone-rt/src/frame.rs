use hal::task::UserspaceRegs;

#[repr(C)]
pub struct TrapFrame {
    pub ra: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,

    // Callee-saved registers
    // Note that sp and tp is not included
    pub gp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

impl TrapFrame {
    pub fn to_child_regs(&self) -> UserspaceRegs {
        UserspaceRegs {
            ra: self.ra,
            sp: hal::task::current().lock().tls.user_sp,
            gp: self.gp,
            tp: riscv::register::sscratch::read(),
            t0: self.t0,
            t1: self.t1,
            t2: self.t2,
            s0: self.s0,
            s1: self.s1,
            a0: 0, // fork() returns 0 to a child process
            a1: self.a1,
            a2: self.a2,
            a3: self.a3,
            a4: self.a4,
            a5: self.a5,
            a6: self.a6,
            a7: self.a7,
            s2: self.s2,
            s3: self.s3,
            s4: self.s4,
            s5: self.s5,
            s6: self.s6,
            s7: self.s7,
            s8: self.s8,
            s9: self.s9,
            s10: self.s10,
            s11: self.s11,
            t3: self.t3,
            t4: self.t4,
            t5: self.t5,
            t6: self.t6,
            sepc: riscv::register::sepc::read() + 4, // skip `ecall`
        }
    }
}
