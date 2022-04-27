use core::convert::TryInto;
use linux_abi::syscall::tables::TABLE_X86_64 as SYSCALL_TABLE;
use linux_abi::syscall::SyscallHandler;
use sgx_types::{int32_t, sgx_exception_info_t};

core::arch::global_asm!(include_str!("asm/syscall.asm"));

#[no_mangle]
pub unsafe extern "C" fn handle_syscall(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    nr: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    let result;

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
        Some(SyscallHandler::SyscallClone(_f)) => {
            panic!("clone() is not supported on SGX");
        }
        Some(SyscallHandler::SyscallExecvePre(_f)) => {
            panic!("execve() is not supported on SGX");
        }
        None => panic!("unknown syscall number {}", nr),
    }

    hal::task::yield_to_sched();

    result
}

#[no_mangle]
pub extern "C" fn handle_syscall_exception(frame: *mut sgx_exception_info_t) -> int32_t {
    // get arguments from the frame
    let (nr, arg0, arg1, arg2, arg3, arg4, arg5) = {
        let frame = unsafe { &*frame };
        (
            frame.cpu_context.rax as usize,
            frame.cpu_context.rdi as usize,
            frame.cpu_context.rsi as usize,
            frame.cpu_context.rdx as usize,
            frame.cpu_context.r10 as usize,
            frame.cpu_context.r8 as usize,
            frame.cpu_context.r9 as usize,
        )
    };
    let nr = nr.try_into().unwrap();
    let result = unsafe { handle_syscall(arg0, arg1, arg2, nr, arg3, arg4, arg5) };

    unsafe {
        // write return value back to the frame
        (*frame).cpu_context.rax = result as _;
        // move to the next instruction of `int x80` ,which is required by the SGX Guide Ref
        //@?+4/+8?
        (*frame).cpu_context.rip = (*frame).cpu_context.rip + 2;
    }

    0
}
