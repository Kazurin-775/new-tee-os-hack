use core::fmt::Write;

pub struct UnsafeUart;

extern "C" {
    fn ocall_syscall_write(eid: u64, fd: usize, ptr: *const u8, len: usize) -> isize;
}

impl Write for UnsafeUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let edge_buf = crate::EDGE_BUF.get().unwrap();
        let edge_buf =
            unsafe { core::slice::from_raw_parts_mut(edge_buf.0 as *mut u8, edge_buf.1) };

        for chunk in s.as_bytes().chunks(kconfig::EDGE_BUFFER_SIZE) {
            edge_buf[0..chunk.len()].copy_from_slice(chunk);
            unsafe {
                ocall_syscall_write(0, 1, edge_buf.as_ptr(), chunk.len());
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! uart_print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        write!($crate::uart::UnsafeUart, $($args)+).unwrap()
    });
}

#[macro_export]
macro_rules! uart_println {
    () => ({
        $crate::uart_print!("\n")
    });
    ($fmt:expr) => ({
        $crate::uart_print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::uart_print!(concat!($fmt, "\n"), $($args)+)
    });
}
