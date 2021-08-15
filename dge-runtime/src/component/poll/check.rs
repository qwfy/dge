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

use crate::rmq_primitive;
use super::data::*;


pub async fn poll_forever<InputMsg, OutputMsg, UserError, Context, CheckResult, AcceptFailureResult>(
    capacity: Capacity,
    jobs: Jobs<InputMsg>,

    // these are used when do the actual checking
    check: fn(InputMsg) -> CheckResult,
    accept_failure: fn(Context, UserError) -> AcceptFailureResult,
    get_rmq_uri: fn() -> String,
    work_exchange: &'static str,
    output_queue: Option<&'static str>,
)
    where
        InputMsg: Clone + Debug + Send + Sync + 'static,
        Context: From<InputMsg> + Send + 'static,
        OutputMsg: Serialize + Send + 'static,
        UserError: From<serde_json::Error> + Debug + Send + 'static,
        CheckResult: Future<Output=Result<Option<OutputMsg>, UserError>> + Send + 'static,
        AcceptFailureResult: Future<Output=Result<(), UserError>> + Send + 'static,
{
    // the maximum number of running jobs
    let slots = Arc::new(Semaphore::new(capacity.max_running_jobs as usize));

    loop {
        debug!(
            "there are {} available slots when this pass started",
            slots.available_permits()
        );

        let sleep_time = sweep_once(
            capacity.clone(),
            jobs.clone(),
            slots.clone(),
            check,
            accept_failure,
            get_rmq_uri,
            work_exchange,
            output_queue,
        ).await;

        debug!("this pass is done, sleeping for {} seconds before the next sleep", sleep_time);
        tokio::time::sleep(std::time::Duration::from_secs(sleep_time as u64)).await;
    }
}

async fn sweep_once<InputMsg, OutputMsg, UserError, Context, CheckResult, AcceptFailureResult>(
    capacity: Capacity,
    jobs: Jobs<InputMsg>,
    slots: Arc<Semaphore>,

    // these are used when do the actual checking
    check: fn(InputMsg) -> CheckResult,
    accept_failure: fn(Context, UserError) -> AcceptFailureResult,
    get_rmq_uri: fn() -> String,
    work_exchange: &'static str,
    output_queue: Option<&'static str>,
) -> u32
    where
        InputMsg: Clone + Debug + Send + Sync + 'static,
        Context: From<InputMsg> + Send + 'static,
        OutputMsg: Serialize + Send + 'static,
        UserError: From<serde_json::Error> + Debug + Send + 'static,
        CheckResult: Future<Output=Result<Option<OutputMsg>, UserError>> + Send + 'static,
        AcceptFailureResult: Future<Output=Result<(), UserError>> + Send + 'static,
{
    // the entire job queue is locked during the sweep,
    // this means that new jobs cannot be added to the queue
    let mut jobs = jobs.write().await;

    let total_jobs = jobs.len();
    let mut processed_jobs = 0;

    let mut sleep_time = capacity.sweep_sleep_seconds_default;

    // loop over the jobs to dispatch
    loop {
        // every job in the queue is accounted for, run out of jobs
        if processed_jobs >= total_jobs {
            debug!("all {} jobs in the queue are accounted for, terminating this pass", total_jobs);
            break;
        }

        match slots.clone().try_acquire_owned() {
            // run out of slots
            Err(e) => {
                debug!("no available slots, terminating this pass");
                break;
            }

            Ok(slot) => match jobs.pop_front() {
                // run out of jobs
                None => {
                    debug!("job queue is empty, terminating this pass");
                    break;
                }

                Some(job) => {
                    processed_jobs += 1;

                    let (job_status, new_sleep_time) = schedule_one(
                        capacity.clone(),
                        job.clone(),
                        slot,
                        sleep_time,
                        check,
                        accept_failure,
                        get_rmq_uri,
                        work_exchange,
                        output_queue,
                    ).await;
                    sleep_time = new_sleep_time;

                    match job_status {
                        // the job is done, discard it by not pushing it back
                        JobStatus::JobDone => (),

                        // the job is still in progress, put it back for later examination
                        JobStatus::AlreadyRunning
                        | JobStatus::NoUpForDispatch
                        | JobStatus::Dispatched => jobs.push_back(job),
                    }
                }
            },
        }
    }

    sleep_time
}

enum JobStatus {
    JobDone,
    AlreadyRunning,
    Dispatched,
    NoUpForDispatch,
}

