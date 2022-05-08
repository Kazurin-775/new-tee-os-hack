use edge_proto::caller::{EdgeCaller, SharedMemCaller};
use kconfig::{EDGE_BUFFER_SIZE, KERNEL_UTM_BASE};
use spin::Mutex;

use crate::arch::keystone::sbi;

static GLOBAL_EDGE_CALLER: Mutex<SharedMemCaller> = Mutex::new(SharedMemCaller::new(
    KERNEL_UTM_BASE as *mut u8,
    0x1_000,
    (KERNEL_UTM_BASE + 0x1_000) as *mut u8,
    EDGE_BUFFER_SIZE,
    edge_call_kick,
));

fn edge_call_kick() -> edge_proto::caller::Result<()> {
    sbi::stop_enclave(sbi::STOP_EDGE_CALL_HOST);
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
