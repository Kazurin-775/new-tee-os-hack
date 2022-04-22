use hal::{
    arch::x86_vm::{gdt, vm::UserAddressSpace},
    edge::EdgeFile,
    task::{Task, TaskFuture},
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
    addr_space.alloc_map(hal::cfg::USER_STACK_TOP - 0x1000..hal::cfg::USER_STACK_TOP);

    // construct a task
    // Currently the `user_sp` is not initialized here, but rather in `ret_from_fork`.
    // This should be a bug (it prevents `fork` from being implemented properly).
    let task = Task::create(0);
    // Hack: write the entry point to rbx (used by `ret_from_fork`)
    task.lock().ktask_ctx.as_mut().unwrap().rbx = entry_point as usize;
    let task_future = TaskFuture::new(task);

    // enter user mode
    executor::spawn(task_future);
    executor::run_until_idle();
}

#[no_mangle]
unsafe extern "C" fn ret_from_fork() -> ! {
    gdt::enter_user();

    core::arch::asm!(
        // save kernel sp
        "mov    gs:[0], rsp",
        "swapgs",
        // construct an interrupt stack frame
        "push   {ss}",
        // {rsp} cannot fit into an imm32
        "mov    rax, {rsp}",
        "push   rax",
        "pushf",
        "push   {cs}",
        "push   rbx", // rip
        // return to user!
        "iretq",

        rsp = const hal::cfg::USER_STACK_TOP,
        ss = const gdt::USER_DATA_SEL.0,
        cs = const gdt::USER_CODE_SEL.0,

        options(noreturn),
    );
}
