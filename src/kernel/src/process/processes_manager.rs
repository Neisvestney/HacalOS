use crate::process::{Process, ProcessId};
use alloc::collections::BTreeMap;
use spin::{Once, RwLock};

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

    pub fn add_process(&mut self, process: Process) {
        self.processes.insert(process.id, process);
    }
}

unsafe impl Sync for ProcessesManager {}
unsafe impl Send for ProcessesManager {}

pub fn init_processes_manager() {
    PROCESSES_MANAGER.call_once(|| RwLock::new(ProcessesManager::new()));
}
