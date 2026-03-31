use alloc::boxed::Box;
use x86_64::VirtAddr;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::percpu::PerCpu;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::handlers::exit::syscall_exit;
use crate::syscalls::handlers::sleep_ms::syscall_sleep_ms;
use crate::syscalls::syscall_args::SyscallArgs;

pub type SyscallHandler = fn(args: SyscallArgs, syscall_context: &SyscallContext, user_rsp: VirtAddr, percpu: &PerCpu, current_thread_context: &mut Option<Box<ThreadContext>>) -> u64;

pub static SYSCALLS_TABLE: [SyscallHandler; 2] = [syscall_exit, syscall_sleep_ms];