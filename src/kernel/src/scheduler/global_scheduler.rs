use crate::GLOBAL_SCHEDULER;
use crate::scheduler::thread_context::ThreadContext;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use spin::RwLock;
use crate::process::ProcessId;
use crate::process::thread::ThreadId;
use crate::scheduler::signals::{Signals, ThreadSignal};

pub struct GlobalScheduler {
    pub threads_queue: RwLock<VecDeque<Box<ThreadContext>>>,
    pub pending_signals: RwLock<BTreeMap<ProcessId, Signals>>,
}

impl GlobalScheduler {
    pub fn new() -> Self {
        GlobalScheduler {
            threads_queue: RwLock::new(VecDeque::new()),
            pending_signals: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn schedule_thread(&self, thread: ThreadContext) {
        self.threads_queue.write().push_back(Box::new(thread));
    }

    pub fn push_thread_back_and_get_next(
        &self,
        thread: Box<ThreadContext>,
    ) -> Option<Box<ThreadContext>> {
        let mut threads_queue = self.threads_queue.write();
        threads_queue.push_back(thread);
        self.get_next_in(&mut threads_queue)
    }

    pub fn get_next(&self) -> Option<Box<ThreadContext>> {
        let mut threads_queue = self.threads_queue.write();
        self.get_next_in(&mut threads_queue)
    }

    fn get_next_in(
        &self,
        threads_queue: &mut VecDeque<Box<ThreadContext>>,
    ) -> Option<Box<ThreadContext>> {
        threads_queue.pop_front()
    }

    pub fn schedule_signal(&self, process_id: ProcessId, thread_id: ThreadId, signal: ThreadSignal, push_front: bool) {
        let mut pending_signals = self.pending_signals.write();
        let process_pending_signals = pending_signals.entry(process_id).or_default();
        let thread_pending_signals = process_pending_signals.thread_signals.entry(thread_id).or_default();
        if push_front {
            thread_pending_signals.push_front(signal);
        } else {
            thread_pending_signals.push_back(signal);
        }
    }

    pub fn get_next_pending_signal(&self, process_id: ProcessId, thread_id: ThreadId) -> Option<ThreadSignal> {
        let mut pending_signals_guard = self.pending_signals.write();
        if let Some(process_pending_signals) = pending_signals_guard.get_mut(&process_id) {
            if let Some(thread_pending_signals) = process_pending_signals.thread_signals.get_mut(&thread_id) {
                let pending_signal = thread_pending_signals.pop_front();
                pending_signal
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn remove_process_signals_queue(&self, process_id: ProcessId) {
        let mut pending_signals_guard = self.pending_signals.write();
        pending_signals_guard.remove(&process_id);
    }
}

pub struct GlobalSchedulerWrapper(pub UnsafeCell<MaybeUninit<GlobalScheduler>>);

unsafe impl Sync for GlobalSchedulerWrapper {}
unsafe impl Send for GlobalSchedulerWrapper {}

pub fn init_scheduler() {
    unsafe {
        GLOBAL_SCHEDULER
            .0
            .get()
            .replace(MaybeUninit::new(GlobalScheduler::new()));
    }
}

pub fn global_scheduler() -> &'static GlobalScheduler {
    unsafe { (*GLOBAL_SCHEDULER.0.get()).assume_init_ref() }
}