async fn schedule_one<InputMsg, OutputMsg, UserError, Context, CheckResult, AcceptFailureResult>(
    capacity: Capacity,
    job: Arc<RwLock<Job<InputMsg>>>,
    slot: OwnedSemaphorePermit,
    sleep_time: u32,

    // these are used when do the actual checking
    check: fn(InputMsg) -> CheckResult,
    accept_failure: fn(Context, UserError) -> AcceptFailureResult,
    get_rmq_uri: fn() -> String,
    work_exchange: &'static str,
    output_queue: Option<&'static str>,
) -> (JobStatus, u32)
    where
        InputMsg: Clone + Debug + Send + Sync + 'static,
        Context: From<InputMsg> + Send + 'static,
        OutputMsg: Serialize + Send + 'static,
        UserError: From<serde_json::Error> + Debug + Send + 'static,
        CheckResult: Future<Output=Result<Option<OutputMsg>, UserError>> + Send + 'static,
        AcceptFailureResult: Future<Output=Result<(), UserError>> + Send + 'static,
{
    let job_clone = job.clone();
    let mut job = job.write().await;

    let mut sleep_time = sleep_time;

    if job.done {
        debug!("job for message {:?} is done", &job.msg);
        (JobStatus::JobDone, sleep_time)
    } else {
        match job.ticket.clone().try_acquire_owned() {
            Err(_) => {
                debug!("skip dispatching job for {:?}, for there is one already running", &job.msg);
                (JobStatus::AlreadyRunning, sleep_time)
            }
            Ok(ticket) => {
                let now = chrono::offset::Utc::now().timestamp();
                let seconds_passed = 0.max(now - job.last_scheduled);
                if seconds_passed >= capacity.job_checking_interval_seconds as i64 {
                    debug!("scheduling job for {:?} to be run", &job.msg);
                    job.last_scheduled = now;
                    tokio::spawn(do_check(
                        job_clone, slot, ticket,
                        check, accept_failure, get_rmq_uri, work_exchange, output_queue
                    ));
                    (JobStatus::Dispatched, sleep_time)
                } else {
                    // this one is not up for scheduling,
                    debug!("time for {:?} has not come yet", &job.msg);

                    // adjust sleep time to be the smallest
                    let seconds_left = capacity.job_checking_interval_seconds - seconds_passed as u32;
                    sleep_time = sleep_time.min(seconds_left as u32);
                    sleep_time = capacity.sweep_sleep_seconds_min.max(sleep_time);
                    sleep_time = capacity.sweep_sleep_seconds_max.min(sleep_time);

                    (JobStatus::NoUpForDispatch, sleep_time)
                }
            }
        }
    }
}

macro_rules! return_on_failure {
    (
        result=$result:expr,
        job=$job:expr,
        accept_failure=$accept_failure:path,
        input_msg=$input_msg:expr,
        $(,)?
    ) => {{
        let msg_for_logging = $input_msg.clone();
        match $result {
            Ok(t) => t,
            Err(user_error) => {
                // error occurred, try to accept it and mark the job as done,
                // if there is an error when accepting the failure, leave the job's status unchanged
                warn!(
                    "error occurred when checking status for {:?}, accepting it. error is: {:?}",
                    &msg_for_logging, &user_error
                );

                let mut write_job = $job.write().await;
                if !write_job.done {
                    match $accept_failure($input_msg.into(), user_error).await {
                        Ok(()) => {
                            // failure accepted, mark the job as done
                            info!("failure for {:?} accepted, marking the job to be done", &msg_for_logging);
                            write_job.done = true;
                        }
                        Err(accept_failure_user_error) => {
                            // failed to accept failure, leave the done status unchanged
                            warn!(
                                "failed to accept failure for msg {:?}, error is {:?}",
                                &msg_for_logging, &accept_failure_user_error
                            );
                        }
                    }
                } else {
                    warn!("job for {:?} is already done", &msg_for_logging);
                }

                // return from the caller
                return ();
            }
        }
    }};
}

async fn do_check<InputMsg, OutputMsg, UserError, Context, CheckResult, AcceptFailureResult>(
    job: Arc<RwLock<Job<InputMsg>>>,
    slot: OwnedSemaphorePermit,
    ticket: OwnedSemaphorePermit,

    // these are used when do the actual checking
    check: fn(InputMsg) -> CheckResult,
    accept_failure: fn(Context, UserError) -> AcceptFailureResult,
    get_rmq_uri: fn() -> String,
    work_exchange: &'static str,
    output_queue: Option<&'static str>,
)
    where
        InputMsg: Clone + Debug + Send + Sync + 'static,
        Context: From<InputMsg> + Send + 'static,
        OutputMsg: Serialize + Send + 'static,
        UserError: From<serde_json::Error> + Debug + Send + 'static,
        CheckResult: Future<Output=Result<Option<OutputMsg>, UserError>> + Send + 'static,
        AcceptFailureResult: Future<Output=Result<(), UserError>> + Send + 'static,
{
    let msg = job.read().await.msg.clone();

    // run the check
    let check_result = check(msg.clone()).await;
    let msg_clone = msg.clone();
    let maybe_output_msg = return_on_failure!(
        result = check_result,
        job = job,
        accept_failure = accept_failure,
        input_msg = msg_clone,
    );

    match maybe_output_msg {
        None => {
            // job is still in progress, do nothing
            debug!("job for {:?} is still in progress", &msg);
            ()
        }
        Some(output_msg) => {
            // acquire lock before handling the result,
            // in case of that the job is already handled by another thead,
            // (currently this cannot happen,
            // since this is only the running instance of the job,
            // as guarded by the ticket)
            let mut write_job = job.write().await;
            if !write_job.done {
                match output_queue {
                    None => (),
                    Some(output_queue) => {
                        debug!("serializing output");
                        let serialization_result =
                            serde_json::to_vec(&output_msg).map_err(|e| e.into());
                        let msg_clone = msg.clone();
                        let output_msg_vec = return_on_failure!(
                            result = serialization_result,
                            job = job,
                            accept_failure = accept_failure,
                            input_msg = msg_clone,
                        );
                        let rmq_uri = get_rmq_uri();
                        debug!("sending output to queue {}", output_queue);
                        let channel = match rmq_primitive::create_channel(rmq_uri).await {
                            Err(e) => {
                                // return to leave the job status unchanged
                                warn!("failed to create RabbitMQ channel, error is {:?}", e);
                                return ();
                            }
                            Ok(channel) => channel
                        };
                        let () = match rmq_primitive::publish(
                            channel,
                            work_exchange,
                            output_queue,
                            output_msg_vec,
                        ).await {
                            Err(e) => {
                                // return to leave the job status unchanged
                                warn!("failed to publish message, error is {:?}", e);
                                return ();
                            }
                            Ok(()) => {
                                debug!("output sent to queue {}", output_queue);
                                ()
                            }
                        };
                    }
                }
                // mark the job as done
                write_job.done = true;
            }
        }
    }
}
