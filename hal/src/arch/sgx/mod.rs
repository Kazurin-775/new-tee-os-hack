use sgx_types::sgx_status_t;

pub mod frame;
pub mod vm;

pub unsafe fn initialize_edge_caller(utm_base: *mut u8) {
    crate::sys::edge::initialize_edge_caller(utm_base);
}

// OCALLS defined in `Enclave.edl`
extern "C" {
    pub fn ocall_edge_kick() -> sgx_status_t;
    pub fn ocall_exit(status: i32) -> sgx_status_t;
    pub fn ocall_switch_gs_base(old_gs_base: *mut usize, new_gs_base: usize) -> sgx_status_t;
}
