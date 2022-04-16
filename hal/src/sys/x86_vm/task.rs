use x86_64::registers::model_specific as msr;

#[repr(C)]
#[derive(Default)]
pub struct KtaskTls {
    /// The "foreign" stack pointer (i.e. user sp in kernel context, and
    /// kernel sp in user context).
    ///
    /// This is read and written using a `xchg gs:[0], rsp` instruction.
    pub foreign_sp: usize,

    // used by assembly code, should not be touched by Rust code
    prev_kctx: usize,
    cur_kctx: usize,

    // the following fields are used by Rust code
    pub pcb_weak_ptr: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct KtaskCtx {
    pub rsp: usize,
    pub rbp: usize,
    pub rbx: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub gs_offset: usize,
}

const KERNEL_STACK_LAYOUT: alloc::alloc::Layout =
    unsafe { alloc::alloc::Layout::from_size_align_unchecked(0x4000, 0x1000) };

impl KtaskTls {
    pub fn from_user_sp(user_sp: usize) -> KtaskTls {
        KtaskTls {
            foreign_sp: user_sp,
            ..Default::default()
        }
    }

    pub fn set_pcb_weak_ptr(&mut self, new: usize) {
        self.pcb_weak_ptr = new;
    }
}

impl KtaskCtx {
    pub fn allocate_for(thread_ctx: *const KtaskTls) -> KtaskCtx {
        let stack = unsafe { alloc::alloc::alloc(KERNEL_STACK_LAYOUT) };
        // write the task's entry address at the bottom of the stack
        unsafe {
            (stack.offset(0x3FF8) as *mut u64).write(ret_from_fork as u64);
        }
        KtaskCtx {
            rsp: (stack as usize) + 0x3FF8,
            gs_offset: thread_ctx as usize,
            ..Default::default()
        }
    }
}

extern "C" {
    // functions defined in `task.asm`
    pub fn ktask_enter(from: *mut KtaskCtx, to: *mut KtaskCtx);
    pub fn ktask_leave();
    // functions defined in `x86-vm-kernel`
    fn ret_from_fork() -> !;
}

core::arch::global_asm!(include_str!("task.asm"));

pub fn current_pcb_weak() -> usize {
    // IA32_KERNEL_GS_BASE must be 0 in a kernel context
    assert_eq!(msr::KernelGsBase::read().as_u64(), 0);
    // the opposite for IA32_GS_BASE
    let tls = msr::GsBase::read().as_u64();
    assert_ne!(tls, 0);

    let tls = tls as *const KtaskTls;
    unsafe { (*tls).pcb_weak_ptr }
}
