#![no_std]

extern crate alloc;

pub mod caller;
pub mod server;

use alloc::string::String;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EdgeCallReq {
    Invalid,
    Print {
        len: u64,
    },
    SyscallWrite {
        pid: i32,
        fd: u64,
        len: u64,
    },
    SyscallMkdirAt {
        pid: i32,
        fd: i32,
        path: String,
        mode: u32,
    },
    SyscallChdir {
        pid: i32,
        path: String,
    },
    SyscallGetCwd {
        pid: i32,
    },
    FileOpen {
        path: String,
    },
    FileRead {
        file_obj: u64,
        len: u32,
    },
    FileGetSize {
        file_obj: u64,
    },
    FileSeek {
        file_obj: u64,
        pos: u64,
    },
    FileClose {
        file_obj: u64,
    },
    StreamShutdown,
}

#[derive(Serialize, Deserialize, EnumAsInner, Debug, Clone, PartialEq, Eq)]
pub enum EdgeCallResp {
    Invalid,
    SyscallResp(i64),
    Ok,
    OkWithU64(u64),
    OkWithString(String),
    Error,
}
