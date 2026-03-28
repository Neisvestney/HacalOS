use crate::process::{Process, ProcessId, ProcessStatus};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::{Once, RwLock};
use crate::process::thread::ThreadStatus;
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
}

unsafe impl Sync for ProcessesManager {}
unsafe impl Send for ProcessesManager {}

#[derive(Debug)]
pub enum ProcessesManagerError {
    ProcessNotFound,
}

pub fn init_processes_manager() {
    PROCESSES_MANAGER.call_once(|| RwLock::new(ProcessesManager::new()));
}
