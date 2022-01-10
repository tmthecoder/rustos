#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

// Entry function as the linker looks for '_start()' by default
#[no_mangle] // Don't mangle this as it's the entrypoint
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    panic!("some panic message");
    loop{}
}

// Called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}