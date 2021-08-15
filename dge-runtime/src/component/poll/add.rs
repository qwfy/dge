use std::collections::VecDeque;
use std::fmt::Display;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

use log::debug;
use log::info;
use log::warn;
use serde::Serialize;
use serde_json;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;
use tokio::sync::SemaphorePermit;

use crate::error::Result;
use crate::rmq_primitive;

use super::data::*;

pub fn new_job<InputMsg>(msg: InputMsg) -> Arc<RwLock<Job<InputMsg>>> {
    Arc::new(RwLock::new(Job {
        last_scheduled: 0,
        done: false,
        ticket: Arc::new(Semaphore::new(1)),
        msg,
    }))
}

pub async fn add_to_jobs<InputMsg>(
    jobs: Jobs<InputMsg>,
    _channel: lapin::Channel,
    msg: InputMsg,
) -> Result<rmq_primitive::Responsibility>
where
    InputMsg: Display,
{
    debug!("adding job for msg {} to the job queue", &msg);
    let job = new_job(msg);
    {
        let mut jobs = jobs.write().await;
        // new jobs are added to the tail of the queue to be fair to the old jobs
        jobs.push_back(job);
    }
    debug!("job added");
    Ok(rmq_primitive::Responsibility::Accept)
}