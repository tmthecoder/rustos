use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // Get a reference to the serial port by an unsave access to the UART I/O port
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        // Initialize
        serial_port.init();
        // Return as mutex
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
// Print to the serial console (host os console)
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

// Actual macro to print to the serial console with formatted strings
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

// Macro to print with a newline appended to the serial console
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}