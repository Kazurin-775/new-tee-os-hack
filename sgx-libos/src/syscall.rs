use core::convert::TryInto;
use linux_abi::syscall::tables::TABLE_X86_64 as SYSCALL_TABLE;
use linux_abi::syscall::SyscallHandler;
use sgx_types::{int32_t, sgx_exception_info_t};

core::arch::global_asm!(include_str!("asm/syscall.asm"));

extern "C" {
    pub fn syscall_entry();
}

#[no_mangle]
pub unsafe extern "C" fn handle_syscall(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    nr: usize,
    arg3: usize,
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
        None => panic!("unknown syscall number {}", nr),
    }

    result
}

#[no_mangle]
pub extern "C" fn handle_syscall_exception(frame: *mut sgx_exception_info_t) -> int32_t {
    // get arguments from the frame
    let (nr, arg0, arg1, arg2, arg3) = {
        let frame = unsafe { &*frame };
        (
            frame.cpu_context.rax as usize,
            frame.cpu_context.rdi as usize,
            frame.cpu_context.rsi as usize,
            frame.cpu_context.rdx as usize,
            frame.cpu_context.r10 as usize,
        )
    };
    let nr = nr.try_into().unwrap();
    let result = unsafe { handle_syscall(arg0, arg1, arg2, nr, arg3) };

    unsafe {
        // write return value back to the frame
        (*frame).cpu_context.rax = result as _;
        // move to the next instruction of `int x80` ,which is required by the SGX Guide Ref
        //@?+4/+8?
        (*frame).cpu_context.rip = (*frame).cpu_context.rip + 2;
    }

    //TODO
    return 0;
    // unsafe{
    //    there are no iret instructions,and the sgx exception mechanism
    //    ensures that the cpu context will be restored to the SSA
    //    rsp--sp rbp--frame rip--pc
    //     asm!(
    //         "mov r12, return":"m{return}"(ret):::"intel",
    //         "mov r13, return":"m{return}"((*frame).cpu_context.r13):::"intel",
    //         "mov r14, return":"m{return}"((*frame).cpu_context.r14):::"intel",
    //         "mov r15, return":"m{return}"((*frame).cpu_context.r15):::"intel",
    //         "mov rbx, return":"m{return}"((*frame).cpu_context.rbx):::"intel",
    //         "mov rsi, return":"m{return}"((*frame).cpu_context.rsi):::"intel",
    //         "mov rdi, return":"m{return}"((*frame).cpu_context.rdi):::"intel",
    //         "mov r8, return":"r{return}"(ret):::"intel",
    //         "mov rbp, return":"m{return}"((*frame).cpu_context.rbp):::"intel",
    //         "mov rsp, return":"m{return}"((*frame).cpu_context.rsp):::"intel",
    //         "mov rip, r8",

    //     );
    // }
}
