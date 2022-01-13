#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)] // Use the shared library's method as the test runner
#![reexport_test_harness_main = "test_main"] // Rename the generated test 'main' function

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Translate;
use rustos::{memory, println};
use rustos::memory::BootInfoFrameAllocator;

// The bootloader package's provided macro to set the entry point of the OS
entry_point!(kernel_main);

// Rust type-checked entry function with the 'boot_info' parameter
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello World{}", "!");
    rustos::init();

    #[cfg(test)]
    test_main(); // Call that renamed function on testing configs

    println!("No Crashes!");

    rustos::hlt_loop();
}

// Called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rustos::hlt_loop();
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