#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)] // Use the shared library's method as the test runner
#![reexport_test_harness_main = "test_main"] // Rename the generated test 'main' function

use core::panic::PanicInfo;
use rustos::println;

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

// Panic called when testing
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustos::test_panic_handler(info);
}

// A trivial test to check passing
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}