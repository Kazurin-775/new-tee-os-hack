use kconfig::{KERNEL_STACK_SIZE, PAGE_SIZE};
use x86_64::registers::model_specific as msr;

pub use crate::arch::x86_vm::frame::UserspaceRegs;

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
    unsafe { alloc::alloc::Layout::from_size_align_unchecked(KERNEL_STACK_SIZE, PAGE_SIZE) };

impl KtaskTls {
    pub fn new() -> KtaskTls {
        KtaskTls {
            ..Default::default()
        }
    }

    pub fn set_pcb_weak_ptr(&mut self, new: usize) {
        self.pcb_weak_ptr = new;
    }
}

impl KtaskCtx {
    pub fn allocate_for(thread_ctx: *const KtaskTls, userspace_regs: &[u8]) -> KtaskCtx {
        let kernel_stack = unsafe { alloc::alloc::alloc(KERNEL_STACK_LAYOUT) };
        // copy userspace regs & the address of `ret_from_fork` to the end of kernel stack
        let len_to_write = core::mem::size_of::<usize>() + userspace_regs.len();
        assert!(len_to_write <= KERNEL_STACK_SIZE);
        let kernel_sp;
        unsafe {
            kernel_sp = kernel_stack.add(KERNEL_STACK_SIZE - len_to_write);
            (kernel_sp as *mut usize).write(ret_from_fork as usize);
            core::slice::from_raw_parts_mut(
                kernel_sp.add(core::mem::size_of::<usize>()),
                userspace_regs.len(),
            )
            .copy_from_slice(userspace_regs);
        }

        KtaskCtx {
            rsp: kernel_sp as usize,
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
