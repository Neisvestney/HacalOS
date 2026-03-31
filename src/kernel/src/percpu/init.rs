use crate::percpu::{PerCpu, gsbase};
use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU64};
use spin::mutex::Mutex;
use x2apic::lapic::LocalApic;
use x86_64::VirtAddr;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;

pub fn init_per_cpu(
    local_apic: LocalApic,
    gdt: &'static mut GlobalDescriptorTable,
    tss: &'static mut TaskStateSegment,
    kernel_stack_top: VirtAddr,
) {
    let cpu_id = unsafe { local_apic.id() };

    let percpu = Box::leak(Box::new(PerCpu {
        this: core::ptr::null_mut(),
        cpu_id,
        local_apic: UnsafeCell::new(local_apic),
        gdt,
        tss,
        kernel_stack_top,
        user_rsp: VirtAddr::zero(),
        scheduling_disabled: AtomicBool::new(true),
        current_thread_ticks_left: AtomicU64::new(0),
        current_thread_context: Mutex::new(None),
        syscall_stack: VirtAddr::zero(),
    }));

    percpu.this = percpu as *mut PerCpu;

    unsafe {
        gsbase::set(percpu);
    }
}
