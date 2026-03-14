const MSR_GS_BASE: u32 = 0xC0000101;
const MSR_KERNEL_GS_BASE: u32 = 0xC0000102;

pub unsafe fn set(ptr: *mut super::PerCpu) {
    let addr = ptr as u64;
    unsafe {
        core::arch::asm!(
        "wrmsr",
        in("ecx") MSR_GS_BASE,
        in("eax") addr as u32,
        in("edx") (addr >> 32) as u32,
        );
    }
}

/// Прочитать GS_BASE → указатель на PerCpu текущего ядра
pub unsafe fn get() -> *mut super::PerCpu {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
        "rdmsr",
        in("ecx")  MSR_GS_BASE,
        out("eax") lo,
        out("edx") hi,
        );
    }
    ((hi as u64) << 32 | lo as u64) as *mut super::PerCpu
}
