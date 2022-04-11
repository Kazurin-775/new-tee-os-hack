use std::{
    os::unix::prelude::AsRawFd,
    sync::{Arc, Mutex},
};

use crate::error::LinuxResult;

use super::TaskFsContext;

pub enum TeeFile {
    Stdio(i32),
}

impl AsRawFd for TeeFile {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match self {
            TeeFile::Stdio(fd) => *fd,
        }
    }
}

impl TaskFsContext {
    pub fn find_fd(&self, fd: i32) -> LinuxResult<&Arc<Mutex<TeeFile>>> {
        self.fd_mappings.get(&fd).ok_or(nix::Error::EBADFD)
    }
}
