use bootloader_structs::GopInfo;
use psf2::Font;
use crate::render::color::Color;

pub struct FrameBufferRenderer<'a> {
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
    pub font: Font<&'a [u8]>,
}

impl<'a> FrameBufferRenderer<'a> {
    pub fn new(gop_info: &GopInfo, font: Font<&'a [u8]>) -> Self {
        FrameBufferRenderer {
            frame_buffer: gop_info.frame_buffer,
            frame_buffer_size: gop_info.frame_buffer_size,
            horizontal_resolution: gop_info.horizontal_resolution,
            vertical_resolution: gop_info.vertical_resolution,
            font,
        }
    }
    pub unsafe fn put_pixel_unchecked(&mut self, x: usize, y: usize, color: Color) {
        self.frame_buffer.add(x as usize * 4 + (y as usize * 4 * self.horizontal_resolution)).cast::<u32>().write_volatile(color.as_u32());
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x > self.horizontal_resolution {
            return;
        }
        if y > self.vertical_resolution {
            return;
        }
        unsafe {
            self.put_pixel_unchecked(x, y, color);
        }
    }

    pub fn put_char(&mut self, char: char, x: usize, y: usize, color: Color) {
        let glyph = self.font.get_ascii(char as u8).unwrap_or(self.font.get_ascii('?' as u8).unwrap());

        for (yo, row) in glyph.enumerate() {
            for (xo, flag) in row.enumerate() {
                if flag {
                    unsafe {
                        self.frame_buffer.add(x + xo * 4 + (y + yo * 4 * self.horizontal_resolution)).cast::<u32>().write_volatile(color.as_u32());
                    }
                }
            }
        }
    }

    pub fn put_string(&mut self, string: &str, x: usize, y: usize, color: Color) {
        let char_width = self.font.width();
        for (i, char) in string.chars().enumerate() {
            self.put_char(char, x + (i * char_width as usize * 4), y, color);
        }
    }
}