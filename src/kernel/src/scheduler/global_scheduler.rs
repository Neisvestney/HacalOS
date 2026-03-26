use alloc::boxed::Box;
use crate::GLOBAL_SCHEDULER;
use crate::scheduler::thread_context::ThreadContext;
use alloc::collections::VecDeque;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use spin::{RwLock, RwLockReadGuard};

pub struct GlobalScheduler {
    pub threads_queue: RwLock<VecDeque<Box<ThreadContext>>>,
}

impl GlobalScheduler {
    pub fn new() -> Self {
        GlobalScheduler {
            threads_queue: RwLock::new(VecDeque::new()),
        }
    }

    pub fn schedule_thread(&self, thread: ThreadContext) {
        self.threads_queue.write().push_back(Box::new(thread));
    }

    pub fn push_thread_back_and_get_next(&self, thread: Box<ThreadContext>) -> Option<Box<ThreadContext>> {
        let mut threads_queue = self.threads_queue.write();
        threads_queue.push_front(thread);
        self.get_next_in(&mut threads_queue)
    }

    pub fn get_next(&self) -> Option<Box<ThreadContext>> {
        let mut threads_queue = self.threads_queue.write();
        self.get_next_in(&mut threads_queue)
    }

    fn get_next_in(&self, threads_queue: &mut VecDeque<Box<ThreadContext>>) -> Option<Box<ThreadContext>> {
        threads_queue.pop_front()
    }
}

pub struct GlobalSchedulerWrapper(pub UnsafeCell<MaybeUninit<GlobalScheduler>>);

unsafe impl Sync for GlobalSchedulerWrapper {}
unsafe impl Send for GlobalSchedulerWrapper {}

pub fn init_scheduler() {
    unsafe {
        GLOBAL_SCHEDULER.0.get().replace(MaybeUninit::new(GlobalScheduler::new()));
    }
}

pub fn global_scheduler() -> &'static GlobalScheduler {
    unsafe {
        (*GLOBAL_SCHEDULER.0.get()).assume_init_ref()
    }
}
