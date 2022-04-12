use alloc::{borrow::ToOwned, vec};
use edge_proto::EdgeCallReq;
use hal::edge::EDGE_BUFFER_SIZE;

use super::SyscallHandler;
use crate::{syscall_try, Errno};

pub const SYSCALL_OPENAT: SyscallHandler = SyscallHandler::Syscall4(syscall_openat);
pub const SYSCALL_READ: SyscallHandler = SyscallHandler::Syscall3(syscall_read);
pub const SYSCALL_WRITE: SyscallHandler = SyscallHandler::Syscall3(syscall_write);
pub const SYSCALL_CLOSE: SyscallHandler = SyscallHandler::Syscall1(syscall_close);

unsafe fn edge_read(fd: usize, buf: &mut [u8]) -> isize {
    hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallRead {
                pid: 1, // FIXME
                fd: fd as i32,
                len: buf.len() as u64,
            })
            .unwrap();
        caller.kick().unwrap();

        let result = caller.read_header().unwrap().into_syscall_resp().unwrap() as isize;
        if result >= 0 {
            let data = caller.read_data().unwrap();
            buf[0..result as usize].copy_from_slice(&data[0..result as usize]);
        }
        result
    })
}

unsafe fn edge_write(fd: usize, buf: &[u8]) -> isize {
    hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallWrite {
                pid: 1, // FIXME
                fd: fd as i32,
                len: buf.len() as u64,
            })
            .unwrap();
        caller.write_data(buf).unwrap();
        caller.kick().unwrap();

        caller.read_header().unwrap().into_syscall_resp().unwrap() as isize
    })
}

unsafe fn syscall_openat(dir_fd: usize, path: usize, flags: usize, mode: usize) -> isize {
    let mut path_buf = vec![0; crate::limits::PATH_MAX];
    let path_len = hal::mem::strncpy_from_user(&mut path_buf, path as *const u8);
    if path_len >= path_buf.len() {
        log::error!("openat: Path buffer overflow");
        return Errno::EFAULT.as_neg_isize();
    }
    let path = core::str::from_utf8(&path_buf[0..path_len]).expect("path is not valid UTF-8");

    let result = hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallOpenAt {
                pid: 1, // FIXME
                dir_fd: dir_fd as i32,
                path: path.to_owned(),
                flags: flags as i32,
                mode: mode as u32,
            })
            .unwrap();
        caller.kick().unwrap();

        caller.read_header().unwrap().into_syscall_resp().unwrap() as isize
    });
    log::trace!(
        "openat({}, {:?}, {}, {:#o}) = {}",
        dir_fd,
        path,
        flags,
        mode,
        result,
    );
    result
}

unsafe fn syscall_read(fd: usize, ptr: usize, mut len: usize) -> isize {
    let mut total_bytes_read = 0;

    let mut ptr = ptr as *mut u8;
    let mut buf = vec![0; EDGE_BUFFER_SIZE];
    while len > EDGE_BUFFER_SIZE {
        let bytes_read = syscall_try!(edge_read(fd, &mut buf)) as usize;
        total_bytes_read += bytes_read;
        hal::mem::copy_to_user(&buf[0..bytes_read], ptr as *mut u8);
        ptr = ptr.add(bytes_read);
        len -= bytes_read;
    }

    let bytes_read = syscall_try!(edge_read(fd, &mut buf)) as usize;
    total_bytes_read += bytes_read;
    hal::mem::copy_to_user(&buf[0..bytes_read], ptr as *mut u8);

    total_bytes_read as isize
}

unsafe fn syscall_write(fd: usize, ptr: usize, mut len: usize) -> isize {
    let mut bytes_written = 0;

    let mut ptr = ptr as *const u8;
    let mut buf = vec![0; EDGE_BUFFER_SIZE];
    while len > EDGE_BUFFER_SIZE {
        hal::mem::copy_from_user(&mut buf, ptr as *const u8);
        bytes_written += syscall_try!(edge_write(fd, &buf));
        ptr = ptr.add(EDGE_BUFFER_SIZE);
        len -= EDGE_BUFFER_SIZE;
    }

    hal::mem::copy_from_user(&mut buf[0..len], ptr as *const u8);
    bytes_written += syscall_try!(edge_write(fd, &buf[0..len]));

    bytes_written
}

unsafe fn syscall_close(fd: usize) -> isize {
    let result = hal::edge::with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::SyscallClose {
                pid: 1, // FIXME
                fd: fd as i32,
            })
            .unwrap();
        caller.kick().unwrap();

        caller.read_header().unwrap().into_syscall_resp().unwrap() as isize
    });
    log::trace!("close({}) = {}", fd, result);
    result
}
