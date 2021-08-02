use askama;
use lapin;
use thiserror;

use crate::graph::Edge;
use crate::graph::Node;

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

    // runtime errors
    #[error("Failed to publish to {}, error: {}", .queue, .error)]
    FailedToPublishRmqMsg { queue: String, error: String },

    #[error(transparent)]
    RabbitMQError(#[from] lapin::Error),

    // errors returned by user functions
    #[error("User error: {}", .error)]
    UserError { error: String },
}
