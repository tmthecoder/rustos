#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"] // Rename the generated test 'main' function

use core::panic::PanicInfo;
use rustos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main(); // No need to be in test mode as this will never run outside of tests

    loop{}
}

// Handler for when panic is called
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustos::test_panic_handler(info)
}

// Test printing after boot to ensure the VGA console works
#[test_case]
fn test_println() {
    println!("test_println output");
}