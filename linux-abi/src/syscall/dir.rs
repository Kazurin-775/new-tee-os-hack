use alloc::{borrow::ToOwned, vec};
use edge_proto::EdgeCallReq;

use crate::Errno;

use super::SyscallHandler;

pub const SYSCALL_MKDIRAT: SyscallHandler = SyscallHandler::Syscall3(syscall_mkdirat);

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
                pid: 1, // FIXME
                fd: fd as i32,
                path: path.to_owned(),
                mode: mode as u32,
            })
            .unwrap();
        caller.kick().unwrap();
        caller.read_header().unwrap().into_syscall_resp().unwrap() as isize
    });

    log::debug!("mkdirat(_, {:?}, {:#o}) = {}", path, mode, result);
    result
}
