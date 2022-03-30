#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

use sgx_trts::enclave;
use sgx_types::*;

use linux_abi::syscall;
use sgx_cfg::*;
use sgx_hal::EDGE_MEM_BASE;

#[no_mangle]
pub extern "C" fn rt_main(sharemem: *mut u8, memsz: usize) -> sgx_status_t {
    sgx_rt::trap::trap_handler_init();

    unsafe {
        let edge_mem = &mut EDGE_MEM_BASE;
        edge_mem.buffer = core::slice::from_raw_parts_mut(sharemem, memsz);
        edge_mem.len = memsz;

        HEAP_START = enclave::get_heap_base() as _;
        HEAP_SIZE = enclave::get_heap_size();
        elfloader::MYALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);

        //demo of ocall
        // let info="hello world";
        // let mut ptr=info.as_ptr() as usize;

        // match syscall::fs::SYSCALL_WRITE {
        //     syscall::SyscallHandler::Syscall3(f)=>{f(0,ptr,11);},
        //     _=>panic!("Not a write syscall!"),
        // }

        //demo of elf
        let elf_data = edge_mem.read_buffer();

        // let elf = elfloader::elfloader::ElfFile::load(&elf_data);
        // let entry = elf.entry() as usize;
        // let sp=elfloader::elfloader::ElfFile::prepare_libc_args();

        let s = "SGX TEE 操作系统";
        sgx_rt::uart_println!(
            "[:] {} HeapAddr: 0X{:x} ShareAddr: 0X{:x}",
            s,
            HEAP_START as usize,
            sharemem as usize,
        );

        let alloc_test = vec![1, 2, 3];
        drop(alloc_test);
    }
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn t_global_exit_ecall() {}

#[no_mangle]
pub extern "C" fn t_global_init_ecall(id: u64, path: *const u8, len: usize) {}
