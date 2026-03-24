#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "sysv64" fn _start() -> usize {
    unsafe {
        asm!("int $0x80", options(nomem, nostack));
    }

    0
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
