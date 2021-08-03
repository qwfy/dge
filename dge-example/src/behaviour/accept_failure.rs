use super::error::Error;

pub async fn accept_failure(msg: &i32, error: Error) -> Result<(), Error> {
    println!("failure accepted");
    Ok(())
}
