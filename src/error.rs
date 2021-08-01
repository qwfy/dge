use thiserror;

use crate::graph::Edge;
use crate::graph::Node;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node is ill formed: {}", .node)]
    IllFormedNode { node: String },
}
