use std::{
    io::{ErrorKind, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
};

use anyhow::Context;
use edge_proto::{
    server::{EdgeStream, SharedMemEdgeStream},
    EdgeCallReq, EdgeCallResp,
};
use edge_responder::error::EdgeErrorCompat;
use nix::{
    fcntl::OFlag,
    sys::{
        mman::{MapFlags, ProtFlags},
        stat::Mode,
    },
};

pub struct EdgeCallServer {
    sock: UnixListener,
}

struct EdgeCallClient {
    socket: UnixStream,
    edge_mem: *mut u8,
    edge_stream: SharedMemEdgeStream,
}

impl EdgeCallClient {
    pub fn new(mut socket: UnixStream) -> anyhow::Result<EdgeCallClient> {
        // Read edge memory address from the guest
        let mut buf = [0; 8];
        socket
            .read_exact(&mut buf)
            .context("read edge memory address")?;
        let gpa = u64::from_le_bytes(buf);
        log::debug!("Guest reported edge memory at GPA {:#X}", gpa);

        // Map guest memory
        let ram_fd =
            nix::sys::mman::shm_open("/tee-ram", OFlag::O_RDWR, Mode::from_bits_truncate(0o644))
                .context("open /dev/shm/tee-ram")?;
        let edge_mem = unsafe {
            nix::sys::mman::mmap(
                std::ptr::null_mut(),
                0x4_000,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                ram_fd,
                gpa.try_into().unwrap(),
            )
        }
        .context("mmap /dev/shm/tee-ram")? as *mut u8;

        nix::unistd::close(ram_fd).context("close shm_fd")?;

        let edge_stream =
            SharedMemEdgeStream::new(edge_mem, 0x1_000, unsafe { edge_mem.add(0x1_000) }, 0x3_000);
        Ok(EdgeCallClient {
            socket,
            edge_mem,
            edge_stream,
        })
    }

    pub fn edge_stream(&mut self) -> &mut SharedMemEdgeStream {
        &mut self.edge_stream
    }

    pub fn wait(&mut self) -> anyhow::Result<()> {
        let mut buf = [0; 1];
        self.socket
            .read_exact(&mut buf)
            .context("read from serial")?;
        assert_eq!(buf[0], 0xCC);
        Ok(())
    }

    pub fn kick(&mut self) -> anyhow::Result<()> {
        self.socket.write_all(&[0xCC]).context("write to serial")
    }
}

impl Drop for EdgeCallClient {
    fn drop(&mut self) {
        unsafe {
            nix::sys::mman::munmap(self.edge_mem.cast(), 0x4_000).unwrap();
        }
    }
}

impl EdgeCallServer {
    pub fn new() -> anyhow::Result<EdgeCallServer> {
        match std::fs::remove_file("edge.sock") {
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            other => other.context("remove edge.sock")?,
        }
        let sock = UnixListener::bind("edge.sock").context("bind to edge.sock")?;
        Ok(EdgeCallServer { sock })
    }

    pub fn listen(&self) -> anyhow::Result<()> {
        log::info!("Listening for edge calls at edge.sock");
        let mut incoming = self.sock.incoming();
        if let Some(stream) = incoming.next() {
            let mut client = EdgeCallClient::new(stream.context("accept connection")?)
                .context("create edge call client")?;
            loop {
                client.wait().context("wait for guest")?;
                let req = client
                    .edge_stream()
                    .read_header()
                    .compat()
                    .context("read header")?;
                if req == EdgeCallReq::StreamShutdown {
                    log::info!("Edge call client signaled exit");
                    client
                        .edge_stream()
                        .write_header(&EdgeCallResp::Ok)
                        .compat()
                        .context("write header")?;
                    client.kick().context("kick guest")?;
                    break;
                }
                edge_responder::handle_edge_call_req(client.edge_stream(), req)
                    .context("handle edge call")?;
                client.kick().context("kick guest")?;
            }
        }
        Ok(())
    }
}
