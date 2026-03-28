use crate::PAGE_TABLE_MAPPER;
use crate::memory::virtual_memory_allocator::{
    KERNEL_VIRTUAL_MEMORY_ALLOCATOR, VirtualMemoryAllocatorError,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::asm;
use core::ops::DerefMut;
use spin::{Once, RwLock};
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, Size4KiB};

const KERNEL_STACK_PAGES_COUNT: u64 = 40;

pub static KERNEL_STACK_GUARD_PAGES: Once<RwLock<Vec<Page<Size4KiB>>>> = Once::new();

pub fn stack_top_from_slice(stack_slice_pointer: *const [u8]) -> *const u8 {
    let start_bottom = stack_slice_pointer as *const u8;
    let stack_top = unsafe { start_bottom.add(stack_slice_pointer.len()) };
    // Align to 16 bytes
    let stack_top = ((stack_top as usize & !0xF) - 0x100) as *const u8;

    stack_top
}

pub fn allocate_stack() -> Result<(VirtAddr, Page<Size4KiB>), VirtualMemoryAllocatorError> {
    let mut page_table_mapper = PAGE_TABLE_MAPPER.get().unwrap().lock();
    let (allocated, guard_page, _) = KERNEL_VIRTUAL_MEMORY_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .alloc_pages_with_protection_page(
            KERNEL_STACK_PAGES_COUNT + 1,
            page_table_mapper.deref_mut(),
        )?;

    let stack_top = stack_top_from_slice(allocated);

    let mut guard_pages = KERNEL_STACK_GUARD_PAGES
        .call_once(|| RwLock::new(Vec::new()))
        .write();
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
