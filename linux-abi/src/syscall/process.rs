use alloc::{string::String, vec::Vec};
use edge_proto::EdgeCallReq;
use hal::task::UserspaceRegs;

use super::SyscallHandler;

pub const SYSCALL_EXIT: SyscallHandler = SyscallHandler::Syscall1(syscall_exit);
pub const SYSCALL_CLONE: SyscallHandler = SyscallHandler::SyscallClone(syscall_clone);
pub const SYSCALL_EXECVE_PRE: SyscallHandler = SyscallHandler::SyscallExecvePre(syscall_execve_pre);

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
    use crate::Errno;
    use hal::task::{Task, TaskFuture};

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

#[cfg(feature = "multitasking")]
unsafe fn syscall_execve_pre(
    path: usize,
    mut argv_ptr: usize,
    mut envp_ptr: usize,
) -> Result<(String, Vec<String>, Vec<String>), isize> {
    let mut path_buf = alloc::vec![0; crate::limits::PATH_MAX];

    let mut read_string_from_user = |ptr: *const u8| -> Result<String, isize> {
        let path_len = hal::mem::strncpy_from_user(&mut path_buf, ptr);
        if path_len >= path_buf.len() {
            log::error!("execve: Path buffer overflow");
            return Err(crate::Errno::EFAULT.as_neg_isize());
        }
        Ok(String::from(
            core::str::from_utf8(&path_buf[0..path_len]).expect("invalid UTF-8"),
        ))
    };

    // Read path, argv, envp
    let path = read_string_from_user(path as *const u8)?;
    let mut argv = Vec::new();
    loop {
        let ptr = hal::mem::read_from_user(argv_ptr as *const *const u8);
        if ptr.is_null() {
            break;
        }
        argv.push(read_string_from_user(ptr)?);
        argv_ptr += core::mem::size_of::<usize>();
    }
    let mut envp = Vec::new();
    loop {
        let ptr = hal::mem::read_from_user(envp_ptr as *const *const u8);
        if ptr.is_null() {
            break;
        }
        envp.push(read_string_from_user(ptr)?);
        envp_ptr += core::mem::size_of::<usize>();
    }

    Ok((path, argv, envp))
}

#[cfg(not(feature = "multitasking"))]
unsafe fn syscall_execve_pre(
    _path: usize,
    _argv: usize,
    _envp: usize,
) -> Result<(String, Vec<String>, Vec<String>), isize> {
    panic!("execve() is not supported without `multitasking` feature");
}
