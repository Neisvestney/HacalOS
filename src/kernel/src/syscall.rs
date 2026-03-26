use crate::gdt::segment_selectors;
use crate::interrupts::handlers::syscall_handler::syscall_entry;
use x86_64::VirtAddr;
use x86_64::registers::control::EferFlags;
use x86_64::registers::model_specific::{Efer, LStar, SFMask, Star};

pub fn init_syscalls() {
    unsafe {
        let handler_addr = VirtAddr::new(syscall_entry as *const () as u64);
        LStar::write(handler_addr);

        let mut efer = Efer::read();
        efer |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        Efer::write(efer);

        // disable IF (interrupt flag) on enter
        SFMask::write(x86_64::registers::rflags::RFlags::INTERRUPT_FLAG);

        let segment_selectors = segment_selectors();

        Star::write(
            segment_selectors.user_code_selector,
            segment_selectors.user_data_selector,
            segment_selectors.code_selector,
            segment_selectors.data_selector,
        )
        .unwrap();
    }
}
