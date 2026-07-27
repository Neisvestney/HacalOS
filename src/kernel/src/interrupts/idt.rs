use alloc::boxed::Box;
use core::ops::Deref;
use core::sync::atomic::Ordering;
use crate::interrupts::handlers::lapic_timer_handler::lapic_timer_entry;
use crate::interrupts::ioapic::{IO_APIC_BASE_OFFSET, KEYBOARD_ISA_IRQ};
use crate::memory::stack::KERNEL_STACK_GUARD_PAGES;
use crate::utils::with_swaped_gs::with_swaped_gs;
use crate::{gdt, percpu, print, CONSOLE};
use lazy_static::lazy_static;
use log::{error, info, warn};
use spin::MutexGuard;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode, SelectorErrorCode,
};
use x86_64::structures::paging::Page;
use x86_64::{PrivilegeLevel, VirtAddr};
use crate::interrupts::handlers::int_schedule_handler::int_schedule_entry;
use crate::percpu::PerCpu;
use crate::process::processes_manager::PROCESSES_MANAGER;
use crate::scheduler::global_scheduler::global_scheduler;
use crate::scheduler::schedule_on_interrupt::no_tasks_hlt_loop;
use crate::scheduler::thread_context::ThreadContext;

pub const LAPIC_TIMER_VECTOR: u8 = 0x20;
pub const LAPIC_KEYBOARD_VECTOR: u8 = IO_APIC_BASE_OFFSET + KEYBOARD_ISA_IRQ;
pub const LAPIC_ERROR_VECTOR: u8 = 0xFE;
pub const LAPIC_SPURIOUS_VECTOR: u8 = 0xFF;

pub const INT_SCHEDULE_VECTOR: u8 = 0x70;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);

        idt[LAPIC_TIMER_VECTOR]
            .set_handler_addr(VirtAddr::new(lapic_timer_entry as *const () as u64));
        idt[LAPIC_KEYBOARD_VECTOR].set_handler_fn(lapic_keyboard_handler);
        idt[LAPIC_ERROR_VECTOR].set_handler_fn(lapic_error_handler);
        idt[LAPIC_SPURIOUS_VECTOR].set_handler_fn(lapic_spurious_handler);

        idt[INT_SCHEDULE_VECTOR]
            .set_handler_addr(VirtAddr::new(int_schedule_entry as *const () as u64));

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn lapic_keyboard_handler(stack_frame: InterruptStackFrame) {
    with_swaped_gs(
        || {
            use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
            use spin::Mutex;
            use x86_64::instructions::port::Port;

            lazy_static! {
                static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = {
                    let s = ScancodeSet1::new();
                    Mutex::new(Keyboard::new(s, layouts::Us104Key, HandleControl::Ignore))
                };
            }

            let mut keyboard = KEYBOARD.lock();
            let mut port = Port::new(0x60);

            let scancode: u8 = unsafe { port.read() };
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => print!("{}", character),
                        // DecodedKey::RawKey(key) => print!("{:?}", key),
                        _ => {}
                    }
                }
            }

            unsafe { percpu::lapic().end_of_interrupt() }
        },
        stack_frame.code_segment,
    )
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    warn!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let error_code = SelectorErrorCode::new(error_code);
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code: {:?}",
        stack_frame, error_code
    );
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}\n", stack_frame,);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    page_fault_error_code: PageFaultErrorCode,
) {
    let virt_address = Cr2::read();

    with_swaped_gs(|| {
        let percpu = unsafe { percpu::current() };
        let maybe_thread_context = current_thread_context(percpu);

        if let Some(thread_context_guard) = maybe_thread_context && let Some(thread_context) = thread_context_guard.deref() {
            {
                let mut processes = PROCESSES_MANAGER.get().unwrap().write();
                error!("Page fault occurred in thread in non critical section: {:?}\n{:#?}\n{:#?}\nVirtual address: {:#x?}", thread_context, stack_frame, page_fault_error_code, virt_address);
                let global_scheduler = global_scheduler();
                processes.kill_process(-1, thread_context.process_id, global_scheduler).unwrap();
                drop(thread_context_guard);
            }
            x86_64::instructions::interrupts::enable();
            no_tasks_hlt_loop(percpu) // TODO Run next thread here instead of waiting for timer
        } else {
            let stack_guard_pages = KERNEL_STACK_GUARD_PAGES.get().unwrap().read();
            let virt_address = virt_address.unwrap();
            unsafe { CONSOLE.get().unwrap().force_unlock() };
            error!("Page fault occurred in critical section\n{:#?}\n{:#?}\nVirtual address: {:#x?}", stack_frame, page_fault_error_code, virt_address);
            if let Some(guard_page) = stack_guard_pages
                .iter()
                .find(|p| **p == Page::containing_address(virt_address))
            {
                error!("Hit stack guard page: {:#x?}", guard_page);
            }

            panic!("KERNEL EXCEPTION: PAGE FAULT")
        }
    }, stack_frame.code_segment);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "KERNEL EXCEPTION: DOUBLE FAULT\n{:#?}\n{:x}",
        stack_frame, error_code
    );
}

extern "x86-interrupt" fn lapic_spurious_handler(_stack_frame: InterruptStackFrame) {
    warn!("Spurious interrupt");
}

extern "x86-interrupt" fn lapic_error_handler(stack_frame: InterruptStackFrame) {
    with_swaped_gs(
        || {
            let lapic_base = unsafe { x2apic::lapic::xapic_base() };
            let esr = unsafe {
                let ptr = (lapic_base + 0x280) as *mut u32;
                ptr.write_volatile(0); // сброс: сначала пишем
                ptr.read_volatile() // потом читаем
            };
            error!("LAPIC ERROR: ESR = {:#010b}", esr);
            unsafe { percpu::lapic().end_of_interrupt() }
        },
        stack_frame.code_segment,
    );
}

fn current_thread_context(percpu: &PerCpu) -> Option<MutexGuard<Option<Box<ThreadContext>>>> {
    use core::sync::atomic::{Ordering};

    let scheduling_disabled = percpu.scheduling_disabled.load(Ordering::Acquire);

    if !scheduling_disabled {
        percpu.current_thread_context.try_lock()
    } else {
        None
    }
}