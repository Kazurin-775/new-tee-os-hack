use sgx_trts::veh::*;
use sgx_types::{int32_t, sgx_exception_info_t};

pub fn trap_handler_init() {
    match rsgx_register_exception_handler(0, handle_syscall) {
        Some(_e) => (),
        _ => panic!("fail to register syscall"),
    }
}

#[no_mangle]
extern "C" fn handle_syscall(_frame: *mut sgx_exception_info_t) -> int32_t {
    unimplemented!()
}
