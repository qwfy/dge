use serde_json;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("{}", .0)]
    GeneralError(String),
}
