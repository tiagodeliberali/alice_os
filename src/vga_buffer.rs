use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[cfg(test)]
use crate::{serial_print, serial_println};

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Cyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGrey = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.column_position = 0;
        self.clear_line(BUFFER_HEIGHT - 1)
    }

    #[cfg(test)]
    fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_line(row);
        }
        self.column_position = 0;
    }

    fn clear_line(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_print_simple_line() {
    prepare_test("test println... ");
    println!("simple line print...");
    serial_println!("[ok]");
}

#[test_case]
fn test_print_many_line() {
    prepare_test("test println many times... ");
    for _ in 0..200 {
        println!("single line print...");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    prepare_test("test println output... ");
    
    let s = "long line but single one";
    println!("{}", s);

    for (i, c) in s.bytes().enumerate() {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char_screen.ascii_character, c);
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_print_output() {
    prepare_test("test print output... ");
    
    let s = "long line but single one";
    print!("{}", s);

    for (i, c) in s.bytes().enumerate() {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][i].read();
        assert_eq!(char_screen.ascii_character, c);
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_print_invalid_char_output() {
    prepare_test("test print invalid char output... ");
    
    let s = "áçãó";
    print!("{}", s);

    for col in 0..8 {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][col].read();
        assert_eq!(char_screen.ascii_character, 0xfe);
    }

    for col in 8..BUFFER_WIDTH {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][col].read();
        assert_eq!(char::from(char_screen.ascii_character), char::from(' '));
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_print_with_line_break_output() {
    prepare_test("test print with line break output... ");
    WRITER.lock().clear_screen();
    
    let s1 = "first line";
    let s2 = "second line";
    print!("{}\n{}", s1, s2);

    for (i, c) in s1.bytes().enumerate() {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(char_screen.ascii_character), char::from(c), "failed on position {}", i);
    }

    for (i, c) in s2.bytes().enumerate() {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][i].read();
        assert_eq!(char::from(char_screen.ascii_character), char::from(c), "failed on position {}", i);
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_print_break_long_line_output() {
    prepare_test("test print break long line output... ");
    WRITER.lock().clear_screen();

    let s = "a really really really really really really really really really really really really really really long line";
    print!("{}", s);

    for (i, c) in s[..80].bytes().enumerate() {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(char_screen.ascii_character), char::from(c), "failed on position {}", i);
    }

    for (i, c) in s[80..].bytes().enumerate() {
        let char_screen = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][i].read();
        assert_eq!(char::from(char_screen.ascii_character), char::from(c), "failed on position {}", i);
    }

    serial_println!("[ok]");
}

#[cfg(test)]
#[allow(dead_code)]
fn print_screen_serial() {
    for row in 0..BUFFER_HEIGHT {
        for col in 0..BUFFER_WIDTH {
            let value = WRITER.lock().buffer.chars[row][col].read();
            serial_print!("{}", char::from(value.ascii_character));
        }
        serial_println!();
    }
}

#[cfg(test)]
fn prepare_test(name: &str) {
    WRITER.lock().clear_screen();
    serial_print!("{}", name);
}