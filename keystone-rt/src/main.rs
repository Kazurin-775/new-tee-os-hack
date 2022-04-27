#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use hal::{
    arch::keystone::{frame::UserspaceRegs, vm::UserAddressSpace},
    task::{Task, TaskFuture},
    vm::AddressSpace,
};
use kmalloc::{Kmalloc, LockedLinkedListHeap};
use riscv_sv39::{PhysAddr, VirtAddr};

mod entry;
mod exec;
mod frame;
mod klog;
mod panic;
mod syscall;
mod trap;
mod uart;
mod vm;

#[global_allocator]
static ALLOC: LockedLinkedListHeap = unsafe { LockedLinkedListHeap::uninit() };

#[no_mangle]
extern "C" fn rt_main(vm_info: &vm::VmInfo) -> ! {
    // initialize EPM_PHYS
    hal::arch::keystone::EPM_PHYS.call_once(|| vm_info.epm_base);
    // initialize modules
    klog::klog_init().expect("failed to initialize klog module");
    unsafe {
        ALLOC.init(vm_info.free_virt as *mut u8, vm_info.free_size);
    }
    log::debug!("It did not crash!");

    // load U-mode program
    log::debug!("Run keystone-init as init process");
    let addr_space = UserAddressSpace::current();
    let exec_data = linux_abi::elf::exec_within(
        addr_space,
        "keystone-init",
        linux_abi::exec::INIT_ARGV,
        linux_abi::exec::INIT_ENVP,
        elf_loader::arch::RiscV,
        |mm, from, size, to| {
            let from = from as usize;
            for i in 0..(size + 0xFFF) >> 12 {
                mm.addr_space.map_single(
                    VirtAddr(to + (i << 12)),
                    PhysAddr(mm.addr_space.virt2phys((from + (i << 12)) as *const ())),
                );
            }
        },
    );

    let userspace_regs = UserspaceRegs {
        sp: exec_data.user_sp,
        sepc: exec_data.entry,
        ..Default::default()
    };
    let task = Task::create(exec_data.mm, &userspace_regs);
    let task_future = TaskFuture::new(task);

    // execute U-mode program
    unsafe {
        // this should only be needed to be set once if no traps occur
        // in S-mode
        riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
    }
    executor::spawn(task_future);
    executor::run_until_idle();

    log::debug!("All kernel tasks have exited");
    hal::exit_enclave(0);
    unreachable!()
}
