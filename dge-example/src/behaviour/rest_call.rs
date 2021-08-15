use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use dge_runtime::component::poll::Capacity;

use super::error::Error;
use super::data::Integer;
use super::data::Float;

pub async fn init() -> Vec<Float> {
    // maybe load unfinished jobs from the database
    vec![]
}

pub fn get_capacity() -> Capacity {
    std::default::Default::default()
}

pub async fn check(msg: Float) -> Result<Option<Integer>, Error> {
    let rest_result = {
        // simulating a rest call
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        42
    };
    Ok(Some(Integer {
        msg_id: msg.msg_id,
        integer: rest_result,
    }))
}