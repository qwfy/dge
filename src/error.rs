use thiserror;

use crate::graph::Edge;
use crate::graph::Node;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Expect 0 or 1 outgoing edges for node: {}", .node)]
    ExpectZeroOrOneOutGoingEdge { node: String },

    #[error("Node is ill formed: {}", .node)]
    IllFormedNode { node: String },
}
