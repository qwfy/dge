use super::error::Error;

pub async fn accept_failure(msg: (), error: Error) -> Result<(), Error> {
    println!("failure accepted");
    Ok(())
}
