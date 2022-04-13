//! Lists all syscalls in a single module, allowing each architecture to easily
//! define their own syscall tables.

use super::*;

pub use dir::{SYSCALL_CHDIR, SYSCALL_GETCWD, SYSCALL_GETDENTS64, SYSCALL_MKDIRAT};
pub use file::{
    SYSCALL_CLOSE, SYSCALL_DUP, SYSCALL_DUP3, SYSCALL_OPENAT, SYSCALL_READ, SYSCALL_WRITE,
};
pub use process::SYSCALL_EXIT;
