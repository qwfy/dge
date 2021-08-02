use petgraph;
pub use petgraph::graph::EdgeIndex;
pub use petgraph::graph::NodeIndex;
use uuid::Uuid;

use crate::graph::Node::UserHandler;

pub(crate) type PetGraph = petgraph::Graph<Node, Edge>;

/// A node represents the computation.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    /// A no-op node that indicates the start of the computation.
    Start,
    /// Wait for all incoming edges, merge the incoming messages for later consumption.
    WaitAll { merge_messages: String },
    /// Wait for any incoming edges, merge the incoming messages for later consumption.
    WaitAny { merge_messages: String },
    /// Duplicate the output of one node to multiple nodes.
    FanOut,
    /// A user-provided handler that transform the input message into the output message.
    UserHandler { module: String },
}

/// An edge represents a RabbitMQ queue.
#[derive(Clone, Debug)]
pub(crate) struct Edge {
    pub(crate) queue: String,
}

impl Node {
    pub(crate) fn is_start(&self) -> bool {
        match self {
            Node::Start { .. } => true,
            Node::WaitAll { .. } => false,
            Node::WaitAny { .. } => false,
            Node::FanOut => false,
            Node::UserHandler { .. } => false,
        }
    }
}

/// A computational graph where:
///
/// - edges represent messages delivered via RabbitMQ queue
/// - nodes represent computations that transform the input message into output message
pub struct Graph {
    pub(crate) g: PetGraph,
    pub(crate) accept_failure: String,
}

impl Graph {
    /// Create a new computational graph.
    pub fn new(accept_failure: String) -> Graph {
        Graph {
            g: petgraph::Graph::new(),
            accept_failure,
        }
    }

    /// Represent the start of the computation.
    /// Usually this is the first function been called to acquire a starting point
    /// for later operations after the graph is created.
    ///
    /// Return a handle to the start node.
    pub fn start(&mut self) -> NodeIndex {
        self.g.add_node(Node::Start)
    }

    /// Reads the output of the `input` node from the RabbitMQ queue `queue`
    /// and process it with `handler`.
    ///
    /// Return a handle to the `handler` node.
    ///
    /// Internally this will create a new node for `handler`,
    /// and a new edge from `input` to `handler` representing the underlying RabbitMQ queue `queue`.
    pub fn process(&mut self, input: NodeIndex, queue: String, handler: String) -> NodeIndex {
        let handler_node = UserHandler { module: handler };
        let handler_node_i = self.g.add_node(handler_node);
        let edge = Edge { queue };
        self.g.add_edge(input, handler_node_i, edge);
        handler_node_i
    }

    /// Add a node that waits for all messages from `inputs` that belong to a single run,
    /// and merge them for later consumption.
    ///
    /// Return a handle to the newly added node.
    pub fn wait_all(
        &mut self,
        inputs: Vec<NodeIndex>,
        queue: String,
        merge_messages: String,
    ) -> NodeIndex {
        let wait_node_i = self.g.add_node(Node::WaitAll { merge_messages });
        for input_i in inputs {
            self.g.add_edge(
                input_i,
                wait_node_i,
                Edge {
                    queue: queue.clone(),
                },
            );
        }
        wait_node_i
    }

    /// Like [`wait_all`], but wait at least one messages instead of all of them.
    pub fn wait_any(
        &mut self,
        inputs: Vec<NodeIndex>,
        queue: String,
        merge_messages: String,
    ) -> NodeIndex {
        let wait_any_i = self.g.add_node(Node::WaitAny { merge_messages });
        for input_i in inputs {
            self.g.add_edge(
                input_i,
                wait_any_i,
                Edge {
                    queue: queue.clone(),
                },
            );
        }
        wait_any_i
    }

    /// Create a node that will copy messages of `input` delivered via `queue`
    /// to all outgoing edges of the newly created node.
    ///
    /// Return a handle to the newly created node
    pub fn fan_out(&mut self, input: NodeIndex, queue: String) -> NodeIndex {
        let fan_out_i = self.g.add_node(Node::FanOut);
        self.g.add_edge(input, fan_out_i, Edge { queue });

        fan_out_i
    }
}
