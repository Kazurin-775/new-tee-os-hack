use hal::{
    arch::x86_vm::{frame::UserspaceRegs, gdt, vm::UserAddressSpace},
    edge::EdgeFile,
    task::{Task, TaskFuture, TaskMmStruct},
    vm::AddressSpace,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::elf::EdgeElfFile;

pub fn enter_user_mode() {
    // get root page table
    let mut addr_space = UserAddressSpace::current();

    // load init ELF file
    let entry_point;
    {
        let mut edge_file = EdgeElfFile(EdgeFile::open("x86-vm-init"));
        let elf_file = elf_loader::ElfFile::new(&mut edge_file, elf_loader::arch::X86_64);
        elf_file.load_mapped(&mut edge_file, |from, size, to| {
            log::debug!(
                "ELF loader: mapping ({:?} + {:#X}) -> {:#X}",
                from,
                size,
                to,
            );
            let from = from as usize;
            for i in 0..(size + 0xFFF) >> 12 {
                addr_space.map_single(
                    VirtAddr::new((to + (i << 12)) as u64),
                    PhysAddr::new(addr_space.virt2phys((from + (i << 12)) as *const ()) as u64),
                );
            }
        });
        entry_point = elf_file.entry();
        edge_file.0.close();
    }

    // allocate pages for user stack
    addr_space.alloc_map(hal::cfg::USER_STACK_END - 0x1000..hal::cfg::USER_STACK_END);

    // construct a task
    let mm = TaskMmStruct::new(
        addr_space,
        hal::cfg::USER_STACK_END - 0x1_000..hal::cfg::USER_STACK_END,
    );
    let userspace_regs = UserspaceRegs {
        ss: gdt::USER_DATA_SEL.0 as usize,
        rsp: hal::cfg::USER_STACK_END,
        rflags: x86_64::registers::rflags::read_raw() as usize,
        cs: gdt::USER_CODE_SEL.0 as usize,
        rip: entry_point as usize,
        ..Default::default()
    };
    let task = Task::create(mm, &userspace_regs);
    let task_future = TaskFuture::new(task);

    // enter user mode
    executor::spawn(task_future);
    executor::run_until_idle();
}

#[no_mangle]
unsafe extern "C" fn load_user_ds() {
    gdt::enter_user();
}
