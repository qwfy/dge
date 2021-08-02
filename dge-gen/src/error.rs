use askama;
use thiserror;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // graph related errors
    #[error("Node is ill formed: {}", .node)]
    IllFormedNode { node: String },

    #[error(transparent)]
    AskamaError(#[from] askama::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
