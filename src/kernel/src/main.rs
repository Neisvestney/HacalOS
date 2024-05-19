#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod render;
mod interrupts;
mod gdt;
mod serial;
mod print;
mod utils;
mod alloc;
mod memory;

use core::arch::asm;
use core::ops::{Deref, DerefMut};
use core::panic::PanicInfo;
use bootloader_structs::{BootInfo, GopInfo};
use psf2::Font;
use x86_64::instructions::hlt;
use crate::render::color::Color;
use crate::render::console_renderer::ConsoleRenderer;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use spin::{Mutex, Once};
use uefi::table::boot::MemoryType;
use x86_64::structures::paging::{Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::{Cr3, Cr3Flags};
use crate::alloc::boolean_array_frame_allocator::BooleanArrayFrameAllocator;
use crate::gdt::init_gdt;
use crate::interrupts::{init_idt, init_pics};
use crate::memory::paging::init_paging;
use crate::utils::relocate::{relocate_frame, relocate_raw_pointer_mut};

// static BOOT_INFO: Once<BootInfo> = Once::new();
static CONSOLE: Once<Mutex<ConsoleRenderer>> = Once::new();
static PAGE_TABLE_MANAGER: Once<Mutex<OffsetPageTable>> = Once::new();
static FRAME_ALLOCATOR: Once<Mutex<BooleanArrayFrameAllocator>> = Once::new();

const VIRTUAL_TO_PHYSICAL_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFF900000000000) };

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: BootInfo) -> usize {
    x86_64::instructions::interrupts::disable();

    // Console
    let renderer = FrameBufferRenderer::new(&boot_info.gop);
    let font_data = boot_info.font;
    let font = Font::new(font_data).unwrap();
    let console = ConsoleRenderer::new(renderer, font, Color::from_rgb(255, 255, 255), Color::from_rgb(30, 30, 30));
    CONSOLE.call_once(|| Mutex::new(console));

    // Frame allocator
    let memory_map = boot_info.memory_map.unwrap();
    let mut frame_allocator = BooleanArrayFrameAllocator::new(boot_info.frame_allocator_buffer);
    frame_allocator.read_from_memory_map(&memory_map);
    FRAME_ALLOCATOR.call_once(|| Mutex::new(frame_allocator));

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
    init_paging(&memory_map, &boot_info.kernel_memory_map, &boot_info.gop);

    x86_64::instructions::interrupts::enable();

    println!("HacalOS v0.0.2");

    let runtime_system_table = boot_info.runtime_system_table.unwrap();
    let runtime_services = unsafe {runtime_system_table.runtime_services()};

    println!("Time: {}", runtime_services.get_time().unwrap());

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