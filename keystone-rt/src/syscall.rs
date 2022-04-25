use core::convert::TryInto;

use crate::frame::TrapFrame;
use linux_abi::syscall::tables::TABLE_GENERIC as SYSCALL_TABLE;
use linux_abi::syscall::SyscallHandler;
use riscv::register::sepc;

pub unsafe fn handle_syscall(frame: *mut TrapFrame) {
    // get arguments from the frame
    let (nr, arg0, arg1, arg2, arg3, arg4, arg5) = {
        let frame = &*frame;
        (
            frame.a7, frame.a0, frame.a1, frame.a2, frame.a3, frame.a4, frame.a5,
        )
    };
    let nr = nr.try_into().unwrap();
    let result;

    // dispatch syscall by number
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
        Some(SyscallHandler::SyscallClone(f)) => {
            result = f(&(*frame).to_child_regs(), arg0, arg1);
        }
        None => panic!("unknown syscall number {}", nr),
    }

    // write return value back to the frame
    (*frame).a0 = result as usize;
    // move to the next instruction of `ecall`
    sepc::write(sepc::read() + 4);
}
