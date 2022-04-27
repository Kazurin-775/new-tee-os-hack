use alloc::{string::String, vec::Vec};
use hal::{
    task::UserspaceRegs,
    vm::{AddressSpace, ClonableAddressSpace, UserAddressSpace},
};
use riscv_sv39::{PhysAddr, VirtAddr};

pub fn do_execve(path: String, argv: Vec<String>, envp: Vec<String>) {
    log::debug!(
        "execve: Replacing current task with {}, argv = {:?}, envp = {:?}",
        path,
        argv,
        envp,
    );

    let addr_space = UserAddressSpace::current().create_bare();
    let exec_data = linux_abi::elf::exec_within(
        addr_space,
        &path,
        &argv,
        &envp,
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

    hal::task::current()
        .lock()
        .replace(exec_data.mm, &userspace_regs);
}
