//! System call tables for various architectures.
//!
//! If this does not contain what you need, feel free to define your own one.

use phf::{phf_map, Map};

use super::{listing::*, SyscallHandler};

// https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/unistd.h
pub static TABLE_GENERIC: Map<u32, SyscallHandler> = phf_map! {
    17u32 => SYSCALL_GETCWD,
    23u32 => SYSCALL_DUP,
    24u32 => SYSCALL_DUP3,
    34u32 => SYSCALL_MKDIRAT,
    49u32 => SYSCALL_CHDIR,
    56u32 => SYSCALL_OPENAT,
    57u32 => SYSCALL_CLOSE,
    61u32 => SYSCALL_GETDENTS64,
    63u32 => SYSCALL_READ,
    64u32 => SYSCALL_WRITE,
    80u32 => SYSCALL_FSTAT,
    93u32 => SYSCALL_EXIT,
};

// https://blog.rchapman.org/posts/Linux_System_Call_Table_for_x86_64/
pub static TABLE_X86_64: Map<u32, SyscallHandler> = phf_map! {
    0u32 => SYSCALL_READ,
    1u32 => SYSCALL_WRITE,
    3u32 => SYSCALL_CLOSE,
    5u32 => SYSCALL_FSTAT,
    32u32 => SYSCALL_DUP,
    60u32 => SYSCALL_EXIT,
    79u32 => SYSCALL_GETCWD,
    80u32 => SYSCALL_CHDIR,
    217u32 => SYSCALL_GETDENTS64,
    257u32 => SYSCALL_OPENAT,
    258u32 => SYSCALL_MKDIRAT,
    292u32 => SYSCALL_DUP3,
};
