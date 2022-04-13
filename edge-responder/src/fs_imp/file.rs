use std::{
    fs::File,
    os::unix::prelude::{AsRawFd, FromRawFd},
    sync::{Arc, Mutex},
};

use nix::{fcntl::OFlag, sys::stat::Mode, unistd::UnlinkatFlags};

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

    fn find_free_fd(&self) -> i32 {
        // naive implementation
        (0..4096)
            .filter(|fd| !self.fd_mappings.contains_key(fd))
            .next()
            .unwrap()
    }

    pub fn open(&mut self, path: &str, flags: i32, mode: u32) -> LinuxResult<i32> {
        let path = self.resolve_path(&path);
        let fd = nix::fcntl::open(
            &path,
            OFlag::from_bits_truncate(flags),
            Mode::from_bits_truncate(mode),
        )?;
        let file = Arc::new(Mutex::new(TeeFile::File(unsafe { File::from_raw_fd(fd) })));

        let dest_fd = self.find_free_fd();
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

    pub fn dup(&mut self, src_fd: i32, dest_fd: Option<i32>) -> LinuxResult<i32> {
        let src_fd = Arc::clone(self.fd_mappings.get(&src_fd).ok_or(nix::Error::EBADFD)?);
        let dest_fd = dest_fd.unwrap_or_else(|| self.find_free_fd());
        if self.fd_mappings.remove(&dest_fd).is_some() {
            log::debug!("dup2: overwriting fd {}", dest_fd);
        }
        self.fd_mappings.insert(dest_fd, src_fd);
        Ok(dest_fd)
    }

    pub fn unlink_at(&self, dir_fd: i32, path: &str, flags: i32) -> LinuxResult<()> {
        let flags = match flags {
            0 => UnlinkatFlags::NoRemoveDir,
            nix::libc::AT_REMOVEDIR => UnlinkatFlags::RemoveDir,
            other => {
                log::warn!("unlinkat: unknown flags: {}", other);
                return Err(nix::Error::EINVAL);
            }
        };
        if dir_fd == nix::libc::AT_FDCWD {
            let path = self.resolve_path(path);
            nix::unistd::unlinkat(None, &path, flags)
        } else {
            log::debug!("unlinkat: dirfd = {}", dir_fd);
            let dir_file = self.fd_mappings.get(&dir_fd).ok_or_else(|| {
                log::warn!("unlinkat: unknown dir fd: {}", dir_fd);
                nix::Error::EBADFD
            })?;
            nix::unistd::unlinkat(Some(dir_file.lock().unwrap().as_raw_fd()), path, flags)
        }
    }
}
