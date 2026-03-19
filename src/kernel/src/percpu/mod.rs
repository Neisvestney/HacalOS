mod gsbase;
pub mod init;

use core::cell::UnsafeCell;
use x2apic::lapic::LocalApic;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;

#[repr(C)]
pub struct PerCpu {
    pub this: *mut PerCpu,
    pub cpu_id: u32,
    pub local_apic: UnsafeCell<LocalApic>,
    pub gdt: &'static mut GlobalDescriptorTable,
    pub tss: &'static mut TaskStateSegment,
}

// SAFETY: PerCpu must be used only in cpu that associated with struct instance
unsafe impl Send for PerCpu {}
unsafe impl Sync for PerCpu {}

pub unsafe fn current() -> &'static PerCpu {
    unsafe { &*gsbase::get() }
}

pub unsafe fn lapic() -> &'static mut LocalApic {
    unsafe { &mut *(*gsbase::get()).local_apic.get() }
}
