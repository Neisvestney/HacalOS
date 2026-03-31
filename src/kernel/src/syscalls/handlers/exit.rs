use alloc::boxed::Box;
use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::syscall_args::SyscallArgs;
use log::info;
use x86_64::VirtAddr;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::syscalls::helpers::schedule_on_next_tick;

pub fn syscall_exit(
    args: SyscallArgs,
    _syscall_context: &SyscallContext,
    _user_rsp: VirtAddr,
    percpu: &PerCpu,
    current_thread_context: &mut Option<Box<ThreadContext>>,
) -> u64 {
    let current_thread_context = current_thread_context.as_mut().unwrap();
    let mut processes = PROCESSES_MANAGER.get().unwrap().write();
    let global_scheduler = global_scheduler();
    let result = processes.kill_process(
        args.arg1 as i32,
        current_thread_context.process_id,
        global_scheduler,
    );
    schedule_on_next_tick(percpu);
    info!(
        "syscall exit: code: {}, thread context: {}",
        args.arg1, current_thread_context
    );

    if result.is_err() { -1_i64 as u64 } else { 0 }
}
