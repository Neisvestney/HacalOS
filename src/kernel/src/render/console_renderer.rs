use core::fmt;
use psf2::Font;
use crate::render::color::Color;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use crate::render::point::Point;

pub struct ConsoleRenderer<'a> {
    frame_buffer_renderer: FrameBufferRenderer,
    font: Font<&'a [u8]>,
    point: Point,
    foreground_color: Color,
    background_color: Color,
}

impl<'a> ConsoleRenderer<'a> {
    pub fn new(frame_buffer_renderer: FrameBufferRenderer, font: Font<&'a [u8]>, foreground_color: Color, background_color: Color) -> Self {
        ConsoleRenderer {
            frame_buffer_renderer,
            font,
            point: Point::new(0, 0),
            foreground_color,
            background_color,
        }
    }

    pub fn get_width(&self) -> usize {
        self.frame_buffer_renderer.horizontal_resolution / self.font.width() as usize
    }

    pub fn get_height(&self) -> usize {
        self.frame_buffer_renderer.vertical_resolution / self.font.height() as usize
    }

    pub fn set_foreground_color(&mut self, foreground_color: Color) {
        self.foreground_color = foreground_color;
    }

    pub fn set_background_color(&mut self, background_color: Color) {
        self.background_color = background_color;
    }

    pub fn clear(&mut self) {
        self.point = Point::new(0, 0);

        for y in 0..self.frame_buffer_renderer.vertical_resolution {
            for x in 0..self.frame_buffer_renderer.horizontal_resolution {
                unsafe {
                    self.frame_buffer_renderer.put_pixel_unchecked(x, y, self.background_color);
                }
            }
        }
    }

    pub fn next_line(&mut self) {
        self.point.y += 1;
        self.point.x = 0;

        if self.point.y >= self.get_height() {
            self.point.y -= 1;
            self.frame_buffer_renderer.vertical_shift(self.font.height() as usize, self.background_color);
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for char in string.chars() {
            match char {
                '\n' => self.next_line(),
                _ => {
                    unsafe {
                        self.frame_buffer_renderer.put_char_unchecked(char, self.point.x * self.font.width() as usize, self.point.y * self.font.height() as usize, self.foreground_color, &self.font);
                    }

                    self.point.x += 1;
                    if self.point.x >= self.get_width() {
                        self.next_line();
                    }
                }
            }
        }
    }
}

unsafe impl Send for ConsoleRenderer<'_> {}

impl fmt::Write for ConsoleRenderer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}