use std::collections::VecDeque;
use std::fmt::Debug;
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

#[macro_export]
macro_rules! add_to_jobs {
(
    jobs=$jobs:expr,
    msg=$msg:expr,
    save_msg=$save_msg:path
    $(,)?
) => {{
    debug!("adding job for msg {:?} to the job queue", &$msg);
    match $save_msg($msg.clone()).await {
        Err(user_error) => {
            // this will be retried
            warn!("failed to save message {:?}, error is: {:?}", &$msg, user_error);
            Ok(rmq_primitive::Responsibility::Reject)
        }
        Ok(()) => {
            let job = new_job($msg);
            let mut jobs = $jobs.write().await;
            // new jobs are added to the tail of the queue to be fair to the old jobs
            jobs.push_back(job);
            debug!("job added");
            Ok(rmq_primitive::Responsibility::Accept)
        }
    }
}}
}
