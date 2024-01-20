#![no_std]

#[cfg(target_arch = "x86_64")]
pub type KernelMainFunction = extern "sysv64" fn(boot_info: &BootInfo) -> usize;

#[repr(C)]
pub struct BootInfo {
    pub gop: GopInfo,
}

#[repr(C)]
pub struct GopInfo {
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}