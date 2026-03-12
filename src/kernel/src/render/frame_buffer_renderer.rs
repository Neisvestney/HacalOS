use crate::render::color::Color;
use bootloader_structs::GopInfo;
use psf2::Font;

#[allow(unused)]
pub struct FrameBufferRenderer {
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}

impl FrameBufferRenderer {
    pub fn new(gop_info: &GopInfo) -> Self {
        FrameBufferRenderer {
            frame_buffer: gop_info.frame_buffer,
            frame_buffer_size: gop_info.frame_buffer_size,
            horizontal_resolution: gop_info.horizontal_resolution,
            vertical_resolution: gop_info.vertical_resolution,
        }
    }
    pub unsafe fn put_pixel_unchecked(&mut self, x: usize, y: usize, color: Color) {
        unsafe {
            self.frame_buffer
                .add(x * 4 + (y * 4 * self.horizontal_resolution))
                .cast::<u32>()
                .write_volatile(color.as_u32());
        }
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

    pub fn put_char(
        &mut self,
        char: char,
        x: usize,
        y: usize,
        foreground_color: Color,
        background_color: Color,
        font: &Font<&[u8]>,
    ) {
        let glyph = font
            .get_ascii(char as u8)
            .unwrap_or(font.get_ascii(0x0).unwrap());

        for (yo, row) in glyph.enumerate() {
            for (xo, flag) in row.enumerate() {
                self.put_pixel(
                    x + xo,
                    y + yo,
                    if flag {
                        foreground_color
                    } else {
                        background_color
                    },
                );
            }
        }
    }

    pub fn vertical_shift(&mut self, rows: usize, fill_color: Color) {
        let buffer_slice = unsafe {
            core::slice::from_raw_parts_mut(
                self.frame_buffer.cast::<u32>(),
                self.vertical_resolution * self.horizontal_resolution,
            )
        };

        let shift = rows * self.horizontal_resolution;

        for i in 0..buffer_slice.len() - shift {
            buffer_slice[i] = buffer_slice[i + shift];
        }

        for i in buffer_slice.len() - shift..buffer_slice.len() {
            buffer_slice[i] = fill_color.as_u32();
        }
    }

    // pub fn put_string(&mut self, string: &str, x: usize, y: usize, foreground_color: Color, background_color: Color, font: &Font<&[u8]>) {
    //     let char_width = font.width();
    //     for (i, char) in string.chars().enumerate() {
    //         self.put_char(char, x + (i * char_width as usize * 4), y, foreground_color, background_color, font);
    //     }
    // }
}

unsafe impl Send for FrameBufferRenderer {}
