use alloc::{borrow::ToOwned, vec};
use edge_proto::EdgeCallReq;

use crate::Errno;

use super::SyscallHandler;

pub const SYSCALL_MKDIRAT: SyscallHandler = SyscallHandler::Syscall3(syscall_mkdirat);
pub const SYSCALL_GETCWD: SyscallHandler = SyscallHandler::Syscall2(syscall_getcwd);
pub const SYSCALL_CHDIR: SyscallHandler = SyscallHandler::Syscall1(syscall_chdir);
pub const SYSCALL_GETDENTS64: SyscallHandler = SyscallHandler::Syscall3(syscall_getdents64);

unsafe fn syscall_mkdirat(fd: usize, path: usize, mode: usize) -> isize {
    let mut path_buf = vec![0; crate::limits::PATH_MAX];
    let path_len = hal::mem::strncpy_from_user(&mut path_buf, path as *const u8);
    if path_len >= path_buf.len() {
        log::error!("mkdirat: Path buffer overflow");
        return Errno::EFAULT.as_neg_isize();
    }
    let path = core::str::from_utf8(&path_buf[0..path_len]).expect("path is not valid UTF-8");

    let result = hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallMkdirAt {
                pid: hal::task::current_pid(),
                fd: fd as i32,
                path: path.to_owned(),
                mode: mode as u32,
            })
            .unwrap();
        caller.kick().unwrap();
        caller.read_header().unwrap().into_syscall_resp().unwrap() as isize
    });

    log::trace!("mkdirat(_, {:?}, {:#o}) = {}", path, mode, result);
    result
}

unsafe fn syscall_getcwd(buf: usize, size: usize) -> isize {
    let cwd = hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallGetCwd {
                pid: hal::task::current_pid(),
            })
            .unwrap();
        caller.kick().unwrap();
        caller.read_header().unwrap().into_ok_with_string().unwrap()
    });

    log::trace!("getcwd() = {}", cwd);
    if cwd.len() + 1 > size {
        log::warn!("getcwd: Buffer overflow");
        return Errno::ERANGE.as_neg_isize();
    }

    hal::mem::copy_to_user(cwd.as_bytes(), buf as *mut u8);
    hal::mem::copy_to_user(&[0], (buf + cwd.len()) as *mut u8);

    0
}

unsafe fn syscall_chdir(path: usize) -> isize {
    let mut path_buf = vec![0; crate::limits::PATH_MAX];
    let path_len = hal::mem::strncpy_from_user(&mut path_buf, path as *const u8);
    if path_len >= path_buf.len() {
        log::error!("mkdirat: Path buffer overflow");
        return Errno::EFAULT.as_neg_isize();
    }
    let path = core::str::from_utf8(&path_buf[0..path_len]).expect("path is not valid UTF-8");

    let result = hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallChdir {
                pid: hal::task::current_pid(),
                path: path.to_owned(),
            })
            .unwrap();
        caller.kick().unwrap();
        caller.read_header().unwrap().into_syscall_resp().unwrap() as isize
    });

    log::trace!("chdir({:?}) = {}", path, result);
    result
}

unsafe fn syscall_getdents64(fd: usize, buf: usize, size: usize) -> isize {
    let result = hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallGetDents64 {
                pid: hal::task::current_pid(),
                fd: fd as i32,
                len: size.min(hal::cfg::EDGE_BUFFER_SIZE) as u64,
            })
            .unwrap();
        caller.kick().unwrap();

        let result = caller.read_header().unwrap().into_syscall_resp().unwrap();
        if result > 0 {
            assert!((result as usize) <= size);
            let data = caller.read_data().unwrap();
            hal::mem::copy_to_user(&data[0..result as usize], buf as *mut u8);
        }
        result
    });

    log::trace!("getdents64({}, _, {}) = {}", fd, size, result);
    result as isize
}
