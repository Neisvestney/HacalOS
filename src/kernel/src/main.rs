#![feature(abi_x86_interrupt)]
#![feature(ptr_metadata)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use core::arch::asm;
use core::panic::PanicInfo;

use bootloader_structs::BootInfo;
use psf2::Font;
use spin::{Mutex, Once};
use x86_64::VirtAddr;
use x86_64::instructions::hlt;
use x86_64::structures::paging::OffsetPageTable;

use crate::frame_alloc::boolean_array_frame_allocator::BooleanArrayFrameAllocator;
use crate::gdt::init_gdt;
use crate::interrupts::{init_idt, init_pics};
use crate::memory::heap::init_heap;
use crate::memory::paging::init_paging;
use crate::render::color::Color;
use crate::render::console_renderer::ConsoleRenderer;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;

mod frame_alloc;
mod gdt;
mod interrupts;
mod memory;
mod print;
mod render;
mod serial;
mod utils;

// static BOOT_INFO: Once<BootInfo> = Once::new();
static CONSOLE: Once<Mutex<ConsoleRenderer>> = Once::new();
static PAGE_TABLE_MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();
static FRAME_ALLOCATOR: Once<Mutex<BooleanArrayFrameAllocator>> = Once::new();

const VIRTUAL_TO_PHYSICAL_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFF900000000000) };
const HEAP_START: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFF820000000000) };
const HEAD_SIZE: usize = 100 * 1024;

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(boot_info: BootInfo) -> usize {
    x86_64::instructions::interrupts::disable();

    // Console
    let renderer = FrameBufferRenderer::new(&boot_info.gop);
    let font_data = boot_info.font;
    let font = Font::new(font_data).unwrap();
    let console = ConsoleRenderer::new(
        renderer,
        font,
        Color::from_rgb(255, 255, 255),
        Color::from_rgb(30, 30, 30),
    );
    CONSOLE.call_once(|| Mutex::new(console));

    // Frame allocator
    let memory_map = boot_info.memory_map.unwrap();
    let mut frame_allocator = BooleanArrayFrameAllocator::new(boot_info.frame_allocator_buffer);
    frame_allocator.read_from_memory_map(&memory_map);
    FRAME_ALLOCATOR.call_once(|| Mutex::new(frame_allocator));

    let (_width, _height) = {
        let mut console = CONSOLE.get().unwrap().lock();
        console.clear();
        let width = console.get_width();
        let height = console.get_height();

        (width, height)
    };

    init_paging(&memory_map, boot_info.kernel_memory_map, &boot_info.gop);
    init_heap().unwrap();
    init_gdt();
    init_idt();
    init_pics();

    x86_64::instructions::interrupts::enable();

    println!("HacalOS v0.0.2");

    let runtime_system_table = boot_info.runtime_system_table.unwrap();
    let runtime_services = unsafe { runtime_system_table.runtime_services() };

    println!("Time: {}", runtime_services.get_time().unwrap());
    {
        FRAME_ALLOCATOR.get().unwrap().lock().print_stats();
    }

    let mut a = vec![1, 2, 3];
    a.push(5);
    println!("{:?} {:#?}", a, a.as_ptr());

    println!("Hello before interrupt");
    x86_64::instructions::interrupts::int3();
    unsafe {
        asm!("int $0x80", options(nomem, nostack));
    }
    println!("Hello after interrupt");

    loop {
        hlt();
    }
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
