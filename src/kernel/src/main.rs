#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader_structs::BootInfo;
use x86_64::instructions::hlt;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: &BootInfo) -> usize {
    test();

    loop { hlt(); }
}

fn test() {

}


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}