use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::gdt;

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
        idt
    };
}

// A method to load the IDT
pub fn init_idt() {
    IDT.load()
}

// A function to handle breakpoint exceptions, just prints the exception currently
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}