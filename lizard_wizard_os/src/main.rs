#![no_std]
#![no_main]
mod vga_buffer;
use core::panic::PanicInfo;
const VERSION: &str = "v0.1"; 

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Welcome to Lizard Wizard OS");
    println!("You are running version {}", VERSION);
    panic!("ohw no :( it seems like we encountered a problem!");
    loop {}
}