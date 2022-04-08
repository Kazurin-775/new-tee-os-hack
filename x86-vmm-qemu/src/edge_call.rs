use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
};

use anyhow::Context;
use edge_proto::{
    server::{EdgeCallError, EdgeStream},
    EdgeCallReq, EdgeCallResp,
};
use edge_responder::error::EdgeErrorCompat;

pub struct EdgeCallServer {
    sock: UnixListener,
}

struct EdgeCallClient(UnixStream, Vec<u8>);

impl EdgeStream for EdgeCallClient {
    fn read_header(&mut self) -> Result<EdgeCallReq, EdgeCallError> {
        let data = self.read_data()?;
        postcard::from_bytes(&data).map_err(|err| {
            log::error!("Failed to deserialize edge call header: {}", err);
            EdgeCallError
        })
    }

    fn read_data(&mut self) -> Result<&[u8], EdgeCallError> {
        let mut len = [0; 4];
        self.0.read_exact(&mut len).map_err(|_| EdgeCallError)?;
        let len = u32::from_le_bytes(len) as usize;
        self.1 = vec![0; len];
        self.0.read_exact(&mut self.1).map_err(|_| EdgeCallError)?;
        Ok(&self.1)
    }

    fn write_header(&mut self, header: &EdgeCallResp) -> Result<(), EdgeCallError> {
        let data = postcard::to_allocvec(header).map_err(|err| {
            log::error!("Failed to serialize edge call header: {}", err);
            EdgeCallError
        })?;
        self.0.write_all(&data).map_err(|_| EdgeCallError)
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), EdgeCallError> {
        self.0.write_all(data).map_err(|_| EdgeCallError)
    }
}

impl EdgeCallServer {
    pub fn new() -> anyhow::Result<EdgeCallServer> {
        std::fs::remove_file("edge.sock").context("remove edge.sock")?;
        let sock = UnixListener::bind("edge.sock").context("bind to edge.sock")?;
        Ok(EdgeCallServer { sock })
    }

    pub fn listen(&self) -> anyhow::Result<()> {
        log::info!("Listening for edge calls at edge.sock");
        let mut incoming = self.sock.incoming();
        if let Some(stream) = incoming.next() {
            let mut edge_stream = EdgeCallClient(stream?, Vec::new());
            loop {
                let req = edge_stream.read_header().compat()?;
                if req == EdgeCallReq::StreamShutdown {
                    log::info!("Edge call client signaled exit");
                    edge_stream.write_header(&EdgeCallResp::Ok).compat()?;
                    edge_stream.0.flush()?;
                    // trick: wait for QEMU to properly receive data
                    assert_eq!(edge_stream.0.read(&mut [0])?, 0);
                    break;
                }
                edge_responder::handle_edge_call_req(&mut edge_stream, req)
                    .context("handle edge call")?;
                edge_stream.0.flush()?;
            }
        }
        Ok(())
    }
}
