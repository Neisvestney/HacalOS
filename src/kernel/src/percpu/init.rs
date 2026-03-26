use crate::percpu::{PerCpu, gsbase};
use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU64};
use x2apic::lapic::LocalApic;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;

pub fn init_per_cpu(
    local_apic: LocalApic,
    gdt: &'static mut GlobalDescriptorTable,
    tss: &'static mut TaskStateSegment,
) {
    let cpu_id = unsafe { local_apic.id() };

    let percpu = Box::leak(Box::new(PerCpu {
        this: core::ptr::null_mut(),
        cpu_id,
        local_apic: UnsafeCell::new(local_apic),
        gdt,
        tss,
        start_scheduling_on_next_tick: AtomicBool::new(false),
        current_thread_ticks_left: AtomicU64::new(0),
        current_thread_context: UnsafeCell::new(None),
    }));

    percpu.this = percpu as *mut PerCpu;

    unsafe {
        gsbase::set(percpu);
    }
}
