use super::error::Error;

pub enum ErrorContext {
    OfI32(i32),
    OfString(String),
}

impl From<&i32> for ErrorContext {
    fn from(x: &i32) -> ErrorContext {
        ErrorContext::OfI32(*x)
    }
}

impl From<&String> for ErrorContext {
    fn from(x: &String) -> ErrorContext {
        ErrorContext::OfString(x.clone())
    }
}

pub async fn accept_failure(msg: &i32, error: Error) -> Result<(), Error> {
    println!("failure accepted");
    Ok(())
}
