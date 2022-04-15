#![no_std]
#![feature(alloc_error_handler)]

use hal::edge::EdgeFile;
use sgx_types::sgx_status_t;

extern crate alloc;

mod elf;
mod heap;
mod klog;
mod panic;
mod syscall;
mod trap;

pub use panic::panic_handler;

#[no_mangle]
pub extern "C" fn rt_main(utm_base: *mut u8, utm_size: usize) -> sgx_status_t {
    // Initialize edge call
    assert_eq!(utm_size, kconfig::EDGE_MEM_SIZE);
    unsafe {
        hal::arch::sgx::initialize_edge_caller(utm_base);
    }

    // Initialize trap handler
    trap::trap_handler_init();

    // Get heap information
    let (heap_base, heap_size);
    unsafe {
        heap_base = sgx_trts::enclave::get_heap_base();
        heap_size = sgx_trts::enclave::get_heap_size();
    }

    // Log system
    klog::klog_init().expect("failed to initialize klog module");

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

    log::debug!("SGX TEE OS is running!");
    log::debug!(
        "HeapAddr: {:#X}, HeapSize: {:#X}, UtmAddr: {:#X}, UtmSize: {:#X}",
        heap_base as usize,
        heap_size,
        utm_base as usize,
        utm_size,
    );

    let (rsrv_base, _rsrv_size) = heap::query_rsrv_mem();
    assert!(!rsrv_base.is_null());

    // Load sgx-init as an ELF file
    let mut edge_file = elf::EdgeElfFile(EdgeFile::open("sgx-init"));
    let elf_file = elf_loader::ElfFile::new(&mut edge_file, elf_loader::arch::X86_64);
    elf_file.load_allocated(&mut edge_file, |ptr, size| {
        let placement = ptr as usize + rsrv_base as usize;
        log::debug!(
            "ELF loader: mapping ({:?} + {:#X}) -> {:#X}",
            ptr,
            size,
            placement,
        );
        let result_addr = unsafe {
            sgx_alloc::rsrvmem::alloc_with_addr(
                placement as *mut u8,
                u32::try_from(size / kconfig::PAGE_SIZE).unwrap(),
            )
        };
        assert_eq!(result_addr as usize, placement);
        result_addr
    });

    // Call ELF's main() with the address of the syscall handler
    let elf_main: extern "C" fn(unsafe extern "C" fn()) =
        unsafe { core::mem::transmute(elf_file.entry() as usize + rsrv_base as usize) };
    elf_main(syscall::syscall_entry);

    sgx_status_t::SGX_SUCCESS
}
