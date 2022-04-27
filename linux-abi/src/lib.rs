#![no_std]

extern crate alloc;

#[cfg(feature = "multitasking")]
pub mod elf;
mod errno;
pub mod exec;
pub mod limits;
pub mod syscall;

pub use errno::Errno;
