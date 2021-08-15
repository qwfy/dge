use log::debug;
use log::info;
use std::collections::VecDeque;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;
use tokio::sync::SemaphorePermit;


pub struct Job<T> {
    pub last_scheduled: i64,
    pub done: bool,
    // ensure that there is only one instance of this job
    pub ticket: Arc<Semaphore>,

    // the initiating message
    pub msg: T,
}


pub type Jobs<T> = Arc<RwLock<VecDeque<Arc<RwLock<Job<T>>>>>>;


#[derive(Clone)]
pub struct Capacity {
    pub max_running_jobs: u32,

    pub sweep_sleep_seconds_default: u32,
    pub sweep_sleep_seconds_min: u32,
    pub sweep_sleep_seconds_max: u32,

    pub job_checking_interval_seconds: u32,
}

impl std::default::Default for Capacity {
    fn default() -> Self {
        Capacity {
            max_running_jobs: 10,
            sweep_sleep_seconds_default: 5,
            sweep_sleep_seconds_min: 1,
            sweep_sleep_seconds_max: 60,
            job_checking_interval_seconds: 10,
        }
    }
}