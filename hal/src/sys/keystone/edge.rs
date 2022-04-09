use edge_proto::caller::{EdgeCaller, SharedMemCaller};
use spin::Mutex;

use crate::{arch::keystone::sbi, cfg::KERNEL_UTM_BASE};

static GLOBAL_EDGE_CALLER: Mutex<SharedMemCaller> = Mutex::new(SharedMemCaller::new(
    KERNEL_UTM_BASE as *mut u8,
    0x1_000,
    (KERNEL_UTM_BASE + 0x1000) as *mut u8,
    0x3_000,
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
