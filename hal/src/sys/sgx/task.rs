pub use crate::arch::sgx::frame::UserspaceRegs;

#[repr(C)]
#[derive(Default)]
pub struct KtaskTls {
    /// According to x86_64 ABI, `gs:[0]` should contain a pointer to the TLS
    /// itself. This enables unprivileged code to obtain the TLS's address.
    pub self_addr: usize,

    /// The syscall function's address, used by user mode program.
    ///
    /// Example: `call gs:[0x8]`
    pub syscaller: usize,

    /// The "foreign" stack pointer (i.e. user `rsp` in kernel context, and
    /// kernel `rsp` in user context).
    ///
    /// This is read and written using `xchg gs:[0x10], rsp`.
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

const KERNEL_STACK_SIZE: usize = 0x4000;
const KERNEL_STACK_LAYOUT: alloc::alloc::Layout = unsafe {
    alloc::alloc::Layout::from_size_align_unchecked(KERNEL_STACK_SIZE, kconfig::PAGE_SIZE)
};

impl KtaskTls {
    pub fn new() -> KtaskTls {
        KtaskTls {
            // self_addr and pcb_weak_ptr are initialized later
            syscaller: syscall_entry as usize,
            ..Default::default()
        }
    }

    pub fn set_pcb_weak_ptr(&mut self, new: usize) {
        self.pcb_weak_ptr = new;
    }
}

impl KtaskCtx {
    pub fn allocate_for(thread_ctx: *const KtaskTls, userspace_regs: &[u8]) -> KtaskCtx {
        // HACK: const_cast `thread_ctx` to a `*mut KtaskTls` in order to
        // initialize `self_addr`.
        // FIXME: This is UB and should be avoided.
        unsafe {
            (*(thread_ctx as *mut KtaskTls)).self_addr = thread_ctx as usize;
        }

        // Allocate kernel stack.
        let kernel_stack = unsafe { alloc::alloc::alloc(KERNEL_STACK_LAYOUT) };
        // Write userspace regs & the address of `ret_from_fork` to the end of
        // kernel stack.
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
    fn ktask_enter_asm(from: *mut KtaskCtx, to: *mut KtaskCtx);
    pub fn ktask_leave();
    // functions defined in `sgx-libos`
    fn ret_from_fork() -> !;
    fn syscall_entry();
}

core::arch::global_asm!(include_str!("task.asm"));

#[no_mangle]
pub unsafe extern "C" fn ktask_enter(from: *mut KtaskCtx, to: *mut KtaskCtx) {
    // Switch GS base using Rust code, since this involves an OCALL.
    let mut old_gs = 0xDEADBEEF;
    let result = crate::arch::sgx::ocall_switch_gs_base(&mut old_gs, (*to).gs_offset);
    assert_eq!(result, sgx_types::sgx_status_t::SGX_SUCCESS);

    // Save old GS to `from` if needed.
    if old_gs != 0 {
        // We are switching from a Ktask back to the sched.
        assert_eq!((*from).gs_offset, old_gs);
    } else {
        // We are switching from the sched to a Ktask.
        // Set the sched's gs_offset to 0 since it may be uninitialized.
        (*from).gs_offset = 0;
    }

    // Do the actual context switch.
    ktask_enter_asm(from, to);
}

pub fn current_pcb_weak() -> usize {
    let tls: usize;
    unsafe {
        // Read TLS pointer from `gs:[0]`.
        core::arch::asm!("mov {}, gs:[0]", out(reg) tls);
    }
    assert_ne!(tls, 0);

    let tls = tls as *const KtaskTls;
    unsafe { (*tls).pcb_weak_ptr }
}
