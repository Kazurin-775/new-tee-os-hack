use edge_proto::EdgeCallReq;
use hal::task::UserspaceRegs;

use super::SyscallHandler;

pub const SYSCALL_EXIT: SyscallHandler = SyscallHandler::Syscall1(syscall_exit);
pub const SYSCALL_CLONE: SyscallHandler = SyscallHandler::SyscallClone(syscall_clone);

unsafe fn syscall_exit(retval: usize) -> isize {
    let current = hal::task::current();
    let mut cur_lock = current.lock();
    let pid = cur_lock.pid;
    log::debug!("PID {} exited with status {}", pid, retval);

    // Drop PCB at the edge responder side
    hal::edge::with_edge_caller(|caller| {
        caller.write_header(&EdgeCallReq::PcbDrop { pid }).unwrap();
        caller.kick().unwrap();
        assert!(caller.read_header().unwrap().is_ok());
    });

    // hal::exit_enclave(retval);
    cur_lock.exited = true;
    // Free the local variables, or they will cause deadlocks / resource leaks
    drop(cur_lock);
    drop(current);
    hal::task::yield_to_sched();
    unreachable!("trying to re-schedule an already terminated task")
}

#[cfg(feature = "multitasking")]
unsafe fn syscall_clone(regs: &UserspaceRegs, flags: usize, stack: usize) -> isize {
    use hal::task::{Task, TaskFuture};
    use crate::Errno;

    const SIGCHLD: usize = 17;

    if flags != SIGCHLD {
        log::warn!("clone() called with unsupported flags: {:#X}", flags);
        return Errno::EINVAL.as_neg_isize();
    }
    if stack != 0 {
        log::warn!("clone() with stack != 0 is not supported");
        return Errno::EINVAL.as_neg_isize();
    }

    let (cur_pid, new_mm);
    {
        let current = hal::task::current();
        let cur_lock = current.lock();
        // Get current PID
        cur_pid = cur_lock.pid;
        // Clone the address space
        new_mm = cur_lock.mm.duplicate();
    }
    // Create a new task
    let task = Task::create(new_mm, &regs);
    let pid = task.lock().pid;
    log::debug!("Created a new task with PID = {}", pid);
    // Duplicate PCB at the edge responder side
    hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::PcbDup {
                from: cur_pid,
                to: pid,
            })
            .unwrap();
        caller.kick().unwrap();
        assert!(caller.read_header().unwrap().is_ok());
    });
    // Spawn the new task in the async executor
    executor::spawn(TaskFuture::new(task));

    pid as isize
}

#[cfg(not(feature = "multitasking"))]
unsafe fn syscall_clone(_regs: &UserspaceRegs, _flags: usize, _stack: usize) -> isize {
    panic!("clone() is not supported without `multitasking` feature");
}
