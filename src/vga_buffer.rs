#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // enum's underlying representation will be of type 'u8'
// Enum to abstract colors allowed by the VGA buffer
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Ensures ColorCode has the same layout as a u8
struct ColorCode(u8); // Struct to represent a color code

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // Color code contains the full color byte (including foreground and background)
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Guarantees that struct is laid out like a C struct
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode
}

// The writable screen height (rows in the 2D array)
const BUFFER_HEIGHT: usize = 25;
// The writable screen width (columns in the 2D array)
const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;
#[repr(transparent)] // Ensures the Buffer has the same layout as its single field
// The abstracted representation of the VGA Buffer
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize, // Current position in the last row
    color_code: ColorCode, // Color code for current color
    buffer: &'static mut Buffer, // Reference to the VGA buffer
}

impl Writer {
    // The method that writes a single byte to the VGA buffer, with the given color code
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // If the byte is a newline byte, just call the newline method
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    // Go to the next line if the current one is full
                    self.new_line();
                }

                // Get the row and column of the desired write
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                // Get the color code
                let color_code = self.color_code;
                // Write the new 'ScreenChar' to the buffer at the current position
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code
                });
                self.column_position += 1;
            }
        }
    }

    // Moves each character one line up (deleting the top if applicable) and starts at tge beginning of the last line again
    fn new_line(&mut self) {
        // Loop through each row (except the top) and column, moving all existing characters one row up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character)
            }
        }
        // Clear the bottom row
        self.clear_row(BUFFER_HEIGHT - 1);
        // Reset column position
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        // The blank character (just a space)
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        // Set each item in the buffer to the blank character
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank)
        }
    }
}

impl Writer {
    // A convenience method to write an entire string to the VGA buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Check if byte is in printable ASCII range (or newline)
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Byte isn't in printable ASCII range
                _ => self.write_byte(0xfe)
            }
        }
    }
}

use core::fmt;
use core::fmt::Write;

// Allows access to the fmt::Write trait
// Implement write! format macros for the Writer struct
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use lazy_static::lazy_static;
use spin::Mutex;
// The global interface to use as a writer from external code
// Needs 'lazy_static' as you can't convert raw pointers to references at compile time
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightBlue, Color::Black),
        // The location of the vga buffer: 0xb8000
        // VGA Buffer article: https://os.phil-opp.com/vga-text-mode/
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
// The method to actually send the formatted string to the VGA buffer from 'print!'
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// Add support for the 'print!' macro
// Writes the sent formatted string to the VGA buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

// Add support for the 'println!' macro
// Writes the sent formatted string to the VGA buffer appended by a newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Test that a simple one-line print works
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output")
}

// Test that a multi-line print works (lots of lines)
#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

// Test that a test string actually matches it's supposed value in the VGA buffer
#[test_case]
fn test_println_output() {
    use x86_64::instructions::interrupts;
    let s = "Some test string";
    // Run without interrupts so nothing else is printed
    interrupts::without_interrupts(|| {
        // Lock the writer and get mutable access
       let mut writer = WRITER.lock();
        // Write to the writer
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            // Verify each character
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT-2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}