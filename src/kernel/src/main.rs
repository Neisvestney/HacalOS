#![feature(abi_x86_interrupt)]
#![feature(ptr_metadata)]
#![feature(never_type)]
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
use crate::memory::heap::init_kernel_heap;
use crate::memory::paging::{init_paging, unmap_lower_half};
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
use core::ops::Deref;
use core::panic::PanicInfo;
use log::{info, warn};
use psf2::Font;
use spin::{Mutex, Once};
use uefi::table::{Runtime, SystemTable};
use uefi::table::boot::MemoryMap;
use x86_64::VirtAddr;
use x86_64::instructions::hlt;
use x86_64::structures::paging::{OffsetPageTable, Page, Size4KiB, Translate};
use inithfs::InitHFsRoot;
use crate::hpet::{init_hpet, HPET};
use crate::memory::stack::{allocate_stack, switch_stack_and_jump};
use crate::memory::virtual_memory_allocator::init_kernel_virtual_memory_allocator;
use crate::uefi_runtime_services::relocate_uefi_runtime_services;
use crate::utils::relocate::relocate_raw_pointer;

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
mod hpet;
mod uefi_runtime_services;

// static BOOT_INFO: Once<BootInfo> = Once::new();
static CONSOLE: Once<Mutex<ConsoleRenderer>> = Once::new();
static PAGE_TABLE_MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();
static FRAME_ALLOCATOR: Once<Mutex<BooleanArrayFrameAllocator>> = Once::new();

const VIRTUAL_TO_PHYSICAL_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFF900000000000) };

const VIRTUAL_MEMORY_REGION_START: Page<Size4KiB> = unsafe { Page::from_start_address_unchecked(VirtAddr::new_unsafe(0xFFFF820000000000)) };
const VIRTUAL_MEMORY_REGION_PAGES_COUNT: u64 = 1024;

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "sysv64" fn _start(boot_info: BootInfo) -> usize {
    x86_64::instructions::interrupts::disable();

    init(boot_info);

    0
}

fn init(boot_info: BootInfo) {
    // Console
    let renderer = FrameBufferRenderer::new(&boot_info.gop);
    let font_data = boot_info.font;
    let font = Font::new(font_data).unwrap();
    let mut console = ConsoleRenderer::new(
        renderer,
        font,
        Color::from_rgb(255, 255, 255),
        Color::from_rgb(30, 30, 30),
    );
    console.clear();
    CONSOLE.call_once(|| Mutex::new(console));
    init_logger().expect("Failed to initialize logger");

    info!("HacalOS initializing...");

    // Frame allocator
    let memory_map = boot_info.memory_map.unwrap();
    let mut frame_allocator = BooleanArrayFrameAllocator::new(boot_info.frame_allocator_buffer);
    frame_allocator.read_from_memory_map(&memory_map);
    FRAME_ALLOCATOR.call_once(|| Mutex::new(frame_allocator));

    init_paging(&memory_map, boot_info.kernel_memory_map, &boot_info.gop, &boot_info.font);
    init_kernel_virtual_memory_allocator();
    init_kernel_heap().unwrap();
    let (bsp_kernel_stack_top, _bsp_kernel_stack_protection_page) = allocate_stack().expect("Cannot allocate memory for stack");
    let (gdt, tss) = init_gdt();
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
    init_per_cpu(local_apic, gdt, tss);
    init_ioapic_interrupts(&apic_info, bsp_lapic_id as u8);
    init_hpet(&acpi_tables);

    let runtime_system_table = boot_info.runtime_system_table.unwrap();
    info!("Jumping to new stack");
    switch_stack_and_jump(bsp_kernel_stack_top, move || {main(runtime_system_table, acpi_tables, memory_map, boot_info.inithfs_bytes)});
}

fn main(runtime_system_table: SystemTable<Runtime>, acpi_tables: AcpiTables<AcpiHandlerImpl>, memory_map: MemoryMap, inithfs_bytes: &'static [u8]) -> ! {
    let runtime_system_table = relocate_uefi_runtime_services(runtime_system_table, &memory_map);
    let runtime_services = unsafe { runtime_system_table.runtime_services() };
    info!("Unmapping lower half");
    unmap_lower_half(&memory_map);

    x86_64::instructions::interrupts::enable();

    println!("HacalOS v0.0.3");

    // unsafe {
    //     info!("GDT {:#?}", percpu::current().gdt);
    //     info!("TSS {:#?}", percpu::current().tss);
    // }

    // fn test() {
    //     test();
    // }
    // test();

    // unsafe {
    //     (0xFFFFAA0000000000 as *mut u8).write_volatile(1);
    // }

    info!("Time: {}", runtime_services.get_time().unwrap());

    let inithfs_bytes = unsafe { &*relocate_raw_pointer(inithfs_bytes) };
    let inithfs = InitHFsRoot::from_bytes(inithfs_bytes).expect("Failed to parse InitHFs");
    info!("inithfs {:?}", inithfs);
    let test_file = inithfs.get_file("test.txt").unwrap();
    info!("file test.txt = {} {:p}", str::from_utf8(test_file).unwrap(), test_file);

    unsafe {
        asm!("int $0x80", options(nomem, nostack));
    }

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
                    "ISO: irq={} to gsi={}",
                    iso.isa_source, iso.global_system_interrupt
                );
            }
        }
        _ => panic!("Non-APIC interrupt model"),
    }

    // fn test() {
    //     test()
    // }
    // test();

    {
        let hpet = HPET.get().unwrap().read();
        for a in 0..3 {
            println!("{}", a);
            hpet.wait_ms(1000);
        }
    }

    let mut a = vec![1, 2, 3];
    a.push(5);
    info!("{:?} {:#?}", a, a.as_ptr());

    // info!("Hello before interrupt");
    // x86_64::instructions::interrupts::int3();
    // unsafe {
    //     asm!("int $0x80", options(nomem, nostack));
    // }
    // info!("Hello after interrupt");

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
