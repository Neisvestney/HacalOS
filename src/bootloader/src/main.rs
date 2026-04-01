#![no_main]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader_structs::{BootInfo, GopInfo, KernelMainFunction, KernelMapEntry};
use core::{mem, ptr, slice};
use elf_rs::{Elf, ElfFile, ProgramType};
use log::info;
use uefi::fs::FileSystem;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::table::cfg::{ACPI2_GUID, ACPI_GUID};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page::Size4KiB;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
    PhysFrame,
};
use x86_64::{PhysAddr, VirtAddr};

struct UefiPageAllocator<'a> {
    bt: &'a BootServices,
}

unsafe impl<'a> FrameAllocator<Size4KiB> for UefiPageAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        match self
            .bt
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
        {
            Ok(addr) => Some(PhysFrame::from_start_address(PhysAddr::new(addr)).unwrap()),
            Err(_) => None,
        }
    }
}

impl<'a> FrameDeallocator<Size4KiB> for UefiPageAllocator<'a> {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.bt
            .free_pages(frame.start_address().as_u64(), 1)
            .unwrap();
    }
}

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    // BootServices borrow
    let (
        (kernel_main, kernel_memory_map),
        level_4_table_flags,
        new_page_table_addr,
        gop,
        font,
        inithfs_bytes,
        frame_allocator_buffer,
    ) = {
        let bt = system_table.boot_services();

        // Setup new page table as copy of current
        let (level_4_table, level_4_table_flags) = Cr3::read();
        let current_page_table = level_4_table.start_address().as_u64() as *mut PageTable;

        let new_page_table_addr = bt
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .unwrap();
        let new_page_table_table = new_page_table_addr as *mut PageTable;

        unsafe {
            ptr::copy(current_page_table, new_page_table_table, 1);
        }
        let new_page_table_table = unsafe { &mut *new_page_table_table };

        let mut offset_page_table =
            unsafe { OffsetPageTable::new(new_page_table_table, VirtAddr::new(0)) };

        // Init SimpleFileSystem
        let fs_handle = bt.get_handle_for_protocol::<SimpleFileSystem>().unwrap();
        let fs = bt
            .open_protocol_exclusive::<SimpleFileSystem>(fs_handle)
            .unwrap();
        let mut fs = FileSystem::new(fs);

        // Loader kernel binary into memory
        let kernel = {
            let mut allocator = UefiPageAllocator { bt };
            load_kernel(&mut fs, bt, &mut offset_page_table, &mut allocator)
        };

        // Init gop
        let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = bt
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
            .unwrap();

        let resolution = gop.current_mode_info().resolution();

        let gop_info = GopInfo {
            frame_buffer: gop.frame_buffer().as_mut_ptr(),
            frame_buffer_size: gop.frame_buffer().size(),
            horizontal_resolution: resolution.0,
            vertical_resolution: resolution.1,
        };

        let font = fs.read(cstr16!("spleen-8x16-v2.psf")).unwrap();
        let inithfs_bytes = fs.read(cstr16!("inithfs")).unwrap();

        // Memory map things
        let memory_map_size = bt.memory_map_size();
        let memory_map_buffer_size = memory_map_size.map_size + memory_map_size.entry_size * 3;
        let memory_map_buffer = bt
            .allocate_pool(MemoryType::LOADER_DATA, memory_map_buffer_size)
            .unwrap();
        let memory_map_buffer =
            unsafe { slice::from_raw_parts_mut(memory_map_buffer, memory_map_buffer_size) };
        let memory_map = bt.memory_map(memory_map_buffer).unwrap();

        let last_memory_entry = memory_map
            .entries()
            .filter(|e| e.ty != MemoryType::RESERVED)
            .max_by(|a, b|
                (a.phys_start + a.page_count * 4096).cmp(&(b.phys_start + b.page_count * 4096))
            )
            .unwrap();
        let total_memory_page_count =
            last_memory_entry.phys_start / 4096 + last_memory_entry.page_count;
        let frame_allocator_buffer_size = (total_memory_page_count as usize / 8) + 1;
        let frame_allocator_buffer = bt
            .allocate_pool(MemoryType::LOADER_DATA, frame_allocator_buffer_size)
            .unwrap();
        let frame_allocator_buffer = unsafe {
            slice::from_raw_parts_mut(frame_allocator_buffer, frame_allocator_buffer_size)
        };

        (
            kernel,
            level_4_table_flags,
            new_page_table_addr,
            gop_info,
            font,
            inithfs_bytes,
            frame_allocator_buffer,
        )
    };

    let rsbp_address = find_rsdp(&system_table);

    let mut boot_info = Box::new(BootInfo {
        gop,
        font: Vec::leak(font),
        inithfs_bytes: Vec::leak(inithfs_bytes),
        frame_allocator_buffer,
        memory_map: None,
        runtime_system_table: None,
        kernel_memory_map: Vec::leak(kernel_memory_map),
        rsbp_address,
        //runtime_services: system_table.runtime_services().clone(),
    });

    let (runtime_system_table, memory_map) =
        system_table.exit_boot_services(MemoryType::LOADER_DATA);

    boot_info.runtime_system_table = Some(runtime_system_table);

    boot_info.memory_map = Some(memory_map);

    unsafe {
        Cr3::write(
            PhysFrame::from_start_address(PhysAddr::new(new_page_table_addr)).unwrap(),
            level_4_table_flags,
        );
        kernel_main(*boot_info);
    }

    Status::SUCCESS
}

