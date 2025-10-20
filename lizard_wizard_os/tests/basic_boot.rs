
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lizard_wizard_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lizard_wizard_os::println;

#[unsafe(no_mangle)] 
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}



#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lizard_wizard_os::test_panic_handler(info)
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}
#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

