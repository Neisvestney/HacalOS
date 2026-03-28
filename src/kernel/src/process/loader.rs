use crate::memory::paging::{copy_kernel_mapping, new_page_table};
use crate::process::{Process, ProcessId};
use crate::utils::align::{align_down, align_up};
use crate::utils::relocate::relocate_frame;
use crate::{FRAME_ALLOCATOR, PAGE_TABLE_MAPPER};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::DerefMut;
use goblin::elf64::header;
use goblin::elf::Elf;
use goblin::elf64::program_header::PT_LOAD;
use thiserror::Error;
use uefi::table::boot::PAGE_SIZE;
use x86_64::VirtAddr;
use x86_64::structures::paging::{Mapper, Page, PageTable, PageTableFlags};
use x86_64::structures::paging::page::PageRangeInclusive;
use crate::process::memory_map::{ProcessMemoryMapEntry, ProcessMemoryMapEntryType};

pub fn load_program_to_memory(
    process_id: ProcessId,
    name: String,
    bytes: &[u8],
) -> Result<Process, ProgramLoadError> {
    let elf = Elf::parse(bytes)?;

    if elf.header.e_type != header::ET_EXEC {
        return Err(ProgramLoadError::NotExecutable);
    }

    if elf.header.e_machine != header::EM_X86_64 {
        return Err(ProgramLoadError::ArchitectureNotSupported);
    }

    let name = Arc::new(name);

    let kernel_page_table_manager = PAGE_TABLE_MAPPER.get().unwrap().lock();
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    let (page_table_phys_frame, mut page_table_manager) =
        new_page_table(frame_allocator.deref_mut());
    copy_kernel_mapping(
        page_table_manager.level_4_table_mut(),
        kernel_page_table_manager.level_4_table(),
    );

    let mut process_memory_map = Vec::new();

    for ph in &elf.program_headers {
        if ph.p_type == PT_LOAD {
            let bytes_range = &bytes[ph.file_range()];

            let align = if ph.p_align == 0 {
                PAGE_SIZE as u64
            } else {
                ph.p_align
            };

            let seg_start = align_down(ph.p_vaddr, align);
            let seg_end = align_up(ph.p_vaddr + ph.p_memsz, align);

            let total_size = seg_end - seg_start;
            let pages_count = total_size.div_ceil(PAGE_SIZE as u64);

            let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
            if ph.is_write() {
                flags |= PageTableFlags::WRITABLE;
            }
            if !ph.is_executable() {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            let page_start = VirtAddr::new(seg_start);
            let start_page = Page::containing_address(page_start);
            let end_page = start_page + pages_count - 1;
            let page_range = Page::range_inclusive(start_page, end_page);
            let offset_in_segment = ph.p_vaddr - seg_start;
            process_memory_map.push(ProcessMemoryMapEntry::new(page_range, ProcessMemoryMapEntryType::new_file(name.clone(), offset_in_segment, ph.p_offset, ph.p_filesz)));

            for page_idx in 0..pages_count {
                let virt_addr = VirtAddr::new(seg_start + page_idx * PAGE_SIZE as u64);
                let page = Page::containing_address(virt_addr);
                let frame = frame_allocator.request_page_zeroed().unwrap();

                unsafe {
                    page_table_manager
                        .map_to(page, frame, flags, frame_allocator.deref_mut())
                        .expect("Failed to map page table")
                        .ignore();
                }

                let kernel_addr = relocate_frame(frame).start_address().as_mut_ptr::<u8>();

                let page_start = seg_start + page_idx * PAGE_SIZE as u64;
                let offset_in_segment = (page_start as i64 - ph.p_vaddr as i64) as isize;

                unsafe {
                    let file_start = 0isize.max(offset_in_segment) as usize;
                    let mem_start = 0isize.max(-offset_in_segment) as usize;

                    let max_copy = PAGE_SIZE - mem_start;

                    let file_bytes_left = bytes_range.len().saturating_sub(file_start);
                    let to_copy = core::cmp::min(file_bytes_left, max_copy);

                    if to_copy > 0 {
                        core::ptr::copy_nonoverlapping(
                            bytes_range.as_ptr().add(file_start),
                            kernel_addr.add(mem_start),
                            to_copy,
                        );
                    }
                }
            }
        }
    }

    Ok(Process::new(
        process_id,
        name,
        page_table_manager.level_4_table_mut() as *mut PageTable,
        page_table_phys_frame,
        VirtAddr::new(elf.entry),
        process_memory_map,
    ))
}

#[derive(Error, Debug)]
pub enum ProgramLoadError {
    #[error("Program not executable")]
    NotExecutable,
    #[error("Architecture not supported")]
    ArchitectureNotSupported,
    #[error("Failed to parse elf")]
    ElfParseError(#[from] goblin::error::Error),
}