#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use std_lib::{syscall};

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "sysv64" fn _start() -> ! {
    loop {
        unsafe {
            syscall!(256);

            for i in 0..0x200000 {
                unsafe {
                    asm!("pause");
                }
            }
        }
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
