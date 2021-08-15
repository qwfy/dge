use super::error::Error;
use super::data::Context;

pub async fn accept_failure(_context: Context, _error: Error) -> Result<(), Error> {
    println!("failure accepted");
    Ok(())
}
