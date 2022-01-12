use lazy_static::lazy_static;
use pc_keyboard::ScancodeSet1;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use spin;
use pic8259::ChainedPics;
use crate::{print, println};
use crate::gdt;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// TODO Try to use raw handlers w this post: https://os.phil-opp.com/edition-1/extra/naked-exceptions/
lazy_static! {
    // Create a static reference to the InterruptDescriptorTable that lives the duration of the program
   static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // set the breakpoint handler to the function we made
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // set the double fault handler to the function we made
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // Set the timer handler (From the PIC)
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        // Set the keyboard handler (From the PIC)
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// An enum to map each interrupt index to readable values
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // Defaults to Timer + 1
}

// Functions for easy numeric access to each interrupt index
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// A method to load the IDT
pub fn init_idt() {
    IDT.load()
}

// A function to handle breakpoint exceptions, just prints the exception currently
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// A function to handle double fault exceptions, panics with exception stackframe currently
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// A function to handle timer interrupts, prints a '.' as of now
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    // Notify the PIC that we're finished processing the interrupt
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use spin::Mutex;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }

    // Number mappings on the keyboard (commended in favor of the 'pc-keyboard' crate)
    // let key = match scancode {
    //     0x02 => Some('1'),
    //     0x03 => Some('2'),
    //     0x04 => Some('3'),
    //     0x05 => Some('4'),
    //     0x06 => Some('5'),
    //     0x07 => Some('6'),
    //     0x08 => Some('7'),
    //     0x09 => Some('8'),
    //     0x0a => Some('9'),
    //     0x0b => Some('0'),
    //     _ => None,
    // };
    // if let Some(key) = key {
    //     print!("{}", key);
    // }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}