use core::ops::DerefMut;
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::Size4KiB;
use x86_64::structures::paging::mapper::MapToError;

use crate::PAGE_TABLE_MAPPER;
use crate::memory::virtual_memory_allocator::KERNEL_VIRTUAL_MEMORY_ALLOCATOR;

const HEAP_PAGES_COUNT: u64 = 10;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_kernel_heap() -> Result<(), MapToError<Size4KiB>> {
    let mut page_table_mapper = PAGE_TABLE_MAPPER.get().unwrap().lock();
    let heap = KERNEL_VIRTUAL_MEMORY_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .alloc_pages(HEAP_PAGES_COUNT, page_table_mapper.deref_mut())
        .expect("Failed allocated page for heap");

    unsafe {
        ALLOCATOR.lock().init(heap as *mut u8, heap.len());
    }

    Ok(())
}
