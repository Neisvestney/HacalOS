use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use core::arch::asm;

#[inline(always)]
pub unsafe fn iret_with_context(ctx: &CpuRegistriesContext) -> ! {
    unsafe {
        asm!(
        "mov rsp, {0}",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rsi",
        "pop rdi",
        "pop rbp",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "iretq",
        in(reg) ctx,
        options(noreturn)
        )
    }
}
