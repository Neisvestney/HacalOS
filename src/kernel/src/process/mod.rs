use crate::memory::paging::page_table_mapper_from_table_pointer;
use crate::memory::stack::stack_top_from_slice;
use crate::memory::virtual_memory_allocator::VirtualMemoryAllocator;
use crate::process::memory_map::{ProcessMemoryMapEntry, ProcessMemoryMapEntryType};
use crate::process::thread::{Thread, ThreadId};
use crate::scheduler::thread_context::ThreadContext;
use crate::{
    USER_PROCESS_STACK_PAGES_COUNT, USER_PROCESS_VIRTUAL_MEMORY_REGION_PAGES_COUNT,
    USER_PROCESS_VIRTUAL_MEMORY_REGION_START,
};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter};
use derivative::Derivative;
use log::info;
use x86_64::VirtAddr;
use x86_64::structures::paging::{PageTable, PhysFrame};

pub mod loader;
pub mod memory_map;
pub mod processes_manager;
pub mod thread;
pub mod unloader;

pub type ProcessId = u64;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Process {
    pub id: ProcessId,
    pub exe: Arc<String>,
    pub status: ProcessStatus,
    pub page_table: *mut PageTable,
    pub page_table_phys_frame: PhysFrame,
    pub entry_point: VirtAddr,
    pub threads: Vec<Thread>,
    pub next_thread_id: ThreadId,
    pub virtual_memory_allocator: VirtualMemoryAllocator,
    pub memory_map: Vec<ProcessMemoryMapEntry>,
}

impl Process {
    pub fn new(
        id: ProcessId,
        exe: Arc<String>,
        page_table: *mut PageTable,
        page_table_phys_frame: PhysFrame,
        entry_point: VirtAddr,
        memory_map: Vec<ProcessMemoryMapEntry>,
    ) -> Process {
        Process {
            id,
            exe,
            status: ProcessStatus::Running,
            page_table,
            page_table_phys_frame,
            entry_point,
            threads: Vec::with_capacity(1),
            next_thread_id: 1,
            virtual_memory_allocator: VirtualMemoryAllocator::new(
                USER_PROCESS_VIRTUAL_MEMORY_REGION_START,
                USER_PROCESS_VIRTUAL_MEMORY_REGION_PAGES_COUNT,
                false,
                true,
            ),
            memory_map,
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
        let (stack, stack_guard_page, page_range) = self
            .virtual_memory_allocator
            .alloc_pages_with_protection_page(
                USER_PROCESS_STACK_PAGES_COUNT,
                &mut process_page_table_mapper,
            )
            .map_err(|_| SpawnThreadError::OutOfMemory)?;

        self.memory_map.push(ProcessMemoryMapEntry::new(
            page_range,
            ProcessMemoryMapEntryType::new_stack(stack_guard_page),
        ));

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

impl Drop for Process {
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            info!("Dropping {}", self);
        }
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Process [pid: {}, exe: '{}']", self.id, self.exe)
    }
}

#[derive(Debug)]
pub enum ProcessStatus {
    Running,
    Exiting(i32),
    Exited(i32),
}

#[derive(Debug)]
pub enum SpawnThreadError {
    OutOfMemory,
    MainThreadAlreadyExists,
}
