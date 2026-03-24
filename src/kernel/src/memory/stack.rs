use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::memory::virtual_memory_allocator::{
    KERNEL_VIRTUAL_MEMORY_ALLOCATOR, VirtualMemoryAllocatorError,
};
use core::arch::asm;
use spin::{Once, RwLock};
use x86_64::structures::paging::{Page, Size4KiB};
use x86_64::VirtAddr;

const KERNEL_STACK_PAGES_COUNT: u64 = 40;

pub static STACK_GUARD_PAGES: Once<RwLock<Vec<Page<Size4KiB>>>> = Once::new();

pub fn allocate_stack() -> Result<(VirtAddr, Page<Size4KiB>), VirtualMemoryAllocatorError> {
    let (allocated, guard_page) = KERNEL_VIRTUAL_MEMORY_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .alloc_pages_with_protection_page(KERNEL_STACK_PAGES_COUNT + 1)?;

    let start_bottom = allocated as *const u8;
    let stack_top = unsafe { start_bottom.add(allocated.len()) };
    // Align to 16 bytes
    let stack_top = ((stack_top as usize & !0xF) - 0x100) as *const u8;

    let mut guard_pages = STACK_GUARD_PAGES.call_once(|| RwLock::new(Vec::new())).write();
    guard_pages.push(guard_page);

    Ok((VirtAddr::from_ptr(stack_top), guard_page))
}

pub fn switch_stack_and_jump<F>(stack_top: VirtAddr, f: F) -> !
where
    F: FnOnce() -> !,
{
    let ptr = Box::into_raw(Box::new(f));
    let stack_top: *const u8 = stack_top.as_ptr();

    unsafe {
        asm!(
            "mov rsp, {stack}",
            "mov rdi, {arg}",
            "call {trampoline}",
            stack = in(reg) stack_top,
            arg = in(reg) ptr,
            trampoline = sym trampoline::<F>,
            options(noreturn)
        )
    }
}

extern "C" fn trampoline<F: FnOnce() -> !>(ptr: *mut F) -> ! {
    unsafe {
        let f = Box::from_raw(ptr);
        f()
    }
}
