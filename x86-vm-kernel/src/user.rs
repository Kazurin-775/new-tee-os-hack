use alloc::sync::Arc;
use hal::{
    arch::x86_vm::gdt,
    edge::EdgeFile,
    task::{Task, TaskFuture},
};
use spin::Mutex;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::{elf::EdgeElfFile, memory::MIRROR_BASE_VIRT};

pub fn enter_user_mode() {
    // get root page table
    let rpt_phys = x86_64::registers::control::Cr3::read().0;
    let rpt_ptr = (MIRROR_BASE_VIRT + rpt_phys.start_address().as_u64()).as_mut_ptr();
    let mut rpt = unsafe { OffsetPageTable::new(&mut *rpt_ptr, MIRROR_BASE_VIRT) };
    let mut frame_allocator = crate::memory::HeapFrameAlloc;

    // load init ELF file
    let entry_point;
    {
        let mut edge_file = EdgeElfFile(EdgeFile::open("x86-vm-init"));
        let elf_file = elf_loader::ElfFile::new(&mut edge_file, elf_loader::arch::X86_64);
        elf_file.load_mapped(&mut edge_file, |from, size, to| unsafe {
            log::debug!(
                "ELF loader: mapping ({:?} + {:#X}) -> {:#X}",
                from,
                size,
                to,
            );
            let from = from as usize;
            for i in 0..(size + 0xFFF) >> 12 {
                rpt.map_to(
                    Page::<Size4KiB>::from_start_address(VirtAddr::new((to + (i << 12)) as u64))
                        .unwrap(),
                    PhysFrame::from_start_address(PhysAddr::new(
                        (from + (i << 12)) as u64 - MIRROR_BASE_VIRT.as_u64(),
                    ))
                    .unwrap(),
                    PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE,
                    &mut frame_allocator,
                )
                .unwrap()
                .flush();
            }
        });
        entry_point = elf_file.entry();
        edge_file.0.close();
    }

    // allocate pages for user stack
    unsafe {
        rpt.map_to(
            Page::from_start_address(VirtAddr::new(hal::cfg::USER_STACK_TOP as u64 - 0x1000))
                .unwrap(),
            frame_allocator.allocate_frame().unwrap(),
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::NO_EXECUTE,
            &mut frame_allocator,
        )
        .unwrap()
        .flush();
    }

    // construct a task
    let mut task = Task::create(0);
    task.ktask_ctx.as_mut().unwrap().rbx = entry_point as usize;
    let task_future = TaskFuture::new(Arc::new(Mutex::new(task)));

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
