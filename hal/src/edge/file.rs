use alloc::string::String;
use edge_proto::EdgeCallReq;

use super::{with_edge_caller, EDGE_BUFFER_SIZE};

pub struct EdgeFile {
    file_obj: u64,
}

impl EdgeFile {
    pub fn open(path: &str) -> EdgeFile {
        let file_obj = with_edge_caller(|caller| {
            caller
                .write_header(&EdgeCallReq::FileOpen {
                    path: String::from(path),
                })
                .unwrap();
            caller.kick().unwrap();

            let result = caller.read_header().unwrap();
            let result = result.into_ok_with_u64().expect("failed to open edge file");
            result
        });
        EdgeFile { file_obj }
    }

    pub fn size(&self) -> usize {
        with_edge_caller(|caller| {
            caller
                .write_header(&EdgeCallReq::FileGetSize {
                    file_obj: self.file_obj,
                })
                .unwrap();
            caller.kick().unwrap();

            let result = caller.read_header().unwrap();
            let result = result.into_ok_with_u64().expect("failed to stat edge file");
            result as usize
        })
    }

    fn read_once(&mut self, dest: &mut [u8]) -> usize {
        assert!(dest.len() <= EDGE_BUFFER_SIZE);
        with_edge_caller(|caller| {
            caller
                .write_header(&EdgeCallReq::FileRead {
                    file_obj: self.file_obj,
                    len: dest.len() as u32,
                })
                .unwrap();
            caller.kick().unwrap();

            let result = caller.read_header().unwrap();
            let len = result.into_ok_with_u64().expect("failed to read edge file") as usize;
            dest[0..len].copy_from_slice(&caller.read_data().unwrap()[0..len]);
            len
        })
    }

    pub fn read(&mut self, mut dest: &mut [u8]) -> usize {
        let mut bytes_read = 0;
        while dest.len() > EDGE_BUFFER_SIZE {
            let bytes_read_cur = self.read_once(&mut dest[0..EDGE_BUFFER_SIZE]);
            bytes_read += bytes_read_cur;
            if bytes_read_cur < EDGE_BUFFER_SIZE {
                return bytes_read_cur;
            }
            dest = &mut dest[EDGE_BUFFER_SIZE..];
        }
        bytes_read += self.read_once(dest);
        bytes_read
    }

    pub fn seek(&mut self, pos: u64) {
        with_edge_caller(|caller| {
            caller
                .write_header(&EdgeCallReq::FileSeek {
                    file_obj: self.file_obj,
                    pos,
                })
                .unwrap();
            caller.kick().unwrap();

            let result = caller.read_header().unwrap();
            assert!(result.is_ok(), "failed to seek edge file");
        });
    }

    fn close_remote_file(&self) {
        with_edge_caller(|caller| {
            caller
                .write_header(&EdgeCallReq::FileClose {
                    file_obj: self.file_obj,
                })
                .unwrap();
            caller.kick().unwrap();
        });
    }

    pub fn close(self) {
        // prevent drop handler from being called
        let guard = core::mem::ManuallyDrop::new(self);
        guard.close_remote_file();
    }
}

impl Drop for EdgeFile {
    fn drop(&mut self) {
        self.close_remote_file();
    }
}
