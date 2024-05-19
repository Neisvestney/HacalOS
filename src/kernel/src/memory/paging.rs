use core::ops::DerefMut;
use bootloader_structs::{BootInfo, GopInfo, KernelMapEntry};
use spin::Mutex;
use uefi::table::boot::{MemoryMap, MemoryType};
use x86_64::structures::paging::{Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::{Cr3, Cr3Flags};
use crate::{CONSOLE, FRAME_ALLOCATOR, PAGE_TABLE_MANAGER, println, VIRTUAL_TO_PHYSICAL_OFFSET};
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use crate::utils::relocate::{relocate_frame, relocate_raw_pointer_mut};

pub fn init_paging(memory_map: &MemoryMap, kernel_memory_map: &'static [KernelMapEntry], gop: &GopInfo) {
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();
    frame_allocator.print_stats();

    let page_table_addr = frame_allocator.request_page().unwrap();
    let page_table = page_table_addr.start_address().as_u64() as *mut PageTable;
    let mut page_table_manager = unsafe {OffsetPageTable::new(&mut *page_table, VirtAddr::new(0))};

    for entry in memory_map.entries().filter(|e| e.ty == MemoryType::MMIO || e.ty == MemoryType::MMIO_PORT_SPACE) {
        println!("{:#?}", entry);
    }

    for memory_map_entry in memory_map.entries() {
        for i in 0..memory_map_entry.page_count {
            let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(memory_map_entry.phys_start)).unwrap() + i;
            let virtual_frame = Page::from_start_address(VirtAddr::new(memory_map_entry.phys_start)).unwrap() + i;
            let virtual_frame_higher_half = Page::from_start_address(VirtAddr::new(memory_map_entry.phys_start + VIRTUAL_TO_PHYSICAL_OFFSET.as_u64())).unwrap() + i;

            unsafe {
                page_table_manager
                    .map_to(virtual_frame, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator.deref_mut())
                    .unwrap()
                    .flush();

                page_table_manager
                    .map_to(virtual_frame_higher_half, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator.deref_mut())
                    .unwrap()
                    .flush();
            }
        }
    }
    for memory_map_entry in kernel_memory_map {
        for i in 0..memory_map_entry.page_count {
            unsafe {
                page_table_manager
                    .map_to(memory_map_entry.page + i, memory_map_entry.frame + i, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator.deref_mut())
                    .unwrap()
                    .flush();
            }
        }
    }

    // Relocate framebuffer
    for i in 0..(gop.frame_buffer_size + 4095) / 4096 {
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(gop.frame_buffer as u64)).unwrap() + i as u64;
        let virtual_frame = relocate_frame(frame);

        unsafe {
            page_table_manager
                .map_to(virtual_frame, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator.deref_mut())
                .unwrap()
                .flush();
        }
    }

    let new_frame_buffer_pointer = unsafe {relocate_raw_pointer_mut(gop.frame_buffer)};
    let new_frame_buffer_renderer = FrameBufferRenderer::new(&GopInfo {
        frame_buffer: new_frame_buffer_pointer,
        ..boot_info.gop
    });
    { CONSOLE.get().unwrap().lock().set_renderer(new_frame_buffer_renderer); }

    // Switch to kernel paging table
    unsafe {
        Cr3::write(page_table_addr, Cr3Flags::PAGE_LEVEL_CACHE_DISABLE);
    }

    let page_table_manager = unsafe {OffsetPageTable::new(&mut *relocate_raw_pointer_mut(&mut *page_table), VIRTUAL_TO_PHYSICAL_OFFSET)};
    PAGE_TABLE_MANAGER.call_once(|| Mutex::new(page_table_manager));
}