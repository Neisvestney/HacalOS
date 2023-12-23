#![no_std]

#[cfg(target_arch = "x86_64")]
pub type KernelMainFunction = extern "sysv64" fn(boot_info: &BootInfo) -> usize;

#[repr(C)]
pub struct BootInfo {
    pub i: u32
}