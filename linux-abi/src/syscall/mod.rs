mod dir;
mod file;
pub mod listing;
mod mem;
mod process;
pub mod tables;

#[derive(Clone, Copy)]
pub enum SyscallHandler {
    Syscall1(unsafe fn(usize) -> isize),
    Syscall2(unsafe fn(usize, usize) -> isize),
    Syscall3(unsafe fn(usize, usize, usize) -> isize),
    Syscall4(unsafe fn(usize, usize, usize, usize) -> isize),
    Syscall6(unsafe fn(usize, usize, usize, usize, usize, usize) -> isize),
}

#[macro_export]
macro_rules! syscall_try {
    ($val:expr) => {{
        let val: isize = { $val };
        if val < 0 {
            return val;
        }
        val
    }};
}
