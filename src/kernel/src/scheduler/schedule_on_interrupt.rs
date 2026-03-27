use alloc::boxed::Box;
use core::arch::asm;
use core::sync::atomic::Ordering;
use log::info;
use spin::MutexGuard;
use volatile::VolatilePtr;
use x86_64::instructions::hlt;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::interrupts::iret_wit_context::iret_with_context;
use crate::memory::paging::write_cr3;
use crate::percpu::PerCpu;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::SCHEDULE_TICKS;
use crate::scheduler::thread_context::ThreadContext;

pub fn timer_schedule_tick(percpu: &PerCpu) -> u64 {
    let prev_ticks_left = percpu
        .current_thread_ticks_left
        .fetch_sub(1, Ordering::Relaxed);

    if prev_ticks_left <= 1 {
        percpu.current_thread_ticks_left.store(1, Ordering::Release);
        return 1;
    }

    prev_ticks_left
}

pub unsafe fn timer_schedule_next(
    mut stored_thread_context_guard: MutexGuard<Option<Box<ThreadContext>>>,
    percpu: &PerCpu,
    ctx: &mut CpuRegistriesContext,
) -> ! {
    let global_scheduler = global_scheduler();

    let next_thread_context =
        if let Some(mut previous_thread_context) = stored_thread_context_guard.take() {
            previous_thread_context.cpu_registries_context = *ctx;
            global_scheduler.push_thread_back_and_get_next(previous_thread_context)
        } else {
            global_scheduler.get_next()
        };

    if let Some(next_thread_context) = next_thread_context {
        percpu
            .current_thread_ticks_left
            .store(SCHEDULE_TICKS, Ordering::Relaxed);

        // Switching to next thread
        unsafe {
            write_cr3(next_thread_context.page_table_phys_frame);
        }

        let cpu_registries_context = next_thread_context.cpu_registries_context;

        *stored_thread_context_guard = Some(next_thread_context);
        x86_64::instructions::interrupts::disable();
        drop(stored_thread_context_guard);
        // info!("ts");
        unsafe {
            asm!("swapgs");
            iret_with_context(&cpu_registries_context)
        }
    } else {
        *stored_thread_context_guard = None;
        drop(stored_thread_context_guard);
        loop {
            // No more things to do
            hlt();
        }
    }
}


pub fn syscall_schedule_check(mut stored_thread_context_guard: MutexGuard<Option<Box<ThreadContext>>>, percpu: &PerCpu, ctx: &mut SyscallContext) {
    let ticks_left = percpu.current_thread_ticks_left.load(Ordering::Acquire);
    if ticks_left <= 1 {
        let global_scheduler = global_scheduler();

        let next_thread_context =
            if let Some(mut previous_thread_context) = stored_thread_context_guard.take() {
                previous_thread_context.cpu_registries_context = CpuRegistriesContext::from_syscall_context(ctx, percpu.user_rsp);
                global_scheduler.push_thread_back_and_get_next(previous_thread_context)
            } else {
                global_scheduler.get_next()
            };

        if let Some(next_thread_context) = next_thread_context {
            percpu
                .current_thread_ticks_left
                .store(SCHEDULE_TICKS, Ordering::Relaxed);

            // Switching to next thread
            unsafe {
                write_cr3(next_thread_context.page_table_phys_frame);
            }

            let cpu_registries_context = next_thread_context.cpu_registries_context;

            *stored_thread_context_guard = Some(next_thread_context);
            x86_64::instructions::interrupts::disable();
            drop(stored_thread_context_guard);
            // info!("sts");
            unsafe {
                asm!("swapgs");
                iret_with_context(&cpu_registries_context);
            }
        } else {
            *stored_thread_context_guard = None;

            x86_64::instructions::interrupts::disable();
            drop(stored_thread_context_guard);
            percpu.scheduling_disabled.store(false, Ordering::Release);
            x86_64::instructions::interrupts::enable();

            loop {
                // No more things to do
                hlt();
            }
        }
    }
}