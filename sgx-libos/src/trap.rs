use sgx_trts::veh::*;

pub fn trap_handler_init() {
    match rsgx_register_exception_handler(0, crate::syscall::handle_syscall_exception) {
        Some(_e) => (),
        _ => panic!("failed to register syscall handler"),
    }
}
