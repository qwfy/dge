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

pub enum Context {
    OfInteger(Integer),
    OfFloat(Float),
}

impl From<&Integer> for Context {
    fn from(x: &Integer) -> Context {
        Context::OfInteger(x.clone())
    }
}

impl From<&Float> for Context {
    fn from(x: &Float) -> Context {
        Context::OfFloat(x.clone())
    }
}