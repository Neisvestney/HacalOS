use crate::gdt::segment_selectors;
use crate::memory::paging::write_cr3;
use crate::percpu;
use crate::percpu::PerCpu;
use crate::scheduler::SCHEDULE_TICKS;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::scheduler::global_scheduler::global_scheduler;
use core::arch::{asm, naked_asm};
use core::hint;
use core::sync::atomic::Ordering;
use volatile::VolatilePtr;
use x86_64::instructions::hlt;
use crate::interrupts::iret_wit_context::iret_with_context;

#[unsafe(naked)]
pub extern "C" fn lapic_timer_entry() -> ! {
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
        func = sym lapic_timer_handler,
        options()
    )
}

pub unsafe extern "C" fn lapic_timer_handler(ctx: &mut CpuRegistriesContext) {
    let gs_spaped = if ctx.stack_frame.code_segment != segment_selectors().code_selector {
        asm!("swapgs");
        true
    } else {
        false
    };

    let percpu = unsafe { percpu::current() };

    let ticks_left = timer_schedule_tick(percpu);

    if ctx.stack_frame.code_segment == segment_selectors().code_selector {
        // Interrupted in kernel
        if percpu
            .start_scheduling_on_next_tick
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            unsafe {
                timer_schedule_check(ticks_left, percpu, ctx);
                asm!("swapgs");
            }
        } else {
            unsafe { percpu::lapic().end_of_interrupt() }
        }
    } else {
        // Interrupted user program
        unsafe {
            timer_schedule_check(ticks_left, percpu, ctx);
        }
    }

    unsafe {
        if gs_spaped {
            asm!("swapgs");
        }

        iret_with_context(ctx)
    }
}

fn timer_schedule_tick(percpu: &PerCpu) -> u64 {
    let prev_ticks_left = percpu
        .current_thread_ticks_left
        .fetch_sub(1, Ordering::Relaxed);

    if prev_ticks_left <= 1 {
        percpu.current_thread_ticks_left.store(1, Ordering::Release);
        return 1;
    }

    prev_ticks_left
}

unsafe fn timer_schedule_check(ticks_left: u64, percpu: &PerCpu, ctx: &mut CpuRegistriesContext) {
    let global_scheduler = global_scheduler();

    unsafe { percpu::lapic().end_of_interrupt() }
    x86_64::instructions::interrupts::enable();

    if ticks_left <= 1 {
        let stored_thread_context = unsafe { &mut *percpu.current_thread_context.get() };
        let next_thread_context =
            if let Some(mut previous_thread_context) = stored_thread_context.take() {
                previous_thread_context.cpu_registries_context = *ctx;
                global_scheduler.push_thread_back_and_get_next(previous_thread_context)
            } else {
                global_scheduler.get_next()
            };

        if let Some(next_thread_context) = next_thread_context {
            percpu
                .current_thread_ticks_left
                .store(SCHEDULE_TICKS, Ordering::Relaxed);

            let ctx_volatile = unsafe { VolatilePtr::new(ctx.into()) };
            ctx_volatile.write(next_thread_context.cpu_registries_context); // Switching to next thread
            unsafe {
                write_cr3(next_thread_context.page_table_phys_frame);
            }

            *stored_thread_context = Some(next_thread_context);
        } else {
            percpu
                .start_scheduling_on_next_tick
                .store(true, Ordering::Release);

            *stored_thread_context = None;
            loop {
                // No more thinks to do
                hlt();
            }
        }
    }
}
