use crate::percpu::PerCpu;
use core::arch::{asm, naked_asm};
use log::info;

#[unsafe(naked)]
pub extern "C" fn syscall_entry() -> ! {
    unsafe {
        naked_asm!(
        "swapgs",
        "mov gs:[{ursp}], rsp",

        "mov rsp, gs:[{kstack}]",

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

        "mov rsp, gs:[{ursp}]",

        "swapgs",
        "sysretq",

        handler = sym syscall_handler,
        kstack = const PerCpu::KERNEL_STACK_TOP_STRUCT_OFFSET,
        ursp = const PerCpu::USER_RSP_STRUCT_OFFSET,
        options()
        )
    }
}

#[repr(C)]
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
    let syscall_number = ctx.rax;

    info!("syscall_number: {}", syscall_number);
}
