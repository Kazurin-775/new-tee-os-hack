#[cfg(feature = "async")]
use alloc::boxed::Box;

use crate::{EdgeCallReq, EdgeCallResp};

#[derive(Debug)]
pub struct EdgeCallError;
pub type Result<T> = core::result::Result<T, EdgeCallError>;

pub trait EdgeCaller {
    fn write_header(&mut self, header: &EdgeCallReq) -> Result<()>;
    fn write_data(&mut self, data: &[u8]) -> Result<()>;
    fn kick(&mut self) -> Result<()>;
    fn read_header(&mut self) -> Result<EdgeCallResp>;
    fn read_data(&mut self) -> Result<&[u8]>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncEdgeCaller {
    async fn write_header(&mut self, header: &EdgeCallReq) -> Result<()>;
    async fn write_data(&mut self, data: &[u8]) -> Result<()>;
    async fn kick(&mut self) -> Result<()>;
    async fn read_header(&mut self) -> Result<EdgeCallResp>;
    async fn read_data(&mut self) -> Result<&[u8]>;
}

#[derive(Clone)]
pub struct SharedMemCaller {
    header_zone: (*mut u8, usize),
    data_zone: (*mut u8, usize),
    kick: fn() -> Result<()>,
}

unsafe impl Send for SharedMemCaller {}
unsafe impl Sync for SharedMemCaller {}

impl SharedMemCaller {
    pub const fn new(
        header_zone_ptr: *mut u8,
        header_zone_len: usize,
        data_zone_ptr: *mut u8,
        data_zone_len: usize,
        kick: fn() -> Result<()>,
    ) -> SharedMemCaller {
        SharedMemCaller {
            header_zone: (header_zone_ptr, header_zone_len),
            data_zone: (data_zone_ptr, data_zone_len),
            kick,
        }
    }

    pub fn borrow_header_zone(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.header_zone.0, self.header_zone.1) }
    }

    pub fn borrow_data_zone(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data_zone.0, self.data_zone.1) }
    }

    pub fn borrow_header_zone_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.header_zone.0, self.header_zone.1) }
    }

    pub fn borrow_data_zone_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.data_zone.0, self.data_zone.1) }
    }
}

impl EdgeCaller for SharedMemCaller {
    fn write_header(&mut self, header: &EdgeCallReq) -> Result<()> {
        postcard::to_slice(header, self.borrow_header_zone_mut()).map_err(|err| {
            log::error!("Failed to serialize edge call header: {}", err);
            EdgeCallError
        })?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<()> {
        self.borrow_data_zone_mut()[0..data.len()].copy_from_slice(data);
        Ok(())
    }

    fn kick(&mut self) -> Result<()> {
        (self.kick)()
    }

    fn read_header(&mut self) -> Result<EdgeCallResp> {
        postcard::from_bytes(self.borrow_header_zone()).map_err(|err| {
            log::error!("Failed to deserialize edge call header: {}", err);
            EdgeCallError
        })
    }

    fn read_data(&mut self) -> Result<&[u8]> {
        Ok(self.borrow_data_zone())
    }
}
