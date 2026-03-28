use crate::process::{Process, ProcessId, ProcessStatus};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use log::info;
use spin::{Once, RwLock};
use thiserror::Error;
use crate::process::thread::{ThreadId, ThreadStatus};
use crate::process::unloader::unload_process_from_memory;
use crate::scheduler::global_scheduler::GlobalScheduler;
use crate::scheduler::signals::ThreadSignal;

pub static PROCESSES_MANAGER: Once<RwLock<ProcessesManager>> = Once::new();

#[derive(Debug)]
pub struct ProcessesManager {
    pub processes: BTreeMap<ProcessId, Process>,
    pub next_process_id: ProcessId,
}

impl ProcessesManager {
    pub fn new() -> Self {
        ProcessesManager {
            processes: BTreeMap::new(),
            next_process_id: 1,
        }
    }

    pub fn get_next_process_id(&mut self) -> ProcessId {
        let id = self.next_process_id;
        self.next_process_id += 1;
        id
    }

    pub fn add_process(&mut self, mut process: Process) {
        self.processes.insert(process.id, process);
    }

    pub fn kill_process(&mut self, process_id: ProcessId, global_scheduler: &GlobalScheduler) -> Result<(), ProcessesManagerError> {
        let process = self.processes.get_mut(&process_id).ok_or(ProcessesManagerError::ProcessNotFound)?;
        process.status = ProcessStatus::Exiting(-1);

        for thread in &mut process.threads {
            thread.status = ThreadStatus::Exiting;
            global_scheduler.schedule_signal(process_id, thread.id, ThreadSignal::Terminate, true);
        }
        Ok(())
    }

    pub fn remove_thread_and_cleanup(&mut self, process_id: ProcessId, thread_id: ThreadId) -> Result<(), ProcessesManagerError> {
        let process = self.processes.get_mut(&process_id).ok_or(ProcessesManagerError::ProcessNotFound)?;
        let thread_index = process.threads.iter().position(|thread| thread.id == thread_id).ok_or(ProcessesManagerError::ThreadNotFound)?;
        process.threads.remove(thread_index);

        if process.threads.is_empty() {
            let mut process = self.processes.remove(&process_id).unwrap();
            unload_process_from_memory(&mut process);
        }

        Ok(())
    }
}

unsafe impl Sync for ProcessesManager {}
unsafe impl Send for ProcessesManager {}

#[derive(Debug, Error)]
pub enum ProcessesManagerError {
    #[error("Process not found")]
    ProcessNotFound,
    #[error("Thread not found")]
    ThreadNotFound,
}

pub fn init_processes_manager() {
    PROCESSES_MANAGER.call_once(|| RwLock::new(ProcessesManager::new()));
}
