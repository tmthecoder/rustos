#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle] // Don't mangle this as it's the entrypoint
pub extern "C" fn _start() -> ! {
    // Entry function as the linker looks for '_start()' by default
    loop{}
}

// Called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}