use crate::hpet::HPET;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::percpu;
use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::process::thread::ThreadStatus;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::thread_context::{ThreadContext, ThreadContextStatus};
use crate::syscalls::helpers::{int_schedule, save_syscall_context, schedule_on_next_tick};
use crate::syscalls::syscall_args::SyscallArgs;
use alloc::boxed::Box;
use core::arch::asm;
use core::ops::DerefMut;
use log::info;
use spin::MutexGuard;
use x86_64::VirtAddr;

pub fn syscall_sleep_ms(
    args: SyscallArgs,
    mut current_thread_context: MutexGuard<Option<Box<ThreadContext>>>,
) -> (u64, MutexGuard<Option<Box<ThreadContext>>>) {
    let target_us = {
        let hpet = HPET.get().unwrap().read();
        let mut processes = PROCESSES_MANAGER.get().unwrap().write();

        let current_us = hpet.read_current_us();
        let target_us = current_us.wrapping_add(args.arg1 * 1000);

        let thread_context = current_thread_context.as_mut().unwrap();

        thread_context.status = ThreadContextStatus::Sleeping {
            wake_at: target_us,
        };

        processes
            .chane_thread_status(
                thread_context.process_id,
                thread_context.thread_id,
                ThreadStatus::Sleeping,
            )
            .expect("Trying to change status of thread that was deleted from processes manager");

        x86_64::instructions::interrupts::disable();
        drop(current_thread_context);
        target_us
    };

    int_schedule();

    let percpu = unsafe { percpu::current() };
    let mut current_thread_context = percpu.current_thread_context.lock();
    x86_64::instructions::interrupts::enable();
    let hpet = HPET.get().unwrap().read();
    let mut processes = PROCESSES_MANAGER.get().unwrap().write();

    let thread_context = current_thread_context.as_mut().unwrap();
    thread_context.status = ThreadContextStatus::Running;

    processes
        .chane_thread_status(
            thread_context.process_id,
            thread_context.thread_id,
            ThreadStatus::Running,
        )
        .expect("Trying to change status of thread that was deleted from processes manager");

    let current_us = hpet.read_current_us();
    let undersleeped_ms = if current_us < target_us {
        target_us.wrapping_sub(current_us) / 1000
    } else {
        let oversleeped = current_us.wrapping_sub(target_us);
        info!("{} oversleeped {}ms", thread_context, oversleeped / 1000);
        0
    };

    // global_scheduler.put_thread_context_to_sleep_queue(thread_context, target_us);

    (undersleeped_ms, current_thread_context)
}
