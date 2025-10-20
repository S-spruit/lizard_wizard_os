#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lizard_wizard_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lizard_wizard_os::{println};
const VERSION: &str = "v0.1"; 
//test material
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    lizard_wizard_os::hlt_loop();

}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lizard_wizard_os::test_panic_handler(info)
}
//end of test functions

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Welcome to Lizard Wizard OS");
    println!("You are running version {}", VERSION);
    lizard_wizard_os::init();


    #[cfg(test)]
    test_main();

    
    lizard_wizard_os::hlt_loop();
}


