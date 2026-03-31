use alloc::boxed::Box;
use core::arch::asm;
use core::sync::atomic::Ordering;
use log::info;
use spin::MutexGuard;
use volatile::VolatilePtr;
use x86_64::instructions::hlt;
use x86_64::VirtAddr;
use crate::gdt::segment_selectors;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::interrupts::iret_wit_context::iret_with_context;
use crate::memory::paging::write_cr3;
use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::scheduler::global_scheduler::{global_scheduler, GlobalScheduler};
use crate::scheduler::SCHEDULE_TICKS;
use crate::scheduler::signals::ThreadSignal;
use crate::scheduler::thread_context::{ThreadContext, ThreadContextStatus};
use crate::syscalls::helpers::save_syscall_context;

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
    percpu: &mut PerCpu,
    ctx: &mut CpuRegistriesContext,
) -> ! {
    let global_scheduler = global_scheduler();

    let next_thread_context =
        if let Some(mut previous_thread_context) = stored_thread_context_guard.take() {
            previous_thread_context.cpu_registries_context = *ctx;
            match previous_thread_context.status {
                ThreadContextStatus::Running => {
                    global_scheduler.push_thread_back_and_get_next(previous_thread_context)
                }
                ThreadContextStatus::Sleeping {wake_at} => {
                    global_scheduler.put_thread_context_to_sleep_queue(previous_thread_context, wake_at);
                    global_scheduler.get_next()
                }
            }


        } else {
            global_scheduler.get_next()
        };

    if let Some(next_thread_context) = next_thread_context {
        check_for_signals_and_switch_to_thread(percpu, next_thread_context, stored_thread_context_guard, global_scheduler)
    } else {
        *stored_thread_context_guard = None;
        drop(stored_thread_context_guard);

        no_tasks_hlt_loop(percpu);
    }
}


pub fn syscall_schedule_check(mut stored_thread_context_guard: MutexGuard<Option<Box<ThreadContext>>>, percpu: &mut PerCpu, ctx: &mut SyscallContext, user_rsp: VirtAddr) {
    let ticks_left = percpu.current_thread_ticks_left.load(Ordering::Acquire);
    if ticks_left <= 1 {
        let global_scheduler = global_scheduler();

        let next_thread_context =
            if let Some(mut previous_thread_context) = stored_thread_context_guard.take() {
                save_syscall_context(ctx, percpu.user_rsp, &mut previous_thread_context);
                global_scheduler.push_thread_back_and_get_next(previous_thread_context)
            } else {
                global_scheduler.get_next()
            };

        if let Some(next_thread_context) = next_thread_context {
            check_for_signals_and_switch_to_thread(percpu, next_thread_context, stored_thread_context_guard, global_scheduler)
        } else {
            *stored_thread_context_guard = None;
            drop(stored_thread_context_guard);

            no_tasks_hlt_loop(percpu);
        }
    }
}

fn check_for_signals_and_switch_to_thread(percpu: &mut PerCpu, next_thread_context: Box<ThreadContext>, mut stored_thread_context_guard: MutexGuard<Option<Box<ThreadContext>>>, global_scheduler: &GlobalScheduler) -> ! {
    let next_pending_signal = global_scheduler.get_next_pending_signal(next_thread_context.process_id, next_thread_context.thread_id);
    if let Some(next_pending_signal) = next_pending_signal {
        info!("Handling signal {:?} for {}", next_pending_signal, next_thread_context);
        match next_pending_signal {
            ThreadSignal::Terminate => {
                {
                    *stored_thread_context_guard = None;

                    let mut processes_manager = PROCESSES_MANAGER.get().unwrap().write();
                    processes_manager.remove_thread_and_cleanup(next_thread_context.process_id, next_thread_context.thread_id, global_scheduler).unwrap();

                    drop(next_thread_context);
                }
                drop(stored_thread_context_guard);
                no_tasks_hlt_loop(percpu);
            }
        }
    } else {
        switch_to_thread(percpu, next_thread_context, stored_thread_context_guard)
    }
}

fn switch_to_thread(percpu: &mut PerCpu, next_thread_context: Box<ThreadContext>, mut stored_thread_context_guard: MutexGuard<Option<Box<ThreadContext>>>) -> ! {
    percpu
        .current_thread_ticks_left
        .store(SCHEDULE_TICKS, Ordering::Relaxed);

    // Switching to next thread
    unsafe {
        write_cr3(next_thread_context.page_table_phys_frame);
    }

    let cpu_registries_context = next_thread_context.cpu_registries_context;
    percpu.syscall_stack = next_thread_context.syscall_stack;

    *stored_thread_context_guard = Some(next_thread_context);
    x86_64::instructions::interrupts::disable();
    drop(stored_thread_context_guard);
    // info!("ts");

    if cpu_registries_context.stack_frame.code_segment != segment_selectors().code_selector {
       unsafe {
           asm!("swapgs"); // Switching to user mode
       }
    }

    unsafe {
        iret_with_context(&cpu_registries_context)
    }
}

#[inline(always)]
pub fn no_tasks_hlt_loop(percpu: &PerCpu) -> ! {
    unsafe {
        asm!(
        "mov rsp, {kstack}",
        kstack = in(reg) percpu.kernel_stack_top.as_u64(),
        ); // Moving rsp back to top to prevent stack overflow
    }
    loop {
        // No more things to do
        hlt();
    }
}