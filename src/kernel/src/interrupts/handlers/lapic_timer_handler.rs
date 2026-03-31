use crate::gdt::segment_selectors;
use crate::interrupts::iret_wit_context::iret_with_context;
use crate::memory::paging::write_cr3;
use crate::percpu;
use crate::percpu::PerCpu;
use crate::scheduler::SCHEDULE_TICKS;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::schedule_on_interrupt::{timer_schedule_next, timer_schedule_tick};
use crate::scheduler::thread_context::ThreadContext;
use alloc::boxed::Box;
use core::arch::{asm, naked_asm};
use core::hint;
use core::ops::DerefMut;
use core::sync::atomic::Ordering;
use log::info;
use spin::MutexGuard;
use volatile::VolatilePtr;
use x86_64::instructions::hlt;
use crate::hpet::HPET;
use crate::utils::with_swaped_gs::with_swaped_gs;

#[unsafe(naked)]
pub extern "C" fn lapic_timer_entry() -> ! {
    naked_asm!(
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rbp",
        "push rdi",
        "push rsi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // rdi = &CPURegistriesContext
        "mov rdi, rsp",
        "call {func}",
        "hlt",
        func = sym lapic_timer_handler,
        options()
    )
}

pub unsafe extern "C" fn lapic_timer_handler(ctx: &mut CpuRegistriesContext) {
    let code_segment = ctx.stack_frame.code_segment;
    with_swaped_gs(|| {
        let percpu = unsafe { percpu::current() };

        let ticks_left = timer_schedule_tick(percpu);
        
       if let Some(hpet) = HPET.get().unwrap().try_read() { 
           global_scheduler().try_lock_check_and_wake(hpet.read_current_us(), percpu);
       }

        let scheduling_disabled = percpu.scheduling_disabled.load(Ordering::Acquire);

        if !scheduling_disabled && ticks_left <= 1 {
            let stored_thread_context = percpu.current_thread_context.try_lock();

            if let Some(stored_thread_context_guard) = stored_thread_context {
                // Enabling interrupts only after current_thread_context was taken
                unsafe {
                    percpu::lapic().end_of_interrupt();
                }
                x86_64::instructions::interrupts::enable();

                unsafe {
                    timer_schedule_next(stored_thread_context_guard, percpu, ctx);
                }
            } else {
                unsafe {
                    percpu::lapic().end_of_interrupt();
                }
            }
        } else {
            unsafe {
                percpu::lapic().end_of_interrupt();
            }
        }
    }, code_segment);

    unsafe {
        iret_with_context(ctx)
    }
}
