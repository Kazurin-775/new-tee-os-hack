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

#[no_mangle]
unsafe extern "C" fn ocall_switch_gs_base(new_gs_base: usize) -> usize {
    const ARCH_GET_GS: i32 = 0x1004;
    const ARCH_SET_GS: i32 = 0x1001;

    let mut old_gs_base = 0xDEADBEEF;
    libc::syscall(libc::SYS_arch_prctl, ARCH_GET_GS, &mut old_gs_base);
    log::trace!("Switching GS: {:#X} -> {:#X}", old_gs_base, new_gs_base);
    libc::syscall(libc::SYS_arch_prctl, ARCH_SET_GS, new_gs_base);

    old_gs_base
}
