use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use dge_runtime::component::aggregate::AggregationStatus;

use super::error::Error;
use super::data::Integer;
use super::data::Float;

pub type State = Arc<Mutex<HashMap<String, Phase>>>;

pub async fn init() -> State {
    let state = HashMap::new();
    Arc::new(Mutex::new(state))
}

#[derive(Clone)]
pub enum Phase {
    HaveOneNumber(i32),
    HaveTwoNumber(i32, i32)
}

pub async fn aggregate(state: State, msg: &Integer) -> Result<AggregationStatus<Float>, Error> {
    let mut state = state.lock().unwrap();
    let v = match state.get(&msg.msg_id) {
        None => None,
        Some(x) => Some(x.clone())
    };
    let status = match v {
        None => {
            state.insert(msg.msg_id.clone(), Phase::HaveOneNumber(msg.integer.clone()));
            AggregationStatus::Ignore
        },
        Some(Phase::HaveTwoNumber(_, _)) => {
            AggregationStatus::Ignore
        },
        Some(Phase::HaveOneNumber(existing)) => {
            state.insert(msg.msg_id.clone(), Phase::HaveTwoNumber(existing, msg.integer.clone()));
            AggregationStatus::Aggregated(Float {
                msg_id: msg.msg_id.clone(),
                // since this is an example, we just unwrap it
                float: (existing * msg.integer) as f32,
            })
        },
    };

    Ok(status)
}
