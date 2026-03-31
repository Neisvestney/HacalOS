pub mod cpu_registries_context;
pub mod global_scheduler;
pub mod schedule_on_interrupt;
pub mod signals;
pub mod sleeping_thread;
pub mod thread_context;

pub const SCHEDULE_TICKS: u64 = 20;
