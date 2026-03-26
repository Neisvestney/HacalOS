use crate::memory::paging::alloc_memory_range;
use crate::{VIRTUAL_MEMORY_REGION_PAGES_COUNT, VIRTUAL_MEMORY_REGION_START};
use core::ptr::slice_from_raw_parts_mut;
use spin::{Mutex, Once};
use x86_64::structures::paging::mapper::{MapToError, UnmapError};
use x86_64::structures::paging::{Mapper, Page, Size4KiB};

pub static KERNEL_VIRTUAL_MEMORY_ALLOCATOR: Once<Mutex<VirtualMemoryAllocator>> = Once::new();

#[derive(Debug, Clone)]
pub struct VirtualMemoryAllocator {
    region_start: Page<Size4KiB>,
    region_pages_count: u64,
    pages_allocated: u64,
    flush: bool,
}

impl VirtualMemoryAllocator {
    pub fn new(region_start: Page<Size4KiB>, region_pages_count: u64, flush: bool) -> Self {
        VirtualMemoryAllocator {
            region_start,
            region_pages_count,
            pages_allocated: 0,
            flush,
        }
    }

    pub fn alloc_pages(
        &mut self,
        count: u64,
        page_table_mapper: &mut impl Mapper<Size4KiB>,
    ) -> Result<*const [u8], VirtualMemoryAllocatorError> {
        if count > self.region_pages_count - self.pages_allocated {
            return Err(VirtualMemoryAllocatorError::OutOfMemory);
        }

        let page_range = {
            let start_page = self.region_start + self.pages_allocated;
            let end_page = start_page + count - 1;
            Page::range_inclusive(start_page, end_page)
        };

        self.pages_allocated += count;

        alloc_memory_range(page_range, page_table_mapper, self.flush)
            .map_err(VirtualMemoryAllocatorError::MapToError)?;

        Ok(slice_from_raw_parts_mut(
            page_range.start.start_address().as_mut_ptr(),
            page_range.size() as usize,
        ))
    }

    pub fn alloc_pages_with_protection_page(
        &mut self,
        count: u64,
        page_table_mapper: &mut impl Mapper<Size4KiB>,
    ) -> Result<(*const [u8], Page<Size4KiB>), VirtualMemoryAllocatorError> {
        if count + 1 > self.region_pages_count - self.pages_allocated {
            return Err(VirtualMemoryAllocatorError::OutOfMemory);
        }

        let protection_page = self.region_start + self.pages_allocated;
        let page_range = {
            let start_page = self.region_start + self.pages_allocated + 1;
            let end_page = start_page + count - 1;
            Page::range_inclusive(start_page, end_page)
        };

        self.pages_allocated += count + 1;

        match page_table_mapper.unmap(protection_page) {
            Ok((_, mapper_flush)) => {
                if self.flush {
                    mapper_flush.flush();
                } else {
                    mapper_flush.ignore();
                }
            }
            Err(UnmapError::PageNotMapped) => {}
            Err(e) => return Err(VirtualMemoryAllocatorError::UnmapError(e)),
        };

        alloc_memory_range(page_range, page_table_mapper, self.flush)
            .map_err(VirtualMemoryAllocatorError::MapToError)?;

        Ok((
            slice_from_raw_parts_mut(
                page_range.start.start_address().as_mut_ptr(),
                page_range.size() as usize,
            ),
            protection_page,
        ))
    }
}

#[derive(Debug)]
pub enum VirtualMemoryAllocatorError {
    MapToError(MapToError<Size4KiB>),
    UnmapError(UnmapError),
    OutOfMemory,
}

pub fn init_kernel_virtual_memory_allocator() {
    KERNEL_VIRTUAL_MEMORY_ALLOCATOR.call_once(|| {
        Mutex::new(VirtualMemoryAllocator::new(
            VIRTUAL_MEMORY_REGION_START,
            VIRTUAL_MEMORY_REGION_PAGES_COUNT,
            true,
        ))
    });
}
