use edge_proto::caller::{EdgeCaller, SharedMemCaller};
use kconfig::KERNEL_MIRROR_BASE;
use spin::{Mutex, MutexGuard};
use uart_16550::SerialPort;

// Null initialized
static GLOBAL_EDGE_CALLER: Mutex<SharedMemCaller> = Mutex::new(SharedMemCaller::new(
    core::ptr::null_mut(),
    0,
    core::ptr::null_mut(),
    0,
    edge_call_kick,
));

fn borrow_serial_port() -> MutexGuard<'static, SerialPort> {
    crate::arch::x86_vm::qemu::SERIAL_EDGE.try_lock().unwrap()
}

fn edge_call_kick() -> edge_proto::caller::Result<()> {
    let mut serial = borrow_serial_port();
    serial.send_raw(0xCC);
    assert_eq!(serial.receive(), 0xCC);
    Ok(())
}

pub unsafe fn initialize_edge_caller(edge_mem: *mut u8) {
    let mut caller = GLOBAL_EDGE_CALLER.try_lock().unwrap();
    // Initialize SharedMemCaller
    *caller = SharedMemCaller::new(
        edge_mem,
        0x1_000,
        edge_mem.add(0x1_000),
        0x3_000,
        edge_call_kick,
    );

    let mut serial = borrow_serial_port();
    // Write DMA address to the serial port
    for byte in (edge_mem as u64 - KERNEL_MIRROR_BASE as u64).to_le_bytes() {
        serial.send_raw(byte);
    }
}

pub fn with_edge_caller_impl<F, R>(f: F) -> R
where
    F: FnOnce(&mut dyn EdgeCaller) -> R,
{
    f(&mut *GLOBAL_EDGE_CALLER
        .try_lock()
        .expect("the edge caller is not reentrant"))
}
