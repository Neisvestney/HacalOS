use core::arch::naked_asm;
use crate::percpu;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::scheduler::schedule_on_interrupt::timer_schedule_next;

#[unsafe(naked)]
pub extern "C" fn int_schedule_entry() -> ! {
    naked_asm!(
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rbp",
        "push rdi",
        "push rsi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // rdi = &CPURegistriesContext
        "mov rdi, rsp",
        "call {func}",
        "hlt",
        func = sym int_schedule_handler,
        options()
    )
}

pub unsafe extern "C" fn int_schedule_handler(ctx: &mut CpuRegistriesContext) {
    let percpu = unsafe { percpu::current() };


    let stored_thread_context_guard_guard = percpu.current_thread_context.lock();

    x86_64::instructions::interrupts::enable();

    unsafe {
        let percpu_mut = percpu::current_mut();
        timer_schedule_next(stored_thread_context_guard_guard, percpu_mut, ctx);
    }
}
