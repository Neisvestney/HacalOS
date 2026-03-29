use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::percpu::PerCpu;
use crate::scheduler::thread_context::ThreadContext;
use crate::syscalls::exit::syscall_exit;

mod exit;

pub type SyscallHandler = fn(args: SyscallArgs, percpu: &PerCpu, current_thread_context: &ThreadContext) -> u64;

pub static SYSCALLS_TABLE: [SyscallHandler; 1] = [syscall_exit];

pub struct SyscallArgs {
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
    pub arg6: u64,
}

impl From<&SyscallContext> for SyscallArgs {
    fn from(ctx: &SyscallContext) -> SyscallArgs {
        SyscallArgs {
            arg1: ctx.rdi,
            arg2: ctx.rsi,
            arg3: ctx.rdx,
            arg4: ctx.r10,
            arg5: ctx.r8,
            arg6: ctx.r9,
        }
    }
}