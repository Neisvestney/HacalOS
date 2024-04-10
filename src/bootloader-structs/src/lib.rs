#![no_std]

#[cfg(target_arch = "x86_64")]
pub type KernelMainFunction = extern "sysv64" fn(boot_info: &BootInfo) -> usize;

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct BootInfo<'a> {
    pub gop: GopInfo,
    pub font: &'a[u8],
}

unsafe impl Send for BootInfo<'_> {}
unsafe impl Sync for BootInfo<'_> {}

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct GopInfo {
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}

unsafe impl Send for GopInfo {}
unsafe impl Sync for GopInfo {}