fn load_kernel(
    fs: &mut FileSystem,
    bt: &BootServices,
    mapper: &mut impl Mapper<Size4KiB>,
    allocator: &mut impl FrameAllocator<Size4KiB>,
) -> (KernelMainFunction, Vec<KernelMapEntry>) {
    let kernel_file = fs.read(cstr16!("kernel.elf")).unwrap();

    let elf = Elf::from_bytes(&kernel_file).unwrap();

    info!("Kernel entry point: {:#x}", elf.entry_point());

    let mut kernel_memory_map = Vec::<KernelMapEntry>::new();

    for p in elf.program_header_iter() {
        if p.ph_type() == ProgramType::LOAD {
            info!("Mapping {:x?}", p);
            let pages_count = (p.memsz() + 0x1000 - 1) / 0x1000;
            let allocated_pages_addr = bt
                .allocate_pages(
                    AllocateType::AnyPages,
                    MemoryType::LOADER_DATA,
                    pages_count as usize,
                )
                .unwrap();
            let content = p.content().unwrap();
            copy_to_physical_address(content, allocated_pages_addr);

            kernel_memory_map.push(KernelMapEntry {
                frame: PhysFrame::from_start_address(PhysAddr::new(allocated_pages_addr)).unwrap(),
                page: Page::from_start_address(VirtAddr::new(p.paddr())).unwrap(),
                page_count: pages_count,
            });

            for i in 0..pages_count {
                let physical_addr = allocated_pages_addr + (i * 0x1000);
                let virtual_addr = p.paddr() + (i * 0x1000);
                // info!("Map {:#x} to {:#x}", physical_addr, virtual_addr);
                let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr));
                let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(physical_addr));
                // info!("Map {:?} to {:?}", page, frame);
                unsafe {
                    if let Ok(r) = mapper.map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        allocator,
                    ) {
                        r.flush()
                    }
                }
            }
        }
    }

    let ptr = elf.entry_point() as *const ();
    let function_ptr: KernelMainFunction = unsafe { mem::transmute(ptr) };

    info!("Kernel loaded");

    (function_ptr, kernel_memory_map)
}

fn copy_to_physical_address(src: &[u8], physical_address: u64) {
    let dest_ptr = physical_address as *mut u8;

    unsafe {
        for (offset, byte) in src.iter().enumerate() {
            let ptr = dest_ptr.add(offset);
            ptr.write(*byte);
        }
    }
}

fn find_rsdp(st: &SystemTable<Boot>) -> Option<usize> {
    for entry in st.config_table() {
        if entry.guid == ACPI2_GUID {
            return Some(entry.address as usize);
        }
    }
    for entry in st.config_table() {
        if entry.guid == ACPI_GUID {
            return Some(entry.address as usize);
        }
    }
    None
}