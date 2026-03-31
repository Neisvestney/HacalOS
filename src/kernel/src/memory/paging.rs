use crate::frame_alloc::boolean_array_frame_allocator::{BooleanArrayFrameAllocator, LowerHalfBooleanArrayFrameAllocator};
use crate::render::frame_buffer_renderer::FrameBufferRenderer;
use crate::utils::relocate::{relocate_frame, relocate_raw_pointer, relocate_raw_pointer_mut};
use crate::{CONSOLE, FRAME_ALLOCATOR, PAGE_TABLE_MAPPER, VIRTUAL_TO_PHYSICAL_OFFSET};
use alloc::vec::Vec;
use bootloader_structs::{GopInfo, KernelMapEntry};
use core::ops::DerefMut;
use log::{info, warn};
use psf2::Font;
use spin::Mutex;
use uefi::table::boot::MemoryMap;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::mapper::{CleanUp, MapToError};
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub fn init_paging(
    memory_map: &MemoryMap,
    kernel_memory_map: &'static [KernelMapEntry],
    gop: &GopInfo,
    font: &'static [u8],
) {
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    let page_table_addr = frame_allocator.request_page_zeroed_lower_half().unwrap();
    let page_table = page_table_addr.start_address().as_u64() as *mut PageTable;
    let mut page_table_manager =
        unsafe { OffsetPageTable::new(&mut *page_table, VirtAddr::new(0)) };

    let mut lower_half_frame_allocator = LowerHalfBooleanArrayFrameAllocator(&mut frame_allocator);

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
                        &mut lower_half_frame_allocator,
                    )
                    .unwrap()
                    .flush();

                page_table_manager
                    .map_to(
                        virtual_frame_higher_half,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut lower_half_frame_allocator,
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
                        &mut lower_half_frame_allocator,
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
                    &mut lower_half_frame_allocator,
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
        write_cr3(page_table_addr);
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

        let new_font_slice = unsafe { &*relocate_raw_pointer(font) };
        let new_font = Font::new(new_font_slice).unwrap();

        let mut console = CONSOLE.get().unwrap().lock();

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
                Page::<Size4KiB>::from_start_address(VirtAddr::new(memory_map_entry.phys_start))
                    .unwrap()
                    + i;

            let result = mapper.unmap(virtual_frame);

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
    page_table_mapper: &mut impl Mapper<Size4KiB>,
    flush: bool,
    user: bool,
) -> Result<(), MapToError<Size4KiB>> {
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    for page in page_range {
        let frame = frame_allocator
            .request_page_zeroed()
            .map_err(|_| MapToError::FrameAllocationFailed)?;
        let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        if user {
            flags |= PageTableFlags::USER_ACCESSIBLE;
        }

        unsafe {
            let mapper_flush =
                page_table_mapper.map_to(page, frame, flags, frame_allocator.deref_mut())?;

            if flush {
                mapper_flush.flush();
            } else {
                mapper_flush.ignore();
            }
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

pub unsafe fn write_cr3(page_table_addr: PhysFrame) {
    unsafe {
        Cr3::write(page_table_addr, Cr3Flags::PAGE_LEVEL_CACHE_DISABLE);
    }
}

pub fn new_page_table(
    frame_allocator: &mut BooleanArrayFrameAllocator,
) -> (PhysFrame, OffsetPageTable<'static>) {
    let page_table_frame = frame_allocator.request_page_zeroed().unwrap();
    let page_table_addr = page_table_frame.start_address().as_u64();
    let page_table = page_table_addr as *mut PageTable;
    let page_table_manager = unsafe {
        OffsetPageTable::new(
            &mut *relocate_raw_pointer_mut(page_table),
            VIRTUAL_TO_PHYSICAL_OFFSET,
        )
    };

    (page_table_frame, page_table_manager)
}

pub fn copy_kernel_mapping(target: &mut PageTable, from: &PageTable) {
    for i in 256..512 {
        target[i] = from[i].clone();
    }
}

pub unsafe fn page_table_mapper_from_table_pointer(
    page_table_pointer: *mut PageTable,
) -> OffsetPageTable<'static> {
    unsafe { OffsetPageTable::new(&mut *page_table_pointer, VIRTUAL_TO_PHYSICAL_OFFSET) }
}
