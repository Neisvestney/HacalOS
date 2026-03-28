use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use crate::process::thread::ThreadId;

#[derive(Debug, Clone, Default)]
pub struct Signals {
    pub thread_signals: BTreeMap<ThreadId, VecDeque<ThreadSignal>>,
}

impl Signals {
    pub fn new() -> Self {
        Signals {
            thread_signals: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ThreadSignal {
    Terminate,
}