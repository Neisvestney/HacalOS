use alloc::boxed::Box;
use crate::percpu::PerCpu;
use core::sync::atomic::Ordering;
use x86_64::VirtAddr;
use crate::interrupts::handlers::syscall_handler::SyscallContext;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::scheduler::thread_context::ThreadContext;

#[inline(always)]
pub fn schedule_on_next_tick(percpu: &PerCpu) {
    percpu.current_thread_ticks_left.store(1, Ordering::Release); // So scheduling 100% happens at end of syscall handling process and we never reenter exited process
}


#[inline(always)]
pub fn save_syscall_context(syscall_context: &SyscallContext, user_rsp: VirtAddr, thread_context: &mut ThreadContext) {
    thread_context.cpu_registries_context = CpuRegistriesContext::from_syscall_context(syscall_context, user_rsp)
}