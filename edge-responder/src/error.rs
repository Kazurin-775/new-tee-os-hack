use edge_proto::server::EdgeCallError;

pub trait EdgeErrorCompat<T> {
    fn compat(self) -> anyhow::Result<T>;
}

impl<T> EdgeErrorCompat<T> for Result<T, EdgeCallError> {
    fn compat(self) -> anyhow::Result<T> {
        self.map_err(|_err| anyhow::anyhow!("failed to perform edge call"))
    }
}

pub type LinuxResult<T> = Result<T, nix::Error>;
pub type SyscallResult<T> = Result<T, SyscallError>;

pub enum SyscallError {
    Linux(nix::Error, Option<anyhow::Error>),
    Internal(anyhow::Error),
}

impl From<nix::Error> for SyscallError {
    fn from(errno: nix::Error) -> Self {
        SyscallError::Linux(errno, None)
    }
}

impl From<anyhow::Error> for SyscallError {
    fn from(err: anyhow::Error) -> Self {
        SyscallError::Internal(err)
    }
}
