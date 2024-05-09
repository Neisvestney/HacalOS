#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod render;
mod interrupts;
mod gdt;
mod serial;
mod print;

use core::arch::asm;
use core::panic::PanicInfo;
use bootloader_structs::BootInfo;
use psf2::Font;
use x86_64::instructions::hlt;
use crate::render::color::Color;
use crate::render::console_renderer::ConsoleRenderer;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use spin::{Mutex, Once};
use core::fmt::Write;
use crate::gdt::init_gdt;
use crate::interrupts::{init_idt, init_pics};

static BOOT_INFO: Once<BootInfo> = Once::new();
static CONSOLE: Once<Mutex<ConsoleRenderer>> = Once::new();

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: &'static BootInfo) -> usize {
    x86_64::instructions::interrupts::disable();
    BOOT_INFO.call_once(|| boot_info.clone());
    let boot_info = BOOT_INFO.get().unwrap();

    let renderer = FrameBufferRenderer::new(&boot_info.gop);
    let font_data = boot_info.font;
    let font = Font::new(font_data).unwrap();
    let console = ConsoleRenderer::new(renderer, font, Color::from_rgb(255, 255, 255), Color::from_rgb(30, 30, 30));
    CONSOLE.call_once(|| Mutex::new(console));

    let (width, height) = {
        let mut console = CONSOLE.get().unwrap().lock();
        console.clear();
        let width = console.get_width();
        let height = console.get_height();

        (width, height)
    };

    init_gdt();
    init_idt();
    init_pics();

    x86_64::instructions::interrupts::enable();

    println!("Hello before interrupt");
    x86_64::instructions::interrupts::int3();
    unsafe {
        asm!("int $0x80", options(nomem, nostack));
    }
    println!("Hello after interrupt");

    loop { hlt(); }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    {
        let mut console = CONSOLE.get().unwrap().lock();
        console.set_foreground_color(Color::from_rgb(200, 100, 100));
    }
    println!("{}", _info);

    loop {}
}