use crate::hpet::HPET;
use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::process::thread::ThreadStatus;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::syscall_args::SyscallArgs;
use alloc::boxed::Box;
use core::ops::DerefMut;
use x86_64::VirtAddr;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::syscalls::helpers::{save_syscall_context, schedule_on_next_tick};

pub fn syscall_sleep_ms(
    args: SyscallArgs,
    syscall_context: &SyscallContext,
    user_rsp: VirtAddr,
    percpu: &PerCpu,
    current_thread_context: &mut Option<Box<ThreadContext>>,
) -> u64 {
    let global_scheduler = global_scheduler();
    let hpet = HPET.get().unwrap().read();
    let mut processes = PROCESSES_MANAGER.get().unwrap().write();

    let current_us = hpet.read_current_us();
    let target_us = current_us.wrapping_add(args.arg1 * 1000);

    let mut thread_context = current_thread_context.take().unwrap();
    
    processes
        .chane_thread_status(
            thread_context.process_id,
            thread_context.thread_id,
            ThreadStatus::Sleeping,
        )
        .expect("Trying to change status of thread that was deleted from processes manager");

    save_syscall_context(syscall_context, user_rsp, &mut thread_context);
    global_scheduler.put_thread_context_to_sleep_queue(thread_context, target_us);

    schedule_on_next_tick(percpu);

    0
}
