use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{gdt, print, println};

pub const SYSCALL_API_CALL: usize = 0x80;
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault
         .set_handler_fn(double_fault_handler)
         .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        
        idt[SYSCALL_API_CALL].set_handler_fn(syscall_api_call_handler);

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(pic_timer_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init_pics() {
    unsafe {
        let mut pics = PICS.lock();
        pics.write_masks(0b00000000, 0b00000000);
        pics.initialize();
    };
}

extern "x86-interrupt" fn pic_timer_handler(stack_frame: InterruptStackFrame) {
    print!(".");
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
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