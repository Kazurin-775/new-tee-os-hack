#![no_std]
#![feature(alloc_error_handler)]

use alloc::vec;
use hal::println;
use sgx_trts::enclave;
use sgx_types::sgx_status_t;

extern crate alloc;

// pub mod syscall;
mod heap;
pub mod panic;
mod trap;

#[no_mangle]
pub extern "C" fn rt_main(utm_base: *mut u8, utm_size: usize) -> sgx_status_t {
    // Initialize edge call
    assert_eq!(utm_size, kconfig::EDGE_MEM_SIZE);
    unsafe {
        hal::arch::sgx::initialize_edge_caller(utm_base);
    }

    // Initialize trap handler
    // FIXME: can't this be put off until the heap is initialized?
    trap::trap_handler_init();

    // Initialize heap
    let (heap_base, heap_size);
    unsafe {
        heap_base = enclave::get_heap_base() as _;
        heap_size = enclave::get_heap_size();
        heap::ALLOCATOR.lock().init(heap_base, heap_size);
    }

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

    println!("SGX TEE OS is running!");
    println!(
        "HeapAddr: {:#X}, HeapSize: {:#X}, UtmAddr: {:#X}, UtmSize: {:#X}",
        heap_base as usize, heap_size, utm_base as usize, utm_size,
    );

    let alloc_test = vec![1, 2, 3];
    drop(alloc_test);

    sgx_status_t::SGX_SUCCESS
}
