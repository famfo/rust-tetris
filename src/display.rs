use crate::util::Color;
use std::cell::RefCell;
use std::io::Write;

const ESC: &str = "\x1b";

#[derive(Debug)]
struct Pixel {
    c: char,
    foreground_color: Color,
    background_color: Color,
}

pub struct Display {
    buffer: Vec<Vec<Pixel>>,
    writer: RefCell<Box<dyn Write>>,
}

impl Display {
    pub fn new(width: u32, height: u32, writer: RefCell<Box<dyn Write>>) -> Display {
        let mut rows = Vec::with_capacity(height as usize);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width as usize);
            for _ in 0..width {
                row.push(Pixel {
                    c: ' ',
                    foreground_color: Color::Black,
                    background_color: Color::Black,
                });
            }
            rows.push(row);
        }

        Display {
            buffer: rows,
            writer,
        }
    }

    pub fn render(&mut self) {
        self.clear_screen();

        let mut left = 0;

        if let Ok((w, _)) = termion::terminal_size() {
            //println!("{} {}", self.buffer.len(), self.buffer.len());
            left = ((0.5 * w as f32) as u32) - ((self.buffer.len() as f32 * 0.5) as u32);
        }

        self.set_cursor_pos(left, 0);

        let mut foreground_color = Color::Black;
        let mut background_color = Color::Black;

        let mut y = 0;

        for row in &self.buffer {
            for pixel in row {
                if pixel.foreground_color != foreground_color {
                    foreground_color = pixel.foreground_color;
                    self.set_foreground_color(pixel.foreground_color);
                }
                if pixel.background_color != background_color {
                    background_color = pixel.background_color;
                    self.set_background_color(pixel.background_color);
                }

                let bytes = [pixel.c as u8];
                assert!(self.writer.borrow_mut().write_all(&bytes).is_ok());
            }
            y += 1;
            self.set_cursor_pos(left, y);
        }

        assert!(self.writer.borrow_mut().flush().is_ok());
    }

    pub fn set_text<S: AsRef<str>>(
        &mut self,
        text: S,
        x: u32,
        y: u32,
        foreground_color: Color,
        background_color: Color,
    ) {
        let row = &mut self.buffer[y as usize];
        let mut i = 0;

        for c in text.as_ref().chars() {
            let cell = &mut row[(x + i) as usize];
            cell.c = c;
            cell.foreground_color = foreground_color;
            cell.background_color = background_color;
            i += 1;
        }
    }

    pub fn clear_screen(&self) {
        assert!(self
            .writer
            .borrow_mut()
            .write_all(self.esc("2J").as_bytes())
            .is_ok());
        assert!(self.writer.borrow_mut().flush().is_ok());
    }

    pub fn clear_buffer(&mut self) {
        for row in 0..self.buffer.len() {
            for col in 0..self.buffer[row].len() {
                self.buffer[row][col].c = ' ';
                self.buffer[row][col].foreground_color = Color::Black;
                self.buffer[row][col].background_color = Color::Black;
            }
        }
    }

    fn set_cursor_pos(&self, x: u32, y: u32) {
        // Console positions are 1-based
        self.print(&self.esc(&format!("{};{}H", y + 1, x + 1)));
    }

    fn esc(&self, text: &str) -> String {
        format!("{}[{}", ESC, text)
    }

    fn print(&self, text: &str) {
        assert!(self.writer.borrow_mut().write_all(text.as_bytes()).is_ok());
    }

    fn set_foreground_color(&self, color: Color) {
        self.print(&self.esc(&format!("38;5;{}m", color as i32)));
    }

    fn set_background_color(&self, color: Color) {
        self.print(&self.esc(&format!("48;5;{}m", color as i32)));
    }
}
