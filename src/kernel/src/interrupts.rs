use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{gdt, println};

pub const SYSCALL_API_CALL: usize = 0x80;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault
         .set_handler_fn(double_fault_handler)
         .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        
        idt[SYSCALL_API_CALL].set_handler_fn(syscall_api_call_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, page_fault_error_code: PageFaultErrorCode)
{
    println!("EXCEPTION: PAGEFAULT\n{:#?}\n{:#?}", stack_frame, page_fault_error_code);
}

extern "x86-interrupt" fn syscall_api_call_handler(stack_frame: InterruptStackFrame)
{
    println!("SYSCALL: API CALL\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}