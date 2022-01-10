#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

mod vga_buffer;

// Hello world byt to bytes
static HELLO: &[u8] = b"Hello World!";

// Entry function as the linker looks for '_start()' by default
#[no_mangle] // Don't mangle this as it's the entrypoint
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();
    vga_buffer::WRITER.lock().write_str("\n New Line Hello").unwrap();
    loop{}
}

// Called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}