use crate::percpu::{PerCpu, gsbase};
use alloc::boxed::Box;
use core::cell::UnsafeCell;
use x2apic::lapic::LocalApic;

pub fn init_per_cpu(local_apic: LocalApic) {
    let cpu_id = unsafe { local_apic.id() };

    let percpu = Box::leak(Box::new(PerCpu {
        this: core::ptr::null_mut(), // заполним ниже
        cpu_id,
        local_apic: UnsafeCell::new(local_apic),
    }));

    percpu.this = percpu as *mut PerCpu;

    unsafe {
        gsbase::set(percpu);
    }
}
