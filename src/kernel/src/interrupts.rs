use core::ops::Index;
use lazy_static::lazy_static;
use pc_keyboard::ScancodeSet1;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{gdt, print, println};

pub const SYSCALL_API_CALL: u8 = 0x80;
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
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
        
        idt.index(1);
        
        idt[SYSCALL_API_CALL].set_handler_fn(syscall_api_call_handler);

        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(pic_timer_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(pic_keyboard_handler);
        
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
        pics.write_masks(0b11111000, 0b11111111);
        pics.initialize();
    };
}

extern "x86-interrupt" fn pic_timer_handler(stack_frame: InterruptStackFrame) {
    // print!(".");
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn pic_keyboard_handler(stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
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

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
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