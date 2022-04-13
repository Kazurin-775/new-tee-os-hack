use std::{os::unix::prelude::AsRawFd, sync::Arc};

use anyhow::Context;
use edge_proto::{server::EdgeStream, EdgeCallResp};

use crate::{
    error::{EdgeErrorCompat, SyscallResult},
    pcb::TASKS,
};

pub fn openat(
    _stream: &mut dyn EdgeStream,
    pid: i32,
    dir_fd: i32,
    path: String,
    flags: i32,
    mode: u32,
) -> SyscallResult<isize> {
    if dir_fd != nix::libc::AT_FDCWD {
        return Err(anyhow::anyhow!("only FD_ATCWD is supported").into());
    }

    TASKS
        .lock()
        .unwrap()
        .get_mut(&pid)
        .ok_or(anyhow::anyhow!("no such process"))?
        .fs
        .open(&path, flags, mode)
        .map(|fd| fd as isize)
        .map_err(Into::into)
}

pub fn special_read(
    stream: &mut dyn EdgeStream,
    pid: i32,
    fd: i32,
    len: u64,
) -> anyhow::Result<()> {
    let local_file = Arc::clone(
        TASKS
            .lock()
            .unwrap()
            .get(&pid)
            .ok_or(anyhow::anyhow!("no such process"))?
            .fs
            .find_fd(fd as i32)?,
    );
    let guard = local_file.lock().unwrap();

    let mut buf = vec![0; len as usize];
    let result = nix::unistd::read(guard.as_raw_fd(), &mut buf);

    stream
        .write_header(&EdgeCallResp::SyscallResp(
            result
                .map(|len| len.try_into().unwrap()) // usize -> isize
                .map_err(|errno| errno as isize)
                .unwrap_or_else(std::convert::identity) as i64,
        ))
        .compat()
        .context("write header")?;
    if let Ok(len) = result {
        stream
            .write_data(&buf[0..len])
            .compat()
            .context("write data")?;
    }

    Ok(())
}

pub fn write(stream: &mut dyn EdgeStream, pid: i32, fd: i32, len: u64) -> SyscallResult<isize> {
    let local_file = Arc::clone(
        TASKS
            .lock()
            .unwrap()
            .get(&pid)
            .ok_or(anyhow::anyhow!("no such process"))?
            .fs
            .find_fd(fd as i32)?,
    );
    let guard = local_file.lock().unwrap();

    let result = nix::unistd::write(
        guard.as_raw_fd(),
        &stream.read_data().compat().context("read data")?[0..len as usize],
    )
    .map(|len| len.try_into().unwrap())?; // usize -> isize

    Ok(result)
}

pub fn close(_stream: &mut dyn EdgeStream, pid: i32, fd: i32) -> SyscallResult<isize> {
    TASKS
        .lock()
        .unwrap()
        .get_mut(&pid)
        .ok_or(anyhow::anyhow!("no such process"))?
        .fs
        .close(fd)
        .map(|()| 0)
        .map_err(Into::into)
}

pub fn dup(
    _stream: &mut dyn EdgeStream,
    pid: i32,
    src_fd: i32,
    dest_fd: Option<i32>,
) -> SyscallResult<isize> {
    TASKS
        .lock()
        .unwrap()
        .get_mut(&pid)
        .ok_or(anyhow::anyhow!("no such process"))?
        .fs
        .dup(src_fd, dest_fd)
        .map(|fd| fd as isize)
        .map_err(Into::into)
}

pub fn mkdirat(
    _stream: &mut dyn EdgeStream,
    pid: i32,
    fd: i32,
    path: String,
    mode: u32,
) -> SyscallResult<isize> {
    TASKS
        .lock()
        .unwrap()
        .get(&pid)
        .ok_or(anyhow::anyhow!("no such process"))?
        .fs
        .mkdirat(fd, &path, mode)
}

pub fn chdir(_stream: &mut dyn EdgeStream, pid: i32, path: String) -> SyscallResult<isize> {
    TASKS
        .lock()
        .unwrap()
        .get_mut(&pid)
        .ok_or(anyhow::anyhow!("no such process"))?
        .fs
        .chdir(&path)
        .map(|()| 0)
        .map_err(Into::into)
}

pub fn special_getcwd(stream: &mut dyn EdgeStream, pid: i32) -> anyhow::Result<()> {
    let cwd = TASKS
        .lock()
        .unwrap()
        .get_mut(&pid)
        .ok_or(anyhow::anyhow!("no such process"))?
        .fs
        .cwd();
    stream
        .write_header(&EdgeCallResp::OkWithString(cwd))
        .compat()
        .context("write header")?;
    Ok(())
}
