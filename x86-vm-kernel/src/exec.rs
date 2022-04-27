use alloc::string::String;
use hal::{
    arch::x86_vm::gdt,
    task::UserspaceRegs,
    vm::{AddressSpace, ClonableAddressSpace, UserAddressSpace},
};
use x86_64::{PhysAddr, VirtAddr};

pub fn do_execve(path: String) {
    log::debug!("execve: Replacing current task with {}", path);

    let addr_space = UserAddressSpace::current().create_bare();
    let exec_data = linux_abi::elf::exec_within(
        addr_space,
        &path,
        linux_abi::exec::INIT_ARGV, // TODO
        linux_abi::exec::INIT_ENVP, // TODO
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

    let userspace_regs = UserspaceRegs {
        ss: gdt::USER_DATA_SEL.0 as usize,
        rsp: exec_data.user_sp,
        rflags: x86_64::registers::rflags::read_raw() as usize,
        cs: gdt::USER_CODE_SEL.0 as usize,
        rip: exec_data.entry,
        ..Default::default()
    };

    hal::task::current()
        .lock()
        .replace(exec_data.mm, &userspace_regs);
}
