use super::error::Error;
use super::data::Integer;

pub type State = ();

pub async fn init() -> () {
    ()
}

pub async fn handle(_state: State, msg: &Integer) -> Result<Integer, Error> {
    let Integer {msg_id, integer} = msg;
    Ok(Integer {
        msg_id: msg_id.clone(),
        integer: integer * integer
    })
}
