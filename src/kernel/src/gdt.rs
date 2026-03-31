use crate::memory::stack::allocate_kernel_stack;
use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use x86_64::instructions::segmentation::Segment;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{CS, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 1;
pub const PAGE_FAULT_IST_INDEX: u16 = 2;

static SEGMENTS: SelectorsWrapper = SelectorsWrapper(UnsafeCell::new(MaybeUninit::uninit()));

fn build_tss(kernel_stack_top: VirtAddr) -> &'static mut TaskStateSegment {
    let tss = Box::leak(Box::new(TaskStateSegment::new()));

    let (double_fault_stack, _, _) =
        allocate_kernel_stack().expect("Cannot allocate stack for DOUBLE_FAULT_IST_INDEX");
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = double_fault_stack;

    let (page_fault_stack, _, _) =
        allocate_kernel_stack().expect("Cannot allocate stack for PAGE_FAULT_IST_INDEX");
    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = page_fault_stack;

    tss.privilege_stack_table[PrivilegeLevel::Ring0 as usize] = kernel_stack_top; // No magic numbers on my watch

    tss
}

fn build_gdt(tss: *const TaskStateSegment) -> (&'static mut GlobalDescriptorTable, Selectors) {
    let gdt = Box::leak(Box::new(GlobalDescriptorTable::new()));
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let user_data_selector = gdt.append(Descriptor::user_data_segment());
    let user_code_selector = gdt.append(Descriptor::user_code_segment());
    let tss_selector = unsafe { gdt.append(Descriptor::tss_segment_unchecked(tss)) };
    (
        gdt,
        Selectors {
            code_selector,
            data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector,
        },
    )
}

#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub struct SelectorsWrapper(pub UnsafeCell<MaybeUninit<Selectors>>);

unsafe impl Sync for SelectorsWrapper {}
unsafe impl Send for SelectorsWrapper {}

pub fn init_gdt(
    kernel_stack_top: VirtAddr,
) -> (
    &'static mut GlobalDescriptorTable,
    &'static mut TaskStateSegment,
) {
    let tss = build_tss(kernel_stack_top);
    let (gdt, selectors) = build_gdt(tss);

    unsafe {
        SEGMENTS.0.get().replace(MaybeUninit::new(selectors));
    }

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

pub fn segment_selectors() -> &'static Selectors {
    unsafe { (*SEGMENTS.0.get()).assume_init_ref() }
}
