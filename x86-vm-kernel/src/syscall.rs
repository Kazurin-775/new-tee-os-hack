use hal::arch::x86_vm::gdt;
use linux_abi::syscall::tables::TABLE_X86_64 as SYSCALL_TABLE;
use linux_abi::syscall::SyscallHandler;
use x86_64::VirtAddr;

use crate::frame::SyscallFrame;

core::arch::global_asm!(include_str!("asm/syscall.asm"));

extern "C" {
    fn syscall_entry();
}

#[no_mangle]
unsafe extern "C" fn handle_syscall(frame: *mut SyscallFrame) {
    gdt::enter_kernel();
    let result;

    // get arguments from the frame
    let (nr, arg0, arg1, arg2, arg3, arg4, arg5) = {
        let frame = &*frame;
        (
            frame.rax, frame.rdi, frame.rsi, frame.rdx, frame.r10, frame.r8, frame.r9,
        )
    };

    // dispatch syscall by number
    let nr = nr as u32;
    match SYSCALL_TABLE.get(&nr).map(|&f| f) {
        Some(SyscallHandler::Syscall1(f)) => {
            result = f(arg0);
        }
        Some(SyscallHandler::Syscall2(f)) => {
            result = f(arg0, arg1);
        }
        Some(SyscallHandler::Syscall3(f)) => {
            result = f(arg0, arg1, arg2);
        }
        Some(SyscallHandler::Syscall4(f)) => {
            result = f(arg0, arg1, arg2, arg3);
        }
        Some(SyscallHandler::Syscall6(f)) => {
            result = f(arg0, arg1, arg2, arg3, arg4, arg5);
        }
        Some(SyscallHandler::SyscallClone(f)) => todo!(),
        None => panic!("unknown syscall number {}", nr),
    }

    (*frame).rax = result as usize;

    hal::task::yield_to_sched();

    gdt::enter_user();
}

pub fn init() {
    use x86_64::registers::model_specific as msr;

    // configure segment selectors
    msr::Star::write(
        gdt::USER_CODE_SEL,
        gdt::USER_DATA_SEL,
        gdt::KERNEL_CODE_SEL,
        gdt::KERNEL_DATA_SEL,
    )
    .unwrap();

    // configure syscall handler address
    msr::LStar::write(VirtAddr::from_ptr(syscall_entry as *const ()));

    // set IA32_EFER.SCE = 1
    unsafe {
        msr::Efer::update(|flags| {
            flags.set(msr::EferFlags::SYSTEM_CALL_EXTENSIONS, true);
        });
    }
}
