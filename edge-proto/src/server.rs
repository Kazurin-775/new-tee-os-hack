#[cfg(feature = "async")]
use alloc::boxed::Box;

use crate::{EdgeCallReq, EdgeCallResp};

#[derive(Debug)]
pub struct EdgeCallError;
pub type Result<T> = core::result::Result<T, EdgeCallError>;

pub trait EdgeStream {
    fn read_header(&mut self) -> Result<EdgeCallReq>;
    fn read_data(&mut self) -> Result<&[u8]>;
    fn write_header(&mut self, header: &EdgeCallResp) -> Result<()>;
    fn write_data(&mut self, data: &[u8]) -> Result<()>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncEdgeStream {
    async fn read_header(&mut self) -> Result<EdgeCallReq>;
    async fn read_data(&mut self) -> Result<&[u8]>;
    async fn write_header(&mut self, header: &EdgeCallResp) -> Result<()>;
    async fn write_data(&mut self, data: &[u8]) -> Result<()>;
}

#[derive(Clone)]
pub struct SharedMemEdgeStream {
    header_zone: (*mut u8, usize),
    data_zone: (*mut u8, usize),
}

unsafe impl Send for SharedMemEdgeStream {}
unsafe impl Sync for SharedMemEdgeStream {}

impl SharedMemEdgeStream {
    pub const fn new(
        header_zone_ptr: *mut u8,
        header_zone_len: usize,
        data_zone_ptr: *mut u8,
        data_zone_len: usize,
    ) -> SharedMemEdgeStream {
        SharedMemEdgeStream {
            header_zone: (header_zone_ptr, header_zone_len),
            data_zone: (data_zone_ptr, data_zone_len),
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

impl EdgeStream for SharedMemEdgeStream {
    fn read_header(&mut self) -> Result<EdgeCallReq> {
        postcard::from_bytes(self.borrow_header_zone()).map_err(|err| {
            log::error!("Failed to deserialize edge call header: {}", err);
            EdgeCallError
        })
    }

    fn read_data(&mut self) -> Result<&[u8]> {
        Ok(self.borrow_data_zone())
    }

    fn write_header(&mut self, header: &EdgeCallResp) -> Result<()> {
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
}
