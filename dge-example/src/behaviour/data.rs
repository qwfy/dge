use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Integer {
    pub msg_id: String,
    pub integer: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Float {
    pub msg_id: String,
    pub float: f32,
}

pub enum ErrorContext {
    OfInteger(Integer),
    OfFloat(Float),
}

impl From<&Integer> for ErrorContext {
    fn from(x: &Integer) -> ErrorContext {
        ErrorContext::OfInteger(x.clone())
    }
}

impl From<&Float> for ErrorContext {
    fn from(x: &Float) -> ErrorContext {
        ErrorContext::OfFloat(x.clone())
    }
}