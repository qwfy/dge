use super::error::Error;

pub async fn init_state() -> () {
    ()
}

pub async fn handle(state: (), input_msg: &i32) -> Result<i32, Error> {
    Ok(input_msg + 2)
}