#![no_std]

use uefi::prelude::SystemTable;
use uefi::table::boot::MemoryMap;
use uefi::table::Runtime;
use x86_64::structures::paging::{Page, PhysFrame, Size4KiB};

#[cfg(target_arch = "x86_64")]
#[allow(improper_ctypes_definitions)]
pub type KernelMainFunction = extern "sysv64" fn(boot_info: BootInfo) -> usize;

#[derive(Debug)]
#[repr(C)]
#[allow(improper_ctypes_definitions)]
pub struct BootInfo {
    pub gop: GopInfo,
    pub font: &'static [u8],
    pub frame_allocator_buffer: &'static mut [u8],
    pub memory_map: Option<MemoryMap<'static>>,
    pub kernel_memory_map: &'static [KernelMapEntry],
    pub runtime_system_table: Option<SystemTable<Runtime>>,
    pub rsbp_address: Option<usize>,
}

#[derive(Debug)]
pub struct KernelMapEntry {
    pub page: Page<Size4KiB>,
    pub frame: PhysFrame<Size4KiB>,
    pub page_count: u64,
}

unsafe impl Send for BootInfo {}

unsafe impl Sync for BootInfo {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(C)]
pub struct GopInfo {
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}

unsafe impl Send for GopInfo {}

unsafe impl Sync for GopInfo {}
