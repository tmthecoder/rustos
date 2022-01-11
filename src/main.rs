#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // Rename the generated test 'main' function

use core::panic::PanicInfo;

mod vga_buffer;
mod serial;

// Entry function as the linker looks for '_start()' by default
#[no_mangle] // Don't mangle this as it's the entrypoint
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main(); // Call that renamed function on testing configs

    loop{}
}

// Called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// A panic handler called solely when testing (exits and prints to serial)
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// Main test runner (needed for manual test config)
// '&[&dyn Fn()] - Slice of items that implement the 'Fn()' trait (basically a list of references to functions)
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    // Run each test
    for test in tests {
        test.run();
    }
    // Exit with a successful code as all tests passed
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
// A trivial test to check passing
fn trivial_assertion() {
    assert_eq!(1, 1);
}

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
        serial_println!("[ok");
    }
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