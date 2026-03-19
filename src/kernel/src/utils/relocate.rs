use crate::VIRTUAL_TO_PHYSICAL_OFFSET;
use core::ptr;
use x86_64::structures::paging::{Page, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

pub fn relocate_addr(virt_addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(virt_addr.as_u64() + VIRTUAL_TO_PHYSICAL_OFFSET.as_u64())
}

pub fn relocate_frame(frame: PhysFrame) -> Page {
    unsafe { Page::from_start_address_unchecked(relocate_addr(frame.start_address())) }
}

pub unsafe fn relocate_raw_pointer<T: ?Sized>(pointer: *const T) -> *const T {
    let addr = pointer
        .addr()
        .wrapping_add(VIRTUAL_TO_PHYSICAL_OFFSET.as_u64() as usize);
    ptr::from_raw_parts(addr as *const (), ptr::metadata(pointer))
}


pub unsafe fn relocate_raw_pointer_mut<T: ?Sized>(pointer: *mut T) -> *mut T {
    // pointer.byte_offset(VIRTUAL_TO_PHYSICAL_OFFSET.as_u64() as isize)

    let addr = pointer
        .addr()
        .wrapping_add(VIRTUAL_TO_PHYSICAL_OFFSET.as_u64() as usize);
    ptr::from_raw_parts_mut(addr as *mut (), ptr::metadata(pointer))
}
