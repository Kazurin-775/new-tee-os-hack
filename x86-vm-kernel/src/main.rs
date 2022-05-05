#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, panic_info_message)]

extern crate alloc;

mod exec;
mod frame;
mod heap;
mod interrupt;
mod klog;
mod panic;
mod syscall;
mod user;

use bootloader::{entry_point, BootInfo};

entry_point!(start_kernel);

fn clear_screen(boot_info: &mut BootInfo) {
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        log::debug!("Clearing VGA buffer");
        fb.buffer_mut().fill(0);
    } else {
        log::debug!("VGA buffer is not mapped");
    }
}

fn start_kernel(boot_info: &'static mut BootInfo) -> ! {
    // assert that a mirror mapping is created at `KERNEL_MIRROR_BASE`
    assert_eq!(
        Option::from(boot_info.physical_memory_offset),
        Some(hal::cfg::KERNEL_MIRROR_BASE as u64)
    );
    // Allocate edge memory from a usable memory region
    let mem_region = heap::find_usable_region(boot_info);
    unsafe {
        hal::arch::x86_vm::initialize_edge_caller(
            (mem_region.start as usize + hal::cfg::KERNEL_MIRROR_BASE) as *mut u8,
        );
    }

    klog::klog_init().unwrap();
    heap::init(
        mem_region.start as usize + hal::cfg::UTM_SIZE,
        mem_region.end as usize,
    );
    log::debug!("Edge memory and heap initialized at {:?}", mem_region);

    hal::arch::x86_vm::arch_init();
    syscall::init();
    interrupt::init();
    x86_64::instructions::interrupts::enable();

    clear_screen(boot_info);
    log::info!("It didn't crash!");

    user::enter_user_mode();

    log::debug!("All kernel tasks have exited");
    hal::exit_enclave(0);
    unreachable!()
}
