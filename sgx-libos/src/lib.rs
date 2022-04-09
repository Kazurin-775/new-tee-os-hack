#![no_std]
#![feature(alloc_error_handler)]

use alloc::vec;
use sgx_trts::enclave;
use sgx_types::sgx_status_t;

extern crate alloc;

use spin::Once;

// pub mod syscall;
mod trap;
#[macro_use]
mod uart;
mod heap;
pub mod panic;

static EDGE_BUF: Once<(usize, usize)> = Once::new();

#[no_mangle]
pub extern "C" fn rt_main(utm_base: *mut u8, utm_size: usize) -> sgx_status_t {
    EDGE_BUF.call_once(|| (utm_base as usize, utm_size));
    trap::trap_handler_init();

    unsafe {
        let heap_base = enclave::get_heap_base() as _;
        let heap_size = enclave::get_heap_size();
        heap::ALLOCATOR.lock().init(heap_base, heap_size);

        //demo of ocall
        // let info="hello world";
        // let mut ptr=info.as_ptr() as usize;

        // match syscall::fs::SYSCALL_WRITE {
        //     syscall::SyscallHandler::Syscall3(f)=>{f(0,ptr,11);},
        //     _=>panic!("Not a write syscall!"),
        // }

        //demo of elf
        // let elf = elfloader::elfloader::ElfFile::load(&elf_data);
        // let entry = elf.entry() as usize;
        // let sp=elfloader::elfloader::ElfFile::prepare_libc_args();

        uart_println!("SGX TEE OS is running!");
        uart_println!(
            "HeapAddr: {:#X}, HeapSize: {:#X}, UtmAddr: {:#X}, UtmSize: {:#X}",
            heap_base as usize,
            heap_size,
            utm_base as usize,
            utm_size,
        );

        let alloc_test = vec![1, 2, 3];
        drop(alloc_test);
    }
    sgx_status_t::SGX_SUCCESS
}
