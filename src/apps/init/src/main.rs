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
            syscall!(42);

            for i in 0..0x100000 {
                unsafe {
                    asm!("pause");
                }
            }

            let prt = 0x0 as *mut u8;
            prt.write_volatile(0x1); // Causes #PF
        }
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
