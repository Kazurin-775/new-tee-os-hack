use hal::{
    arch::x86_vm::{frame::UserspaceRegs, gdt, vm::UserAddressSpace},
    task::{Task, TaskFuture},
    vm::AddressSpace,
};
use x86_64::{PhysAddr, VirtAddr};

pub fn enter_user_mode() {
    log::debug!("Run x86-vm-init as init process");

    // get root page table
    let addr_space = UserAddressSpace::current();
    // exec x86-vm-init
    let exec_data = linux_abi::elf::exec_within(
        addr_space,
        "x86-vm-init",
        linux_abi::exec::INIT_ARGV,
        linux_abi::exec::INIT_ENVP,
        elf_loader::arch::X86_64,
        |mm, from, size, to| {
            let from = from as usize;
            for i in 0..(size + 0xFFF) >> 12 {
                mm.addr_space.map_single(
                    VirtAddr::new((to + (i << 12)) as u64),
                    PhysAddr::new(mm.addr_space.virt2phys((from + (i << 12)) as *const ()) as u64),
                );
            }
        },
    );

    // construct a task
    let userspace_regs = UserspaceRegs {
        ss: gdt::USER_DATA_SEL.0 as usize,
        rsp: exec_data.user_sp,
        rflags: x86_64::registers::rflags::read_raw() as usize,
        cs: gdt::USER_CODE_SEL.0 as usize,
        rip: exec_data.entry,
        ..Default::default()
    };
    let task = Task::create(exec_data.mm, &userspace_regs);
    let task_future = TaskFuture::new(task);

    // enter user mode
    executor::spawn(task_future);
    executor::run_until_idle();
}

#[no_mangle]
unsafe extern "C" fn load_user_ds() {
    gdt::enter_user();
}
