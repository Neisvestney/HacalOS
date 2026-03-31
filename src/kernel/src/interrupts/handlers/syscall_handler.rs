use crate::percpu;
use crate::percpu::PerCpu;
use crate::scheduler::schedule_on_interrupt::syscall_schedule_check;
use core::arch::{naked_asm};
use core::ops::{Deref, DerefMut};
use log::{warn};
use crate::syscalls::syscall_args::SyscallArgs;
use crate::syscalls::syscalls_table::SYSCALLS_TABLE;

#[unsafe(naked)]
pub extern "C" fn syscall_entry() -> ! {
    unsafe {
        naked_asm!(
        "swapgs",
        "mov gs:[{ursp}], rsp",

        "mov rsp, gs:[{kstack}]",

        "push rax", // Align stack tp 16 bytes
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov rdi, rsp", // &SyscallContext
        "call {handler}",

        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop rax",

        "mov rsp, gs:[{ursp}]",

        "swapgs",
        "sysretq",

        handler = sym syscall_handler,
        kstack = const PerCpu::SYSCALL_STACK_STRUCT_OFFSET,
        ursp = const PerCpu::USER_RSP_STRUCT_OFFSET,
        options()
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SyscallContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11_rflags: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx_rip: u64,
    pub rbx: u64,
    pub rax: u64,
}


pub extern "C" fn syscall_handler(ctx: &mut SyscallContext) {
    let percpu = unsafe { percpu::current_mut() };
    let mut stored_thread_context = percpu.current_thread_context.lock();
    let user_rsp = percpu.user_rsp;
    x86_64::instructions::interrupts::enable();

    if stored_thread_context.is_some() {
        let syscall_handler = SYSCALLS_TABLE.get(ctx.rax as usize);
        let ret = if let Some(handler) = syscall_handler {
            handler(SyscallArgs::from(ctx.deref()), stored_thread_context)
        } else {
            warn!("{} called unknown syscall #{}", stored_thread_context.as_ref().unwrap(), ctx.rax);
            (0, stored_thread_context)
        };

        ctx.rax = ret.0;
        stored_thread_context = ret.1;
    } else {
        panic!("Syscall called with no thread context\n{:?}\n{:?}", ctx, percpu);
    }

    // for i in 0..0x1000000 {
    //     hint::spin_loop();
    // }

    let percpu_mut = unsafe { percpu::current_mut() };
    syscall_schedule_check(stored_thread_context, percpu_mut, ctx, user_rsp);
    x86_64::instructions::interrupts::disable();
    percpu_mut.user_rsp = user_rsp; // In case of syscall was preempted
}
