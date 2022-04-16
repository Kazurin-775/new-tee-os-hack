use log::*;

use super::SyscallHandler;

pub const SYSCALL_EXIT: SyscallHandler = SyscallHandler::Syscall1(syscall_exit);

unsafe fn syscall_exit(retval: usize) -> isize {
    debug!("U-mode program exited with status {}", retval);
    // hal::exit_enclave(retval);
    hal::task::current().upgrade().unwrap().lock().exited = true;
    hal::task::yield_to_sched();
    unreachable!("trying to re-schedule an already terminated task")
}
