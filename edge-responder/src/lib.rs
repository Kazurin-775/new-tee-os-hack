use anyhow::Context;
use edge_proto::{server::EdgeStream, EdgeCallReq, EdgeCallResp};
use error::{SyscallError, SyscallResult};

use crate::error::EdgeErrorCompat;

mod edge_file;
pub mod error;
mod fs_imp;
mod pcb;
mod syscall_imp;

pub fn handle_edge_call(stream: &mut dyn EdgeStream) -> anyhow::Result<()> {
    let header = stream.read_header().compat().context("read header")?;
    handle_edge_call_req(stream, header)
}

pub fn handle_edge_call_req(
    stream: &mut dyn EdgeStream,
    header: EdgeCallReq,
) -> anyhow::Result<()> {
    use edge_proto::EdgeCallReq::*;

    match header {
        Print { len } => {
            let data = &stream.read_data().compat().context("read data")?[0..len as usize];
            let text = std::str::from_utf8(data)
                .expect("the enclave tries to print an invalid UTF-8 string");
            print!("{}", text);
            stream
                .write_header(&EdgeCallResp::Ok)
                .compat()
                .context("write header")?;
        }
        SyscallWrite { pid, fd, len } => {
            let result = syscall_imp::write(stream, pid, fd, len);
            write_syscall_result(stream, result).context("write result")?;
        }
        FileOpen { path } => {
            write_anyhow_result(
                stream,
                edge_file::edge_open(&path).map(EdgeCallResp::OkWithU64),
            )?;
        }
        FileRead { file_obj, len } => {
            let mut buf = vec![0; len as usize];
            write_anyhow_result(
                stream,
                edge_file::edge_read(file_obj, &mut buf)
                    .map(|len| EdgeCallResp::OkWithU64(len as u64)),
            )?;
            stream.write_data(&buf).compat().context("write data")?;
        }
        FileGetSize { file_obj } => {
            write_anyhow_result(
                stream,
                edge_file::edge_get_size(file_obj).map(EdgeCallResp::OkWithU64),
            )?;
        }
        FileSeek { file_obj, pos } => {
            write_anyhow_result(
                stream,
                edge_file::edge_seek(file_obj, pos).map(|()| EdgeCallResp::Ok),
            )?;
        }
        FileClose { file_obj } => {
            edge_file::edge_close(file_obj);
            stream
                .write_header(&EdgeCallResp::Ok)
                .compat()
                .context("write header")?;
        }
        other @ (Invalid | StreamShutdown) => {
            log::warn!("Invalid edge call {:?}, ignoring", other);
        }
    }

    Ok(())
}

fn write_syscall_result(
    stream: &mut dyn EdgeStream,
    result: SyscallResult<isize>,
) -> anyhow::Result<()> {
    let result_as_isize = match result {
        Ok(r) => r,
        Err(SyscallError::Linux(errno, None)) => errno as isize,
        Err(SyscallError::Linux(errno, Some(err))) => {
            log::warn!("Error handling syscall: {:#}", err);
            errno as isize
        }
        Err(SyscallError::Internal(err)) => {
            return Err(err).context("error handling syscall");
        }
    };
    stream
        .write_header(&EdgeCallResp::SyscallResp(result_as_isize as i64))
        .compat()
}

fn write_anyhow_result(
    stream: &mut dyn EdgeStream,
    result: anyhow::Result<EdgeCallResp>,
) -> anyhow::Result<()> {
    stream
        .write_header(&match result {
            Ok(header) => header,
            Err(err) => {
                log::error!("Failed to perform edge call: {:#}", err);
                EdgeCallResp::Error
            }
        })
        .compat()
}
