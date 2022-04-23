#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use hal::{
    arch::keystone::vm::UserAddressSpace,
    edge::EdgeFile,
    task::{Task, TaskFuture, TaskMmStruct, VmArea},
    vm::AddressSpace,
};
use kmalloc::{Kmalloc, LockedLinkedListHeap};
use log::debug;
use riscv_sv39::{PhysAddr, VirtAddr};

mod elf;
mod entry;
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
    let entry;
    let addr_space = UserAddressSpace::current();
    let mut mm = TaskMmStruct::new(
        addr_space,
        hal::cfg::USER_STACK_END - 0x1_000..hal::cfg::USER_STACK_END,
    );
    {
        // open ELF file
        let mut elf_file = elf::EdgeElfFile(EdgeFile::open("keystone-init"));

        // load & map ELF file
        let elf = elf_loader::ElfFile::new(&mut elf_file, elf_loader::arch::RiscV);
        elf.load_mapped(&mut elf_file, |from, size, to| {
            debug!(
                "ELF loader: mapping ({:?} + {:#X}) -> {:#X}",
                from, size, to
            );
            let from = from as usize;
            for i in 0..(size + 0xFFF) >> 12 {
                mm.addr_space.map_single(
                    VirtAddr(to + (i << 12)),
                    PhysAddr(mm.addr_space.virt2phys((from + (i << 12)) as *const ())),
                );
            }
            // map the section to TaskMmStruct
            // TODO: create an abstraction
            mm.vmas.insert(
                to,
                VmArea {
                    range: to..to + ((size + 0xFFF) & !0xFFF),
                },
            );
        });
        entry = elf.entry() as usize;

        // map an extra page for the initial stack
        // TODO: this should be done by TaskMmStruct
        mm.addr_space
            .alloc_map(hal::cfg::USER_STACK_END - 0x1_000..hal::cfg::USER_STACK_END);
    }

    // Copy argv and envp to the user stack's end
    let (user_stack_data, user_sp) = linux_abi::exec::prepare_user_stack_data(
        hal::cfg::USER_STACK_END,
        &linux_abi::exec::INIT_ARGV,
        &linux_abi::exec::INIT_ENVP,
    );
    assert!(user_stack_data.len() < 0x1_000);
    unsafe {
        hal::mem::copy_to_user(&user_stack_data, user_sp as *mut u8);
    }

    log::debug!("Run keystone-init as init process");
    let task = Task::create(mm, user_sp);
    let task_future = TaskFuture::new(task);

    // execute U-mode program
    unsafe {
        riscv::register::sepc::write(entry);
        riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
    }
    executor::spawn(task_future);
    executor::run_until_idle();

    log::debug!("All kernel tasks have exited");
    hal::exit_enclave(0);
    unreachable!()
}
