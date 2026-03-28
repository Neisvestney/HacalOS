use crate::memory::paging::page_table_mapper_from_table_pointer;
use crate::process::Process;
use crate::utils::dump_page_table::dump_lower_half;
use crate::{FRAME_ALLOCATOR, LOWER_HALF_PAGE_RANGE};
use core::ops::DerefMut;
use core::ptr;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameDeallocator, Mapper, PhysFrame};
use x86_64::structures::paging::mapper::{CleanUp, UnmapError};

pub fn unload_process_from_memory(process: &mut Process) {
    // info!("MemoryMap: {:#?}", process.memory_map);

    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();
    {
        let mut page_table_manager =
            unsafe { page_table_mapper_from_table_pointer(process.page_table) };

        for memory_map_entry in &process.memory_map {
            for page in memory_map_entry.page_range {
                match page_table_manager.unmap(page) {
                    Ok((_, mapper_flush)) => mapper_flush.ignore(),
                    Err(UnmapError::PageNotMapped) => {}
                    Err(e) => panic!(
                        "Unexpected error in unmapping page {:?} while unloading process\n{:?}\n{:#?}",
                        page, e, &process
                    ),
                }
            }
        }

        unsafe {
            page_table_manager.clean_up_addr_range(LOWER_HALF_PAGE_RANGE, frame_allocator.deref_mut());
        }

        if cfg!(debug_assertions) {
            for i in 0..256 {
                let e = &page_table_manager.level_4_table()[i];
                if !e.is_unused() {
                    dump_lower_half(&page_table_manager);
                    panic!("Page table entry is not unused, i: {}", i);
                }
            }
        }
    }

    unsafe {
        frame_allocator.deallocate_frame(process.page_table_phys_frame);
    }
    process.memory_map.clear();
    process.threads.clear();
    process.page_table_phys_frame = PhysFrame::from_start_address(PhysAddr::zero()).unwrap();
    process.page_table = ptr::null_mut();
}
