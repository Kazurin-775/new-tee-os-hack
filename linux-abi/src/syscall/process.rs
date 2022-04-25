use hal::task::{Task, TaskFuture, UserspaceRegs};
use log::*;

use crate::Errno;

use super::SyscallHandler;

pub const SYSCALL_EXIT: SyscallHandler = SyscallHandler::Syscall1(syscall_exit);
pub const SYSCALL_CLONE: SyscallHandler = SyscallHandler::SyscallClone(syscall_clone);

const SIGCHLD: usize = 17;

unsafe fn syscall_exit(retval: usize) -> isize {
    debug!("U-mode program exited with status {}", retval);
    // hal::exit_enclave(retval);
    hal::task::current().lock().exited = true;
    hal::task::yield_to_sched();
    unreachable!("trying to re-schedule an already terminated task")
}

unsafe fn syscall_clone(regs: &UserspaceRegs, flags: usize, stack: usize) -> isize {
    if flags != SIGCHLD {
        log::warn!("clone() called with unsupported flags: {:#X}", flags);
        return Errno::EINVAL.as_neg_isize();
    }
    if stack != 0 {
        log::warn!("clone() with stack != 0 is not supported");
        return Errno::EINVAL.as_neg_isize();
    }

    let new_mm = hal::task::current().lock().mm.duplicate();
    let task = Task::create(new_mm, &regs);
    let pid = task.lock().pid;
    log::debug!("Created a new task with PID = {}", pid);
    executor::spawn(TaskFuture::new(task));
    pid as isize
}
