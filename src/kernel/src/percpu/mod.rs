mod gsbase;
pub mod init;

use crate::scheduler::thread_context::ThreadContext;
use alloc::boxed::Box;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::mem::offset_of;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use x2apic::lapic::LocalApic;
use x86_64::VirtAddr;
use x86_64::instructions::hlt;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;

#[repr(C)]
pub struct PerCpu {
    pub this: *mut PerCpu,
    pub cpu_id: u32,
    pub local_apic: UnsafeCell<LocalApic>,
    pub gdt: &'static mut GlobalDescriptorTable,
    pub tss: &'static mut TaskStateSegment,
    pub kernel_stack_top: VirtAddr,
    pub user_rsp: VirtAddr,

    pub start_scheduling_on_next_tick: AtomicBool,
    pub current_thread_ticks_left: AtomicU64,
    pub current_thread_context: UnsafeCell<Option<Box<ThreadContext>>>,
}

impl PerCpu {
    pub const KERNEL_STACK_TOP_STRUCT_OFFSET: usize = offset_of!(PerCpu, kernel_stack_top);
    pub const USER_RSP_STRUCT_OFFSET: usize = offset_of!(PerCpu, user_rsp);
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

pub fn start_scheduling_on_next_tick() -> ! {
    unsafe {
        current()
            .start_scheduling_on_next_tick
            .store(true, Ordering::Relaxed);
    }

    loop {
        hlt();
    }
}
