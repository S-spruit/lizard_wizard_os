// in src/lib.rs
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::entry_point;
#[cfg(test)]
use bootloader::BootInfo;
use x86_64::instructions::port::{PortGeneric, ReadWriteAccess};
pub mod interrupts;
pub mod gdt;
pub mod serial;
pub mod memory;
pub mod vga_buffer;

pub const VERSION: &str = "v0.1.1";

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    init_rtc();
    unsafe { interrupts::PICS.lock().initialize();};
    x86_64::instructions::interrupts::enable();
}

fn init_rtc() {
    use x86_64::instructions::port::Port;
    let mut command_port: PortGeneric<u8, ReadWriteAccess> = Port::new(0x70);
    let mut data_port: PortGeneric<u8, ReadWriteAccess> = Port::new(0x71);
    unsafe {
        // DISABLE NMI
        command_port.write(0x8A as u8);

        //read register A
        command_port.write(0x0A as u8);
        let prev = data_port.read();
        command_port.write(0x0A as u8);

        //set rate to 1hz?
        data_port.write((prev & 0xF0) | 0x0F);

        command_port.write(0x8B as u8);
        let prev = data_port.read();
        command_port.write(0x8B);
        data_port.write(prev | 0x40);

        
    }
    
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}
#[cfg(test)]
entry_point!(test_kernel_main);
/// Entry point for `cargo test`
#[cfg(test)]
#[unsafe(no_mangle)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

