#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Hello world byt to bytes
static HELLO: &[u8] = b"Hello World!";

// Entry function as the linker looks for '_start()' by default
#[no_mangle] // Don't mangle this as it's the entrypoint
pub extern "C" fn _start() -> ! {
    // The location of the vga buffer
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            // Write the string byte
            *vga_buffer.offset(i as isize * 2) = byte;
            // Write the color byte (0xb is light cyan)
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb
        }
    }

    loop{}
}

// Called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}