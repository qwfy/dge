use lapin;
use thiserror;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // runtime errors
    #[error("Failed to publish to {}, error: {}", .queue, .error)]
    FailedToPublishRmqMsg { queue: String, error: String },

    #[error(transparent)]
    RabbitMQError(#[from] lapin::Error),

    // errors returned by user functions
    #[error("User error: {}", .error)]
    UserError { error: String },
}
