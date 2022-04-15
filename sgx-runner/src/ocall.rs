#[no_mangle]
unsafe extern "C" fn ocall_syscall_write(fd: i32, buf: *const i8, count: usize) -> i64 {
    libc::write(fd, buf.cast(), count) as i64
}

#[no_mangle]
unsafe extern "C" fn ocall_edge_kick() {
    if let Err(err) = edge_responder::handle_edge_call(&mut *crate::EDGE_MEM.lock().unwrap()) {
        log::error!("Failed to handle edge call: {:#}", err);
        std::process::exit(1);
    }
}

#[no_mangle]
unsafe extern "C" fn ocall_exit(retval: i32) {
    log::info!("Enclave exited with status {}", retval);
    // TODO: perform cleanups
    std::process::exit(retval);
}
