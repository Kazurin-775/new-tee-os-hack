use kconfig::KERNEL_STACK_SIZE;

pub use crate::arch::keystone::frame::UserspaceRegs;

#[repr(C)]
#[derive(Default)]
pub struct KtaskTls {
    user_sp: usize,
    kernel_sp: usize,
    // used by assembly code, should not be touched by Rust code
    prev_ktask_ctx: usize,
    cur_ktask_ctx: usize,

    // the following fields are used by Rust code
    pub pcb_weak_ptr: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct KtaskCtx {
    sp: usize,
    ra: usize,
    tp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

const KERNEL_STACK_LAYOUT: alloc::alloc::Layout = unsafe {
    alloc::alloc::Layout::from_size_align_unchecked(KERNEL_STACK_SIZE, kconfig::PAGE_SIZE)
};

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
        assert!(!kernel_stack.is_null(), "failed to allocate kernel stack");
        // copy userspace regs to the end of kernel stack
        assert!(userspace_regs.len() <= KERNEL_STACK_SIZE);
        let kernel_sp;
        unsafe {
            kernel_sp = kernel_stack.add(KERNEL_STACK_SIZE - userspace_regs.len());
            core::slice::from_raw_parts_mut(kernel_sp, userspace_regs.len())
                .copy_from_slice(userspace_regs);
        }

        KtaskCtx {
            sp: kernel_sp as usize,
            ra: ret_from_fork as usize,
            tp: thread_ctx as usize,
            ..Default::default()
        }
    }
}

// functions defined in task.S
extern "C" {
    pub fn ktask_enter(from: *mut KtaskCtx, to: *mut KtaskCtx);
    pub fn ktask_leave();
    fn ret_from_fork() -> !;
}

pub fn current_pcb_weak() -> usize {
    // check if tp is non-zero
    let tp: usize;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) tp);
    }
    assert_ne!(tp, 0);

    // read PCB pointer from TLS
    let tls = tp as *const KtaskTls;
    unsafe { (*tls).pcb_weak_ptr }
}

core::arch::global_asm!(include_str!("./task.S"));
