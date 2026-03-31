use alloc::boxed::Box;
use spin::MutexGuard;
use x86_64::VirtAddr;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::percpu::PerCpu;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::handlers::exit::syscall_exit;
use crate::syscalls::handlers::sleep_ms::syscall_sleep_ms;
use crate::syscalls::syscall_args::SyscallArgs;

pub type SyscallHandler = fn(args: SyscallArgs, current_thread_context: MutexGuard<Option<Box<ThreadContext>>>) -> (u64, MutexGuard<Option<Box<ThreadContext>>>);

pub static SYSCALLS_TABLE: [SyscallHandler; 2] = [syscall_exit, syscall_sleep_ms];