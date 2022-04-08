use edge_proto::{
    caller::{EdgeCallError, EdgeCaller},
    EdgeCallReq,
};
use spin::{Mutex, MutexGuard};
use uart_16550::SerialPort;

pub struct QemuEdgeCaller([u8; 0x4_000]);
pub static GLOBAL_EDGE_CALLER: Mutex<QemuEdgeCaller> = Mutex::new(QemuEdgeCaller([0; 0x4_000]));

impl EdgeCaller for QemuEdgeCaller {
    fn write_header(&mut self, header: &EdgeCallReq) -> edge_proto::caller::Result<()> {
        let data = postcard::to_slice(header, &mut self.0).map_err(|err| {
            log::error!("Failed to serialize edge call header: {}", err);
            EdgeCallError
        })?;
        serial_write_packet(&data);
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> edge_proto::caller::Result<()> {
        serial_write_packet(data);
        Ok(())
    }

    fn kick(&mut self) -> edge_proto::caller::Result<()> {
        Ok(())
    }

    fn read_header(&mut self) -> edge_proto::caller::Result<edge_proto::EdgeCallResp> {
        let data = self.read_data()?;
        postcard::from_bytes(data).map_err(|err| {
            log::error!("Failed to deserialize edge call header: {}", err);
            EdgeCallError
        })
    }

    fn read_data(&mut self) -> edge_proto::caller::Result<&[u8]> {
        let mut len = [0; 4];
        let mut port = borrow_serial_port();
        for byte in &mut len {
            *byte = port.receive();
        }
        let len = u32::from_le_bytes(len) as usize;
        for i in 0..len {
            self.0[i] = port.receive();
        }
        Ok(&self.0[0..len])
    }
}

fn borrow_serial_port() -> MutexGuard<'static, SerialPort> {
    crate::arch::x86_vm::qemu::SERIAL_EDGE.try_lock().unwrap()
}

fn serial_write_packet(data: &[u8]) {
    let mut port = borrow_serial_port();
    for byte in (data.len() as u32).to_le_bytes() {
        port.send_raw(byte);
    }
    for byte in data {
        port.send_raw(*byte);
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
