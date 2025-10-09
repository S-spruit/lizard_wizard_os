#![no_std]
#![no_main]
mod vga_buffer;
use core::panic::PanicInfo;
const VERSION: &str = "v0.1"; 

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("Welcome to Lizard Wizard OS \n").unwrap();
    write!(vga_buffer::WRITER.lock(), "You are running version {}", VERSION).unwrap();

    loop {}
}