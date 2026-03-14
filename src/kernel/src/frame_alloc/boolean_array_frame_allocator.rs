use crate::println;
use crate::utils::boolean_array::BooleanArray;
use crate::utils::human_bytes::human_bytes;
use log::info;
use uefi::table::boot::{MemoryMap, MemoryType};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

pub struct BooleanArrayFrameAllocator<'a> {
    boolean_array: BooleanArray<'a>,
    point: usize,
    total_memory_bytes: u64,
    reserved_memory_bytes: u64,
    free_memory_bytes: u64,
    total_pages_count: u64,
    last_page_index: u64,
}

#[allow(unused)]
impl<'a> BooleanArrayFrameAllocator<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        let mut boolean_array = BooleanArray::new(buffer);
        boolean_array.set_all();

        BooleanArrayFrameAllocator {
            boolean_array,
            point: 0,
            total_memory_bytes: 0,
            reserved_memory_bytes: 0,
            free_memory_bytes: 0,
            total_pages_count: 0,
            last_page_index: 0,
        }
    }

    pub fn read_from_memory_map(&mut self, memory_map: &MemoryMap) {
        for entry in memory_map.entries() {
            self.total_memory_bytes += entry.page_count * 4096;
            self.total_pages_count += entry.page_count;

            let last_page_index = entry.phys_start / 4096 + entry.page_count;
            if last_page_index > self.last_page_index {
                self.last_page_index = last_page_index;
            }

            if entry.ty == MemoryType::CONVENTIONAL {
                self.free_memory_bytes += entry.page_count * 4096;
                self.unlock_pages(
                    PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(entry.phys_start))
                        .unwrap(),
                    entry.page_count as usize,
                );
            } else {
                self.reserved_memory_bytes += entry.page_count * 4096;
            }
        }
    }

    fn lock_page(&mut self, frame: PhysFrame<Size4KiB>) {
        self.boolean_array.set(
            (frame.start_address().as_u64() / frame.size()) as usize,
            true,
        )
    }

    fn lock_pages(&mut self, frame_start: PhysFrame<Size4KiB>, page_count: usize) {
        for index in 0..page_count {
            self.lock_page(frame_start + index as u64);
        }
    }

    fn unlock_page(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = (frame.start_address().as_u64() / frame.size()) as usize;
        self.boolean_array.set(index, false);
        if self.point > index {
            self.point = index;
        }
    }

    fn unlock_pages(&mut self, frame_start: PhysFrame<Size4KiB>, page_count: usize) {
        for index in 0..page_count {
            self.unlock_page(frame_start + index as u64);
        }
    }

    pub fn request_page(&mut self) -> Result<PhysFrame<Size4KiB>, ()> {
        let mut index = self.point;
        while index < self.last_page_index as usize {
            if !self.boolean_array[index] {
                let frame = unsafe {
                    PhysFrame::<Size4KiB>::from_start_address_unchecked(PhysAddr::new(
                        (index * 4096) as u64,
                    ))
                };

                self.lock_page(frame);
                self.free_memory_bytes -= 4096;
                self.point = index + 1;
                return Ok(frame);
            }
            index += 1;
        }

        Err(())
    }

    pub fn free_page(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = (frame.start_address().as_u64() / frame.size()) as usize;
        if self.boolean_array[index] {
            self.unlock_page(frame);
            self.free_memory_bytes += 4096;
        }
    }

    pub fn get_total_memory_bytes(&self) -> u64 {
        self.total_memory_bytes
    }

    pub fn get_total_pages_count(&self) -> u64 {
        self.total_pages_count
    }

    pub fn get_free_memory_bytes(&self) -> u64 {
        self.free_memory_bytes
    }

    pub fn get_reserved_memory_bytes(&self) -> u64 {
        self.reserved_memory_bytes
    }

    pub fn print_stats(&self) {
        info!("Page count: {}", self.get_total_pages_count());
        info!(
            "Total memory: {}",
            human_bytes(self.get_total_memory_bytes() as f64)
        );
        info!(
            "Free memory: {}",
            human_bytes(self.get_free_memory_bytes() as f64)
        );
        info!(
            "Reserved memory: {}",
            human_bytes(self.get_reserved_memory_bytes() as f64)
        );
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for BooleanArrayFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.request_page().ok()
    }
}

impl<'a> FrameDeallocator<Size4KiB> for BooleanArrayFrameAllocator<'a> {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.free_page(frame);
    }
}
