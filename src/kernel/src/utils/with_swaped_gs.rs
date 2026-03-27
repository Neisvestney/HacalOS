use crate::gdt::segment_selectors;
use core::arch::asm;
use x86_64::structures::gdt::SegmentSelector;

pub fn with_swaped_gs<F, R>(f: F, code_segment: SegmentSelector) -> R
where
    F: FnOnce() -> R,
{
    let gs_swaped = if code_segment != segment_selectors().code_selector {
        // Interrupted user program
        unsafe {
            asm!("swapgs");
        }
        true
    } else {
        // Interrupted in kernel
        false
    };

    let r = f();

    if gs_swaped {
        unsafe {
            asm!("swapgs");
        }
    }

    r
}
