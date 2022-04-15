use sgx_types::sgx_status_t;

#[global_allocator]
static ALLOCATOR: sgx_alloc::System = sgx_alloc::System;

#[alloc_error_handler]
fn default_handler(layout: core::alloc::Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size())
}

pub fn query_rsrv_mem() -> (*mut u8, usize) {
    extern "C" {
        fn sgx_get_rsrv_mem_info(addr: *mut *mut u8, max_size: *mut usize) -> sgx_status_t;
    }

    let (mut addr, mut size) = (0 as _, 0);
    let ret = unsafe { sgx_get_rsrv_mem_info(&mut addr, &mut size) };
    if ret != sgx_status_t::SGX_SUCCESS {
        panic!("sgx_get_rsrv_mem_info failed, ret = {}", ret);
    }
    log::debug!("SGX reserved memory addr = {:?}, size = {:#X}", addr, size);
    (addr, size)
}
