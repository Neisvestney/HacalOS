use alloc::boxed::Box;
use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::syscall_args::SyscallArgs;
use log::info;
use spin::MutexGuard;
use x86_64::VirtAddr;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::percpu;
use crate::syscalls::helpers::schedule_on_next_tick;

pub fn syscall_exit(
    args: SyscallArgs,
    mut current_thread_context: MutexGuard<Option<Box<ThreadContext>>>,
) -> (u64, MutexGuard<Option<Box<ThreadContext>>>) {
    let thread_context = current_thread_context.as_mut().unwrap();
    let mut processes = PROCESSES_MANAGER.get().unwrap().write();
    let global_scheduler = global_scheduler();
    let result = processes.kill_process(
        args.arg1 as i32,
        thread_context.process_id,
        global_scheduler,
    );
    let percpu = unsafe {percpu::current()};
    schedule_on_next_tick(percpu);
    info!(
        "syscall exit: code: {}, thread context: {}",
        args.arg1, thread_context
    );

    if result.is_err() { (-1_i64 as u64, current_thread_context) } else { (0, current_thread_context) }
}
