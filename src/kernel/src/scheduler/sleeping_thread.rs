use crate::scheduler::thread_context::ThreadContext;
use alloc::boxed::Box;
use core::cmp::Ordering;

#[derive(Debug)]
pub struct SleepingThread {
    pub wake_at: u64,
    pub thread_context: Box<ThreadContext>,
}

impl SleepingThread {
    pub fn new(thread_context: Box<ThreadContext>, wake_at: u64) -> Self {
        SleepingThread {
            wake_at,
            thread_context,
        }
    }
}

impl PartialEq for SleepingThread {
    fn eq(&self, other: &Self) -> bool {
        self.wake_at == other.wake_at
    }
}

impl Eq for SleepingThread {}

impl PartialOrd for SleepingThread {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SleepingThread {
    fn cmp(&self, other: &Self) -> Ordering {
        other.wake_at.cmp(&self.wake_at)
    }
}
