use pc_keyboard::KeyCode;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::{PortGeneric, ReadWriteAccess};
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::vga_buffer::WRITER;
use crate::{VERSION, hlt_loop, print};
use crate::println;
use crate::gdt;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,

    RealTimeClock = PIC_2_OFFSET
}
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}




lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::RealTimeClock.as_usize()].set_handler_fn(rtc_interrupt_handler);
        idt
    };
}
pub fn init_idt() {
    IDT.load();  
    
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) {
    println!("BREAKPOINT EXCEPTION: \n {:#?}", stack_frame)
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;
    println!("PAGE FAULT EXCEPTION:");
    println!("| Accessed address: {:?}", Cr2::read());
    println!("| Error code: {:?}", error_code);
    println!("|____________________________________|\n {:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    

    unsafe {
        PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    lazy_static!{
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));
    }
    let mut keyboard = KEYBOARD.lock();

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    match character {
                        '\x08' => WRITER.lock().clear_char(),
                        // '\n' => WRITER.lock(),
                        _ => print!("{}", character)
                    }
                },
                DecodedKey::RawKey(key) => {
                    match key {
                        KeyCode::Backspace => {
                            WRITER.lock().clear_char();
                        },
                        _ => print!("{:?}", key)
                    }
                }, //this is where the match will come later to act accordingly when doing enter, backspace, del, etc.
            }
        }
    }
    unsafe {
        PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn rtc_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut command_port: PortGeneric<u8, ReadWriteAccess> = Port::new(0x70);
    let mut data_port: PortGeneric<u8, ReadWriteAccess> = Port::new(0x71);

    let mut seconds: u8 = 0;
    let mut minutes: u8 = 0;
    let mut hours: u8 = 0;
    //https://wiki.osdev.org/RTC

    unsafe {
        // interrupts::disable();

        command_port.write(0x00);
        seconds = data_port.read();

        command_port.write(0x02);
        minutes = data_port.read();

        command_port.write(0x04);
        hours = data_port.read();

        let reg_b = unsafe {
        command_port.write(0x0B);
        data_port.read()
        };

        let is_bcd = (reg_b & 0x04) == 0; // Bit 2 clear usually means BCD format
    
        if is_bcd {
        // You MUST convert if in BCD format
            seconds = bcd_to_bin(seconds);
            minutes = bcd_to_bin(minutes);
            hours = bcd_to_bin(hours);
        }

        let mut buffer = [0u8; 80]; 
        let mut writer = ArrayWriter::new(&mut buffer);
        let _ = write!(writer, "LizWizOS {} ------------------------------------------------------- {:02}:{:02}:{:02}", VERSION, hours, minutes,seconds);
        let statusbar = writer.as_str().unwrap_or("invalid status bar");
        WRITER.lock().write_status(statusbar);

        

        
        
        command_port.write(0x0C as u8);
        data_port.read();

        // interrupts::enable();
    }
    
    

    unsafe {
        PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::RealTimeClock.as_u8());
    }
}
fn bcd_to_bin(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

// str formatter
use core::fmt::{self, Write};

// A simple wrapper around a mutable slice to implement fmt::Write
pub struct ArrayWriter<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

impl<'a> ArrayWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        ArrayWriter { buffer, offset: 0 }
    }
    
    // Convert the buffer content (if valid UTF-8) to a string slice
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.buffer[..self.offset])
    }
}

// Implement the core trait required by the write! macro
impl<'a> Write for ArrayWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if self.offset + len > self.buffer.len() {
            return Err(fmt::Error); // Buffer overflow
        }

        // Copy bytes into the buffer
        self.buffer[self.offset..self.offset + len].copy_from_slice(bytes);
        self.offset += len;
        
        Ok(())
    }
}