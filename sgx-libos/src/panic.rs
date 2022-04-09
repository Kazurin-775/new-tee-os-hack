use core::panic::PanicInfo;
use hal::println;

#[inline]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    println!("[:] {}", _info);
    loop {}
}
