#![no_std]
#![feature(alloc_error_handler)]

use hal::{
    arch::sgx::{frame::UserspaceRegs, vm::UserAddressSpace},
    edge::EdgeFile,
    task::{Task, TaskFuture, TaskMmStruct},
    vm::AddressSpace,
};
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

    // Allocate user stack
    let user_stack_begin = unsafe {
        alloc::alloc::alloc(alloc::alloc::Layout::from_size_align_unchecked(
            0x4000, 0x1000,
        ))
    };
    assert!(!user_stack_begin.is_null());
    log::debug!("Allocated user stack at {:?}", user_stack_begin);

    // Switch to user context and call ELF's main()
    let elf_main = elf_file.entry() as usize + rsrv_base as usize;
    let addr_space = UserAddressSpace::current();
    let user_stack_begin = user_stack_begin as usize;
    let mut mm = TaskMmStruct::new(addr_space, user_stack_begin..user_stack_begin + 0x4000);
    let task = Task::create(
        mm,
        &UserspaceRegs {
            rsp: user_stack_begin + 0x4000,
            rip: elf_main,
        },
    );

    let task_future = TaskFuture::new(task);
    executor::spawn(task_future);
    executor::run_until_idle();

    log::debug!("All kernel tasks have exited");
    sgx_status_t::SGX_SUCCESS
}
