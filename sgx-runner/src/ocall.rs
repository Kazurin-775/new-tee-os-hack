#[no_mangle]
unsafe extern "C" fn ocall_syscall_write(fd: i32, buf: *const i8, count: usize) -> i64 {
    libc::write(fd, buf.cast(), count) as i64
}
