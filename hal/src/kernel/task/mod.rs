use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::mem::MaybeUninit;
use spin::Mutex;

mod pid_pool;

use crate::sys::task::*;
pub use pid_pool::PidPool;

pub type Pid = i32;

pub static PID_POOL: Mutex<PidPool> = Mutex::new(PidPool::new());

pub struct Task {
    pub pid: Pid,

    /// The kernel thread's TLS (thread local storage), used by the `current!`
    /// macro and `ret_from_fork`. Only a kernel task has a TLS (the scheduler
    /// does not).
    ///
    /// Stores the kernel's `sp` and user's `sp`, and acts as a bridge between
    /// the user context and the kernel context.
    ///
    /// This will be loaded into a register accessible in ISRs, and will be used
    /// to fetch the kernel's stack pointer (`sp`) at the beginning of the ISR.
    /// Therefore, its address must be kept static during the kernel's lifetime.
    /// We use a `Box` to achieve this.
    pub tls: Box<KtaskTls>,

    /// The kernel task's context, used to switch between kernel tasks and the
    /// scheduler.
    ///
    /// The task's kernel sp and any callee-saved registers will be put here.
    ///
    /// This field can be borrowed by [`TaskFuture::poll`]. A `None` indicates a
    /// borrowed (vacant) state. When the scheduler switches to this task,
    /// the scheduler takes the `KernelCtx` away (replacing it with
    /// `None`), and then returns it when the task yields back.
    /// This field should never be `None` at any other time.
    pub ktask_ctx: Option<KtaskCtx>,
}

impl Task {
    pub fn create(user_sp: usize) -> Task {
        let pid = PID_POOL.try_lock().unwrap().alloc();
        let tls = Box::new(KtaskTls::from_user_sp(user_sp));
        // TODO: free kernel stack
        let ktask_ctx = Some(KtaskCtx::allocate_for(tls.as_ref()));
        let task = Task {
            pid,
            tls,
            ktask_ctx,
        };
        task
    }
}

pub struct TaskFuture {
    task: Arc<Mutex<Task>>,
}

impl TaskFuture {
    pub fn new(task: Arc<Mutex<Task>>) -> TaskFuture {
        TaskFuture { task }
    }
}

impl Future for TaskFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        // The scheduler's `KtaskCtx` (write only).
        let mut prev_ktask_ctx = MaybeUninit::uninit();

        // Borrow the Task's `KtaskCtx` to ensure exclusive access.
        let mut next_ktask_ctx = self
            .task
            .try_lock()
            .unwrap()
            .ktask_ctx
            .take()
            .expect("ktask context is vacant");

        // Enter the Task!
        unsafe {
            ktask_enter(prev_ktask_ctx.as_mut_ptr(), &mut next_ktask_ctx);
        }

        // `next_ktask_ctx` is now modified, return it to the Task's `KtaskCtx`.
        assert!(self
            .task
            .try_lock()
            .unwrap()
            .ktask_ctx
            .replace(next_ktask_ctx)
            .is_none());

        cx.waker().wake_by_ref();
        core::task::Poll::Pending
    }
}

pub fn yield_to_sched() {
    // Ensure that the TLS is valid (i.e. we are actually inside a Task).
    ensure_ktask_context();

    // Switch back to previous `KtaskCtx` stored in the TLS.
    unsafe {
        ktask_leave();
    }
}

/// The system idle task, used for debugging.
pub struct IdleTask;

impl Future for IdleTask {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        crate::println!("System idle...");
        cx.waker().wake_by_ref();
        core::task::Poll::Pending
    }
}

pub fn current_pid() -> Pid {
    // TODO: support multitasking
    1
}
