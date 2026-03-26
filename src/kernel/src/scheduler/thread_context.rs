use crate::gdt::segment_selectors;
use crate::process::ProcessId;
use crate::process::thread::ThreadId;
use crate::scheduler::cpu_registries_context::CpuRegistriesContext;
use x86_64::VirtAddr;
use x86_64::registers::rflags::RFlags;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::paging::PhysFrame;

#[derive(Debug)]
pub struct ThreadContext {
    pub process_id: ProcessId,
    pub thread_id: ThreadId,
    pub cpu_registries_context: CpuRegistriesContext,
    pub page_table_phys_frame: PhysFrame,
    // TODO xmm registers and etc
}

impl ThreadContext {
    pub fn new(
        thread_id: ThreadId,
        process_id: ProcessId,
        instruction_pointer: VirtAddr,
        stack_pointer: VirtAddr,
        page_table_phys_frame: PhysFrame,
    ) -> Self {
        let segments = segment_selectors();

        let mut cpu_registries_context = CpuRegistriesContext::empty();

        cpu_registries_context.stack_frame.instruction_pointer = instruction_pointer;
        cpu_registries_context.stack_frame.stack_pointer = stack_pointer;
        cpu_registries_context.stack_frame.cpu_flags =
            RFlags::from_bits_truncate(0x2) | RFlags::INTERRUPT_FLAG;
        cpu_registries_context.stack_frame.code_segment = segments.user_code_selector;
        cpu_registries_context.stack_frame.stack_segment = segments.user_data_selector;

        ThreadContext {
            process_id,
            thread_id,
            cpu_registries_context,
            page_table_phys_frame,
        }
    }
}
