use sgx_trts::veh::*;
use sgx_types::{int32_t, sgx_exception_info_t, sgx_exception_vector_t};

pub fn trap_handler_init() {
    match rsgx_register_exception_handler(0, handle_exception) {
        Some(_e) => (),
        _ => panic!("failed to register syscall handler"),
    }
}

#[no_mangle]
extern "C" fn handle_exception(frame: *mut sgx_exception_info_t) -> int32_t {
    // This function only supports handling `sysenter` instructions.
    // For all other kinds of exceptions, it will cause an abort.
    // Unregister this handler to let the debugger handle the exception.

    if unsafe { (*frame).exception_vector } != sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_UD {
        unsafe {
            sgx_trts::libc::abort();
        }
    }

    let rip = unsafe { (*frame).cpu_context.rip };
    const SYSCALL_OPCODE: u16 = u16::from_ne_bytes([0x0F, 0x05]);
    if unsafe { (rip as *const u16).read() != SYSCALL_OPCODE } {
        unsafe {
            sgx_trts::libc::abort();
        }
    }

    crate::syscall::handle_syscall_exception(frame)
}
