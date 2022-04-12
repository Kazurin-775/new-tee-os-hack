use std::{
    fs::File,
    os::unix::prelude::{AsRawFd, FromRawFd},
    sync::{Arc, Mutex},
};

use nix::{fcntl::OFlag, sys::stat::Mode};

use crate::error::LinuxResult;

use super::TaskFsContext;

pub enum TeeFile {
    Stdio(i32),
    File(File),
}

impl AsRawFd for TeeFile {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match self {
            TeeFile::Stdio(fd) => *fd,
            TeeFile::File(file) => file.as_raw_fd(),
        }
    }
}

impl TaskFsContext {
    pub fn find_fd(&self, fd: i32) -> LinuxResult<&Arc<Mutex<TeeFile>>> {
        self.fd_mappings.get(&fd).ok_or(nix::Error::EBADFD)
    }

    pub fn open(&mut self, path: &str, flags: i32, mode: u32) -> LinuxResult<i32> {
        let path = self.resolve_path(&path);
        let fd = nix::fcntl::open(
            &path,
            OFlag::from_bits_truncate(flags),
            Mode::from_bits_truncate(mode),
        )?;
        let file = Arc::new(Mutex::new(TeeFile::File(unsafe { File::from_raw_fd(fd) })));

        // naive implementation
        let dest_fd = (0..4096)
            .filter(|fd| !self.fd_mappings.contains_key(fd))
            .next()
            .unwrap();
        self.fd_mappings.insert(dest_fd, file);
        Ok(dest_fd)
    }

    pub fn close(&mut self, fd: i32) -> LinuxResult<()> {
        if self.fd_mappings.remove(&fd).is_some() {
            Ok(())
        } else {
            Err(nix::Error::EBADFD)
        }
    }
}
