use super::error::Error;
pub type State = ();
pub async fn init() -> () {
    ()
}

pub async fn handle(state: State, input_msg: &i32) -> Result<i32, Error> {
    Ok(input_msg + 2)
}
