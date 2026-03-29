use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::SyscallArgs;
use core::sync::atomic::Ordering;
use log::{info};

pub fn syscall_exit(
    args: SyscallArgs,
    percpu: &PerCpu,
    current_thread_context: &ThreadContext,
) -> u64 {
    let mut processes = PROCESSES_MANAGER.get().unwrap().write();
    let global_scheduler = global_scheduler();
    let result = processes.kill_process(
        args.arg1 as i32,
        current_thread_context.process_id,
        global_scheduler,
    );
    percpu.current_thread_ticks_left.store(1, Ordering::Release); // So scheduling 100% happens at end of syscall handling process and we never reenter exited process
    info!("syscall exit: code: {}, thread context: {}", args.arg1, current_thread_context);

    if result.is_err() { -1_i64 as u64 } else { 0 }
}
