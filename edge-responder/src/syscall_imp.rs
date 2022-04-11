use std::{os::unix::prelude::AsRawFd, sync::Arc};

use anyhow::Context;
use edge_proto::server::EdgeStream;

use crate::{
    error::{EdgeErrorCompat, SyscallResult},
    pcb::TASKS,
};

pub fn write(stream: &mut dyn EdgeStream, pid: i32, fd: u64, len: u64) -> SyscallResult<isize> {
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
