use core::panic::PanicInfo;
use crate::uart_println;

#[inline]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    uart_println!("[:] {}",_info);
    loop {}
}
