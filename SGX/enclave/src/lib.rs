#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // Forward the panic to sgx-libos's handler. This is due to a limitation of
    // the Rust compiler.
    sgx_libos::panic_handler(info)
}

#[no_mangle]
pub extern "C" fn t_global_exit_ecall() {}

#[no_mangle]
pub extern "C" fn t_global_init_ecall(_id: u64, _path: *const u8, _len: usize) {}

// Temporary fix for:
// library/alloc/src/fmt.rs:599: undefined reference to `_Unwind_Resume'
// These functions should not be touched anyway.
#[export_name = "_Unwind_Resume"]
unsafe extern "C" fn bogus_unwind_resume() {
    core::arch::asm!("ud2", options(noreturn));
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
