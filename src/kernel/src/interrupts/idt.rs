use crate::interrupts::handlers::lapic_timer_handler::lapic_timer_entry;
use crate::interrupts::ioapic::{IO_APIC_BASE_OFFSET, KEYBOARD_ISA_IRQ};
use crate::memory::stack::STACK_GUARD_PAGES;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use crate::{CONSOLE, gdt, percpu, print, println};
use lazy_static::lazy_static;
use log::{error, info, warn};
use x86_64::registers::control::Cr2;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::{
    HandlerFuncType, InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
    SelectorErrorCode,
};
use x86_64::structures::paging::Page;
use x86_64::{PrivilegeLevel, VirtAddr};

pub const SYSCALL_API_CALL: u8 = 0x80;

pub const LAPIC_TIMER_VECTOR: u8 = 0x20;
pub const LAPIC_KEYBOARD_VECTOR: u8 = IO_APIC_BASE_OFFSET + KEYBOARD_ISA_IRQ;
pub const LAPIC_ERROR_VECTOR: u8 = 0xFE;
pub const LAPIC_SPURIOUS_VECTOR: u8 = 0xFF;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);

        idt[SYSCALL_API_CALL]
            .set_handler_fn(syscall_api_call_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);

        idt[LAPIC_TIMER_VECTOR].set_handler_addr(VirtAddr::new(lapic_timer_entry as u64));
        idt[LAPIC_KEYBOARD_VECTOR].set_handler_fn(lapic_keyboard_handler);
        idt[LAPIC_ERROR_VECTOR].set_handler_fn(lapic_error_handler);
        idt[LAPIC_SPURIOUS_VECTOR].set_handler_fn(lapic_spurious_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn lapic_keyboard_handler(_stack_frame: InterruptStackFrame) {
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

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    page_fault_error_code: PageFaultErrorCode,
) {
    let virt_address = Cr2::read();

    error!(
        "EXCEPTION: PAGEFAULT\n{:#?}\n{:#?}\nVirtual address: {:#x?}",
        stack_frame, page_fault_error_code, virt_address
    );

    let stack_guard_pages = STACK_GUARD_PAGES.get().unwrap().read();
    let virt_address = virt_address.unwrap();
    if let Some(guard_page) = stack_guard_pages
        .iter()
        .find(|p| **p == Page::containing_address(virt_address))
    {
        error!("Hit stack guard page: {:#x?}", guard_page);
    }

    panic!("EXCEPTION: PAGE FAULT");
}

extern "x86-interrupt" fn syscall_api_call_handler(stack_frame: InterruptStackFrame) {
    info!("SYSCALL: \n{:#?}", stack_frame);
    let a = 1;
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}\n{:x}",
        stack_frame, error_code
    );
}

extern "x86-interrupt" fn lapic_spurious_handler(_frame: InterruptStackFrame) {
    warn!("Spurious interrupt");
}

extern "x86-interrupt" fn lapic_error_handler(_frame: InterruptStackFrame) {
    let lapic_base = unsafe { x2apic::lapic::xapic_base() };
    let esr = unsafe {
        let ptr = (lapic_base + 0x280) as *mut u32;
        ptr.write_volatile(0); // сброс: сначала пишем
        ptr.read_volatile() // потом читаем
    };
    error!("LAPIC ERROR: ESR = {:#010b}", esr);
    unsafe { percpu::lapic().end_of_interrupt() }
}
