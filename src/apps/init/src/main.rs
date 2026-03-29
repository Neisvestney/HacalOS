#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use std_lib::{syscall};
use std_lib::syscalls::syscall_exit;

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "sysv64" fn _start() -> ! {
    for i in 0..5 {
        unsafe {
            syscall!(42);

            for i in 0..0x100000 {
                unsafe {
                    asm!("pause");
                }
            }
        }
    }

   unsafe {
       let prt = 0x0 as *mut u8;
       prt.write_volatile(0x1); // To test #PF
   }

    syscall_exit(0)
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
