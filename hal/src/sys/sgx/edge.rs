use edge_proto::caller::{EdgeCaller, SharedMemCaller};
use spin::Mutex;

// Null initialized
static GLOBAL_EDGE_CALLER: Mutex<SharedMemCaller> = Mutex::new(SharedMemCaller::new(
    core::ptr::null_mut(),
    0,
    core::ptr::null_mut(),
    0,
    edge_call_kick,
));

pub unsafe fn initialize_edge_caller(utm_base: *mut u8) {
    *GLOBAL_EDGE_CALLER.try_lock().unwrap() = SharedMemCaller::new(
        utm_base,
        0x1_000,
        utm_base.add(0x1_000),
        kconfig::EDGE_BUFFER_SIZE,
        edge_call_kick,
    );
}

fn edge_call_kick() -> edge_proto::caller::Result<()> {
    let result = unsafe { crate::arch::sgx::ocall_edge_kick() };
    assert_eq!(result, sgx_types::sgx_status_t::SGX_SUCCESS);
    Ok(())
}

pub fn with_edge_caller_impl<F, R>(f: F) -> R
where
    F: FnOnce(&mut dyn EdgeCaller) -> R,
{
    f(&mut *GLOBAL_EDGE_CALLER
        .try_lock()
        .expect("the edge caller is not reentrant"))
}
