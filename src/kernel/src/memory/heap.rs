use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{Page, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;

use crate::{HEAD_SIZE, HEAP_START};
use crate::memory::paging::alloc_memory_range;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_end = HEAP_START + HEAD_SIZE - 1u64;
        let heap_start_page = Page::containing_address(HEAP_START);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    alloc_memory_range(page_range);

    unsafe {
        ALLOCATOR.lock().init(HEAP_START.as_u64() as *mut u8, HEAD_SIZE);
    }

    Ok(())
}