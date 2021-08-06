use super::error::Error;
use super::data::ErrorContext;

pub async fn accept_failure(_context: ErrorContext, _error: Error) -> Result<(), Error> {
    println!("failure accepted");
    Ok(())
}
