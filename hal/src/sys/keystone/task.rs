#[repr(C)]
#[derive(Default)]
pub struct KtaskTls {
    user_sp: usize,
    kernel_sp: usize,
    // used by assembly code, should not be touched by Rust code
    prev_ktask_ctx: usize,
    cur_ktask_ctx: usize,
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

const KERNEL_STACK_SIZE: usize = 0x4_000;
const KERNEL_STACK_LAYOUT: alloc::alloc::Layout = unsafe {
    alloc::alloc::Layout::from_size_align_unchecked(KERNEL_STACK_SIZE, kconfig::PAGE_SIZE)
};

impl KtaskTls {
    pub fn from_user_sp(user_sp: usize) -> KtaskTls {
        KtaskTls {
            user_sp,
            ..Default::default()
        }
    }
}

impl KtaskCtx {
    pub fn allocate_for(thread_ctx: *const KtaskTls) -> KtaskCtx {
        let kernel_stack = unsafe { alloc::alloc::alloc(KERNEL_STACK_LAYOUT) };
        assert!(!kernel_stack.is_null(), "failed to allocate kernel stack");
        let kernel_stack_end = unsafe { kernel_stack.add(KERNEL_STACK_SIZE) };
        KtaskCtx {
            sp: kernel_stack_end as usize,
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

pub fn ensure_ktask_context() {
    // check if tp is non-zero
    let tp: usize;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) tp);
    }
    assert_ne!(tp, 0);
}

core::arch::global_asm!(include_str!("./task.S"));
