#![no_std]
#![no_main]

mod render;

use core::panic::PanicInfo;
use bootloader_structs::BootInfo;
use psf2::Font;
use x86_64::instructions::hlt;
use crate::render::color::Color;
use crate::render::console_renderer::ConsoleRenderer;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use core::fmt::Write;

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: &BootInfo) -> usize {
    let mut renderer = FrameBufferRenderer::new(&boot_info.gop);
    renderer.put_pixel(1,1, Color::from_rgb(255, 0, 255));
    let font = Font::new(boot_info.font).unwrap();
    let mut console = ConsoleRenderer::new(renderer, font, Color::from_rgb(255, 255, 255), Color::from_rgb(30, 30, 30));

    console.clear();
    for i in 0..101 {
        writeln!(console, "Does this works? {i} {}x{}", console.get_width(), console.get_height()).unwrap();
    }

    loop { hlt(); }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}