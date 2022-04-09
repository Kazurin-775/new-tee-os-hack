#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // Forward the panic to sgx-libos's handler. This is due to a limitation of
    // the Rust compiler.
    sgx_libos::panic::panic_handler(info)
}

#[no_mangle]
pub extern "C" fn t_global_exit_ecall() {}

#[no_mangle]
pub extern "C" fn t_global_init_ecall(_id: u64, _path: *const u8, _len: usize) {}
