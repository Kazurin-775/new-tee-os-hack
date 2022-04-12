use nix::sys::stat::Mode;

use crate::error::SyscallResult;

use super::TaskFsContext;

impl TaskFsContext {
    pub fn mkdirat(&self, fd: i32, path: &str, mode: u32) -> SyscallResult<isize> {
        if fd != nix::libc::AT_FDCWD {
            return Err(anyhow::anyhow!("only FD_ATCWD is supported").into());
        }

        nix::unistd::mkdir(path, Mode::from_bits_truncate(mode))?;

        Ok(0)
    }
}
