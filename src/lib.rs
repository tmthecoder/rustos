#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // Rename the generated test 'main' function

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;

#[cfg(test)]
entry_point!(test_kernel_main);

// Create a trait for all test functions
pub trait Testable {
    fn run(&self) -> ();
}

// Make all existing Functions conform to the Testable traut
impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        // Print the function name to the console
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        // Test passed at this point, so print 'ok'
        serial_println!("[ok]");
    }
}

// Main test runner (needed for manual test config)
// '&[&dyn Fn()] - Slice of items that implement the 'Fn()' trait (basically a list of references to functions)
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    // Run each test
    for test in tests {
        test.run();
    }
    // Exit with a successful code as all tests passed
    exit_qemu(QemuExitCode::Success);
}

// A panic handler called solely when testing (exits and prints to serial)
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // Initialize PICs (unsafe as it can cause undefined behaviour when PIC is misconfigured)
    x86_64::instructions::interrupts::enable(); // Enable interrupts
}

// A loop that sends CPU halt instructions when not needed
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // Represented as a u32
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// A method to exit qemu, uses the x86_64 crate to write an exit code to the 0xf4 port and shutdown the emulator
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32)
    }
}