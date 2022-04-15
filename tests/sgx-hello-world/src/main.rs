#![no_std]
#![no_main]

use core::cell::UnsafeCell;

core::arch::global_asm!(
    r#"
    .section .text.entry
    .global _start

_start:
    jmp     main
    "#
);

struct RacyCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for RacyCell<T> {}

const WRITE: usize = 1;
const EXIT: usize = 60;
static MSG: &str = "Hello, world!\n";
static SYSCALLER: RacyCell<usize> = RacyCell(UnsafeCell::new(0));

unsafe fn syscall(nr: usize, arg0: usize, arg1: usize, arg2: usize) {
    core::arch::asm!(
        "call {syscaller}",
        syscaller = in(reg) SYSCALLER.0.get().read(),
        inout("rax") nr => _,
        in("rdi") arg0,
        in("rsi") arg1,
        in("rdx") arg2,
        out("rcx") _,
        out("r11") _,
    );
}

#[no_mangle]
extern "C" fn main(syscaller: usize) {
    unsafe {
        SYSCALLER.0.get().write(syscaller);
        syscall(WRITE, 1, MSG.as_bytes().as_ptr() as usize, MSG.len());
        syscall(EXIT, 0, 0, 0);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("ud2", options(noreturn));
    }
}
