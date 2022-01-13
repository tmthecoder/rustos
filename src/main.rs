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

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // Write the string 'New!' to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // cirtual address mapped to physical address 0
    //     boot_info.physical_memory_offset
    // ];
    //
    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

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