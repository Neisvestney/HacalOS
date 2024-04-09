#![no_std]
#![no_main]

mod render;

use core::panic::PanicInfo;
use bootloader_structs::BootInfo;
use psf2::Font;
use x86_64::instructions::hlt;
use crate::render::color::Color;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: &BootInfo) -> usize {
    let font = Font::new(boot_info.font).unwrap();
    let mut renderer = FrameBufferRenderer::new(&boot_info.gop, font);

    for x in 0..boot_info.gop.horizontal_resolution {
        for y in 0..boot_info.gop.vertical_resolution {
            renderer.put_pixel(x, y, Color::from_rgb(10, 10, 10))
        }
    }

    renderer.put_string("Goodbye, cruel world...", 20, 20, Color::from_rgb(255, 255, 255));

    loop { hlt(); }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}