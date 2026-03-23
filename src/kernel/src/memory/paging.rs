use alloc::vec::Vec;
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use crate::utils::relocate::{relocate_addr, relocate_frame, relocate_raw_pointer, relocate_raw_pointer_mut};
use crate::{CONSOLE, FRAME_ALLOCATOR, PAGE_TABLE_MAPPER, VIRTUAL_TO_PHYSICAL_OFFSET, println};
use bootloader_structs::{GopInfo, KernelMapEntry};
use core::ops::DerefMut;
use core::ptr::slice_from_raw_parts;
use log::{info, warn};
use psf2::Font;
use spin::Mutex;
use uefi::table::boot::{MemoryMap, MemoryType};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::mapper::{CleanUp, MapToError};
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
use crate::memory::heap::ALLOCATOR;
use crate::utils::human_bytes::human_bytes;

pub fn init_paging(
    memory_map: &MemoryMap,
    kernel_memory_map: &'static [KernelMapEntry],
    gop: &GopInfo,
    font: &'static [u8],
) {
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    let page_table_addr = frame_allocator.request_page().unwrap();
    let page_table = page_table_addr.start_address().as_u64() as *mut PageTable;
    let mut page_table_manager =
        unsafe { OffsetPageTable::new(&mut *page_table, VirtAddr::new(0)) };

    for memory_map_entry in memory_map.entries() {
        for i in 0..memory_map_entry.page_count {
            let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(
                memory_map_entry.phys_start,
            ))
            .unwrap()
                + i;
            let virtual_frame =
                Page::from_start_address(VirtAddr::new(memory_map_entry.phys_start)).unwrap() + i;
            let virtual_frame_higher_half = Page::from_start_address(VirtAddr::new(
                memory_map_entry.phys_start + VIRTUAL_TO_PHYSICAL_OFFSET.as_u64(),
            ))
            .unwrap()
                + i;

            unsafe {
                page_table_manager
                    .map_to(
                        virtual_frame,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        frame_allocator.deref_mut(),
                    )
                    .unwrap()
                    .flush();

                page_table_manager
                    .map_to(
                        virtual_frame_higher_half,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        frame_allocator.deref_mut(),
                    )
                    .unwrap()
                    .flush();
            }
        }
    }
    for memory_map_entry in kernel_memory_map {
        for i in 0..memory_map_entry.page_count {
            unsafe {
                page_table_manager
                    .map_to(
                        memory_map_entry.page + i,
                        memory_map_entry.frame + i,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        frame_allocator.deref_mut(),
                    )
                    .unwrap()
                    .flush();
            }
        }
    }

    // Relocate framebuffer
    for i in 0..(gop.frame_buffer_size + 4095) / 4096 {
        let frame =
            PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(gop.frame_buffer as u64))
                .unwrap()
                + i as u64;
        let virtual_frame = relocate_frame(frame);

        unsafe {
            page_table_manager
                .map_to(
                    virtual_frame,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    frame_allocator.deref_mut(),
                )
                .unwrap()
                .flush();
        }
    }

    unsafe {
        frame_allocator.relocate_buffer();
    }

    // Switch to kernel paging table
    unsafe {
        Cr3::write(page_table_addr, Cr3Flags::PAGE_LEVEL_CACHE_DISABLE);
    }

    let page_table_manager = unsafe {
        OffsetPageTable::new(
            &mut *relocate_raw_pointer_mut(&mut *page_table),
            VIRTUAL_TO_PHYSICAL_OFFSET,
        )
    };
    PAGE_TABLE_MAPPER.call_once(|| Mutex::new(page_table_manager));

    {
        let new_frame_buffer_pointer = unsafe { relocate_raw_pointer_mut(gop.frame_buffer) };
        let new_frame_buffer_renderer = FrameBufferRenderer::new(&GopInfo {
            frame_buffer: new_frame_buffer_pointer,
            ..*gop
        });

        let new_font_slice = unsafe {&*relocate_raw_pointer(font)};
        let new_font = Font::new(new_font_slice).unwrap();

        let mut console = CONSOLE
            .get()
            .unwrap()
            .lock();

        console.frame_buffer_renderer = new_frame_buffer_renderer;
        console.font = new_font;
    }
}

pub fn unmap_lower_half(memory_map: &MemoryMap) {
    let mut mapper = PAGE_TABLE_MAPPER.get().unwrap().lock();
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    let mut relocated_memory_map = Vec::new();
    for memory_map_entry in memory_map.entries() {
        relocated_memory_map.push(*memory_map_entry);
    }

    for memory_map_entry in relocated_memory_map {
        for i in 0..memory_map_entry.page_count {
            let virtual_frame =
                Page::<Size4KiB>::from_start_address(VirtAddr::new(memory_map_entry.phys_start)).unwrap() + i;

            let result = unsafe { mapper.unmap(virtual_frame) };

            match result {
                Ok((_, flush)) => flush.flush(),
                Err(e) => warn!("Err {:?}", e),
            }
        }
    }

    unsafe {
        mapper.clean_up(frame_allocator.deref_mut());
    }
}

pub fn alloc_memory_range(
    page_range: PageRangeInclusive<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();
    let mut mapper = PAGE_TABLE_MAPPER.get().unwrap().lock();

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator.deref_mut())?
                .flush()
        };
    }

    Ok(())
}

pub fn map_mmio_single_page(phys_addr: PhysAddr) -> Result<(), MapToError<Size4KiB>> {
    let mut mapper = PAGE_TABLE_MAPPER.get().unwrap().lock();
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE
        | PageTableFlags::NO_CACHE;
    let frame = PhysFrame::<Size4KiB>::containing_address(phys_addr);
    let virtual_page = relocate_frame(frame);

    unsafe {
        mapper
            .map_to(virtual_page, frame, flags, frame_allocator.deref_mut())?
            .flush();
    }

    Ok(())
}
