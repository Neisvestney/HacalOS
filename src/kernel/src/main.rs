#![feature(abi_x86_interrupt)]
#![feature(ptr_metadata)]
#![no_std]
#![no_main]

extern crate alloc;

use crate::acpi::acpi_handler::AcpiHandlerImpl;
use crate::acpi::{get_acpi_tables, get_apic_info};
use crate::frame_alloc::boolean_array_frame_allocator::BooleanArrayFrameAllocator;
use crate::gdt::init_gdt;
use crate::interrupts::idt::init_idt;
use crate::interrupts::ioapic::init_ioapic_interrupts;
use crate::interrupts::lapic::init_lapic;
use crate::interrupts::pic::disable_pics;
use crate::memory::heap::init_heap;
use crate::memory::paging::init_paging;
use crate::percpu::init::init_per_cpu;
use crate::render::color::Color;
use crate::render::console_renderer::ConsoleRenderer;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use crate::utils::logger::init_logger;
use ::acpi::platform::ProcessorState;
use ::acpi::{AcpiTables, InterruptModel};
use alloc::vec;
use bootloader_structs::BootInfo;
use core::arch::asm;
use core::panic::PanicInfo;
use log::{info, warn};
use psf2::Font;
use spin::{Mutex, Once};
use x86_64::VirtAddr;
use x86_64::instructions::hlt;
use x86_64::structures::paging::{OffsetPageTable, Translate};

mod acpi;
mod frame_alloc;
mod gdt;
mod interrupts;
mod memory;
mod percpu;
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
#[allow(improper_ctypes_definitions)]
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

    init_logger().expect("Failed to initialize logger");

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
    disable_pics();

    let acpi_tables = get_acpi_tables(&boot_info.rsbp_address);
    let apic_info = get_apic_info(&acpi_tables);
    info!("Apic info: {:?}", apic_info);
    let local_apic = init_lapic(&apic_info);
    let bsp_lapic_id = unsafe { local_apic.id() };
    assert!(
        bsp_lapic_id <= u8::MAX as u32,
        "BSP lapic id too large. This should not be possible"
    );
    init_per_cpu(local_apic);
    init_ioapic_interrupts(&apic_info, bsp_lapic_id as u8);

    x86_64::instructions::interrupts::enable();

    println!("HacalOS v0.0.2");

    let runtime_system_table = boot_info.runtime_system_table.unwrap();
    let runtime_services = unsafe { runtime_system_table.runtime_services() };

    info!("Time: {}", runtime_services.get_time().unwrap());

    FRAME_ALLOCATOR.get().unwrap().lock().print_stats();

    let platform_info = acpi_tables
        .platform_info()
        .expect("Failed to get platform info");

    let processor_info = platform_info.processor_info.unwrap();

    let bsp = processor_info.boot_processor;
    if bsp.state == ProcessorState::WaitingForSipi || bsp.is_ap == false {
        info!("BSP: APIC ID = {}", bsp.local_apic_id);
    }

    // AP (Application Processors) — остальные ядра
    for ap in processor_info.application_processors.iter() {
        info!("AP: APIC ID = {}, state = {:?}", ap.local_apic_id, ap.state);
    }

    match platform_info.interrupt_model {
        InterruptModel::Apic(apic) => {
            // Адрес Local APIC (общий для всех ядер)
            let lapic_addr = apic.local_apic_address;
            info!("Local APIC address: {:#x}", lapic_addr);

            // Перечисляем все IO APIC
            for io_apic in apic.io_apics.iter() {
                info!(
                    "IO APIC id={}, addr={:#x}, gsi_base={}",
                    io_apic.id, io_apic.address, io_apic.global_system_interrupt_base,
                );
            }

            // Interrupt Source Overrides (например, IRQ0 → GSI2 для таймера)
            for iso in apic.interrupt_source_overrides.iter() {
                warn!(
                    "ISO: irq={} → gsi={}",
                    iso.isa_source, iso.global_system_interrupt
                );
            }
        }
        _ => panic!("Non-APIC interrupt model"),
    }

    // let mut a = vec![1, 2, 3];
    // a.push(5);
    // println!("{:?} {:#?}", a, a.as_ptr());
    //
    // println!("Hello before interrupt");
    // x86_64::instructions::interrupts::int3();
    // unsafe {
    //     asm!("int $0x80", options(nomem, nostack));
    // }
    // println!("Hello after interrupt");

    loop {
        hlt();
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    CONSOLE
        .get()
        .unwrap()
        .lock()
        .set_foreground_color(Color::ERROR);
    println!("{}", _info);

    loop {}
}
