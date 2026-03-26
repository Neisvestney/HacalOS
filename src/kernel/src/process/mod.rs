use crate::memory::paging::page_table_mapper_from_table_pointer;
use crate::memory::stack::stack_top_from_slice;
use crate::memory::virtual_memory_allocator::VirtualMemoryAllocator;
use crate::process::thread::{Thread, ThreadId};
use crate::scheduler::thread_context::ThreadContext;
use crate::{
    USER_PROCESS_STACK_PAGES_COUNT, USER_PROCESS_VIRTUAL_MEMORY_REGION_PAGES_COUNT,
    USER_PROCESS_VIRTUAL_MEMORY_REGION_START,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use x86_64::structures::paging::{PageTable, PhysFrame};
use x86_64::VirtAddr;

pub mod loader;
pub mod memory_map;
pub mod thread;

pub type ProcessId = u64;

pub struct Process {
    pub id: ProcessId,
    pub exe: String,
    pub page_table: *mut PageTable,
    pub page_table_phys_frame: PhysFrame,
    pub entry_point: VirtAddr,
    pub threads: Vec<Thread>,
    pub next_thread_id: ThreadId,
    pub virtual_memory_allocator: VirtualMemoryAllocator,
}
impl Process {
    pub fn new(
        id: ProcessId,
        exe: String,
        page_table: *mut PageTable,
        page_table_phys_frame: PhysFrame,
        entry_point: VirtAddr,
    ) -> Process {
        Process {
            id,
            exe,
            page_table,
            page_table_phys_frame,
            entry_point,
            threads: Vec::with_capacity(1),
            next_thread_id: 0,
            virtual_memory_allocator: VirtualMemoryAllocator::new(
                USER_PROCESS_VIRTUAL_MEMORY_REGION_START,
                USER_PROCESS_VIRTUAL_MEMORY_REGION_PAGES_COUNT,
                false,
            ),
        }
    }

    pub fn add_main_thread(&mut self) -> Result<ThreadContext, SpawnThreadError> {
        if !self.threads.is_empty() {
            return Err(SpawnThreadError::MainThreadAlreadyExists);
        }

        let new_thread_context = self.add_thread(Some("main".to_string()), self.entry_point)?;
        Ok(new_thread_context)
    }

    pub fn add_thread(
        &mut self,
        name: Option<String>,
        start_instruction_point: VirtAddr,
    ) -> Result<ThreadContext, SpawnThreadError> {
        let mut process_page_table_mapper =
            unsafe { page_table_mapper_from_table_pointer(self.page_table) };
        let (stack, stack_guard_page) = self
            .virtual_memory_allocator
            .alloc_pages_with_protection_page(
                USER_PROCESS_STACK_PAGES_COUNT,
                &mut process_page_table_mapper,
            )
            .map_err(|_| SpawnThreadError::OutOfMemory)?;

        let (new_thread, new_thread_context) = Thread::new_with_context(
            self.next_thread_id,
            self.id,
            name,
            start_instruction_point,
            VirtAddr::from_ptr(stack_top_from_slice(stack)),
            stack_guard_page,
            self.page_table_phys_frame,
        );

        self.threads.push(new_thread);
        self.next_thread_id += 1;

        Ok(new_thread_context)
    }
}

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Process")
            .field("id", &self.id)
            .field("exe", &self.exe)
            .field("page_table", &format_args!("{:p}", self.page_table))
            .field(
                "page_table_phys_frame",
                &format_args!("{:#X}", self.page_table_phys_frame.start_address()),
            )
            .field("entry_point", &format_args!("{:#X}", self.entry_point))
            .field("threads", &self.threads)
            .field("next_thread_id", &self.next_thread_id)
            .field("virtual_memory_allocator", &self.virtual_memory_allocator)
            .finish()
    }
}

#[derive(Debug)]
pub enum SpawnThreadError {
    OutOfMemory,
    MainThreadAlreadyExists,
}
