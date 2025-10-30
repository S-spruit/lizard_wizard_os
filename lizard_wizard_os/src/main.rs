#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lizard_wizard_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{
    BootInfo,
    entry_point
};
use core::panic::PanicInfo;
use lizard_wizard_os::{
    memory::{self, BootInfoFrameAllocator},
    println
};
use x86_64::{
    structures::paging::Page,
    VirtAddr
};
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
entry_point!(kernel_main);

#[unsafe(no_mangle)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!(" Welcome to Lizard Wizard OS ");
    println!(" You are running version {} ", lizard_wizard_os::VERSION);
    lizard_wizard_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut _mapper = unsafe { memory::init(phys_mem_offset) };
    let mut _frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // // map an unused page
    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // // write the string `New!` to the screen through the new mapping
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    #[cfg(test)]
    test_main();

    lizard_wizard_os::hlt_loop();
}

