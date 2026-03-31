use crate::syscall;
use core::arch::asm;

pub fn syscall_exit(exit_code: i32) -> ! {
    let result = syscall!(0, exit_code as u64);

    if result as i64 == -1 {
        panic!("syscall exit failed: {}", result);
    }

    loop {
        unsafe { asm!("pause") }
    }
}

pub fn syscall_sleep_ms(ms: u64) {
    syscall!(1, ms);
}

