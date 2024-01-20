#![no_std]
#![no_main]

mod render;

use core::panic::PanicInfo;
use bootloader_structs::BootInfo;
use x86_64::instructions::hlt;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: &BootInfo) -> usize {
    test();

    unsafe {
        boot_info.gop.frame_buffer.cast::<u32>().write_volatile(0xFFFFFF);
    }

    loop { hlt(); }
}

fn test() {

}


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}