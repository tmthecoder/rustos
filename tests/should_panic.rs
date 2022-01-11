#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rustos::{QemuExitCode, exit_qemu, serial_println, serial_print};

// Exiting panic handler for the panic test
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// A test that deliberately causes a panic
fn should_fail() {
    serial_print!("should panic::should_fail...\t");
    assert_eq!(0, 1);
}