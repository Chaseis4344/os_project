use lazy_static::lazy_static;
use volatile::Volatile;
use core::fmt;
use spin::Mutex;


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]

//Give us strict numbers to use, but make it english
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
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]

struct ColorCode(u8);
//The color code is 8 bits, bbbbffff
//So we start 0000bbbb -> bbbb0000 -> bbbbffff
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

//These traits are required for printing
//We also need the struct to be laid out the same way it is in C
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
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
                b'\n' => self.new_line(),
                byte =>{
                    if self.column_position >= BUFFER_WIDTH {
                        self.new_line();
                    }
                

                let row = BUFFER_HEIGHT -1; 
                let col = self.column_position;
                
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes(){
            match byte {
                //Printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
               //Out of Range Error
                _ => self.write_byte(0xfe),
            }
        }
    }

   

    fn new_line(&mut self) {
        //Move everything dsplayed up 1 line
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        //Clear the next row out
        //Line Feed
        self.clear_row(BUFFER_HEIGHT -1);
        //Carriage Return
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize){
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        //Write a space from left to right on current row
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}



impl core::fmt::Write for Writer {

    fn write_str(&mut self, s: &str)  -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Green, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
});
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
