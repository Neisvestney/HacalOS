use crate::percpu::PerCpu;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::handlers::exit::syscall_exit;
use crate::syscalls::syscall_args::SyscallArgs;

pub type SyscallHandler = fn(args: SyscallArgs, percpu: &PerCpu, current_thread_context: &ThreadContext) -> u64;

pub static SYSCALLS_TABLE: [SyscallHandler; 1] = [syscall_exit];