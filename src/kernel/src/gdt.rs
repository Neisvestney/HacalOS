use alloc::boxed::Box;
use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::Segment;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{CS, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use crate::memory::stack::allocate_stack;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 1;
pub const PAGE_FAULT_IST_INDEX: u16 = 2;

fn build_tss() -> &'static mut TaskStateSegment {
    let tss = Box::leak(Box::new(TaskStateSegment::new()));

    let (double_fault_stack, _) = allocate_stack().expect("Cannot allocate stack for DOUBLE_FAULT_IST_INDEX");
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = double_fault_stack;

    let (page_fault_stack, _) = allocate_stack().expect("Cannot allocate stack for PAGE_FAULT_IST_INDEX");
    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = page_fault_stack;
    tss
}

fn build_gdt(tss: *const TaskStateSegment) -> (&'static mut GlobalDescriptorTable, Selectors) {
    let gdt = Box::leak(Box::new(GlobalDescriptorTable::new()));
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = unsafe {gdt.append(Descriptor::tss_segment_unchecked(tss))};
    (
        gdt,
        Selectors {
            code_selector,
            data_selector,
            tss_selector,
        },
    )
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init_gdt() -> (&'static mut GlobalDescriptorTable, &'static mut TaskStateSegment) {
    let tss = build_tss();
    let (gdt, selectors) = build_gdt(tss);

    unsafe {
        gdt.load_unsafe();
    }

    unsafe {
        CS::set_reg(selectors.code_selector);
        SS::set_reg(selectors.data_selector);
        load_tss(selectors.tss_selector);
    }

    (gdt, tss)
}
