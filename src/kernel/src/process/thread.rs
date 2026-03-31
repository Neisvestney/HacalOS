use crate::process::ProcessId;
use crate::scheduler::thread_context::ThreadContext;
use alloc::string::String;
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, PhysFrame};

pub type ThreadId = u64;

#[derive(Debug)]
pub struct Thread {
    pub id: ThreadId,
    pub name: Option<String>,
    pub status: ThreadStatus,
    pub stack_guard_page: Page,
}

#[derive(Debug)]
pub enum ThreadStatus {
    Running,
    Blocked(ThreadContext),
    Sleeping,
    Exiting,
}

impl Thread {
    pub fn new_with_context(
        id: ThreadId,
        pid: ProcessId,
        name: Option<String>,
        start_instruction_point: VirtAddr,
        stack_pointer: VirtAddr,
        stack_guard_page: Page,
        page_table_phys_frame: PhysFrame,
        syscall_stack: VirtAddr,
    ) -> (Self, ThreadContext) {
        let context = ThreadContext::new(
            id,
            pid,
            start_instruction_point,
            stack_pointer,
            page_table_phys_frame,
            syscall_stack,
        );

        let thread = Thread {
            id,
            name,
            status: ThreadStatus::Running,
            stack_guard_page,
        };

        (thread, context)
    }
}
