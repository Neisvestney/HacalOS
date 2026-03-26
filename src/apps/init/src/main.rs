#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "sysv64" fn _start() -> ! {
    loop {
        unsafe {
            // asm!("int $0x80", options(nomem, nostack));

            syscall!(42);
        }
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


#[macro_export]
macro_rules! syscall {
    ($num:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};

    ($num:expr, $a1:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $a1,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};

    ($num:expr, $a1:expr, $a2:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $a1,
                in("rsi") $a2,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};

    ($num:expr, $a1:expr, $a2:expr, $a3:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $a1,
                in("rsi") $a2,
                in("rdx") $a3,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};

    ($num:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $a1,
                in("rsi") $a2,
                in("rdx") $a3,
                in("r10") $a4,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};

    ($num:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $a1,
                in("rsi") $a2,
                in("rdx") $a3,
                in("r10") $a4,
                in("r8")  $a5,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};

    ($num:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {{
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $a1,
                in("rsi") $a2,
                in("rdx") $a3,
                in("r10") $a4,
                in("r8")  $a5,
                in("r9")  $a6,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        ret
    }};
}