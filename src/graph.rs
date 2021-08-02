use petgraph;
pub use petgraph::graph::EdgeIndex;
pub use petgraph::graph::NodeIndex;
use uuid::Uuid;

use crate::graph::Node::UserHandler;

pub(crate) type PetGraph = petgraph::Graph<Node, Edge>;

/// A node represents the computation.
///
/// Every node has an unique name associated with it,
/// which will be used as the file name of the generated file.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    /// A no-op node that indicates the start of the computation.
    Start { name: String },
    /// Wait for all incoming edges, merge the incoming messages for later consumption.
    WaitAll {
        name: String,
        merge_messages: String,
    },
    /// Wait for any incoming edges, merge the incoming messages for later consumption.
    WaitAny {
        name: String,
        merge_messages: String,
    },
    /// Duplicate the output of one node to multiple nodes.
    FanOut { name: String },
    /// A user-provided handler that transform the input message into the output message.
    UserHandler {
        name: String,
        behaviour_module: String,
    },
}

/// An edge represents a RabbitMQ queue.
#[derive(Clone, Debug)]
pub(crate) struct Edge {
    pub(crate) queue: String,
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
    pub fn start<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        self.g.add_node(Node::Start { name: name.into() })
    }

    /// Reads the output of the `input` node from the RabbitMQ queue `queue`
    /// and process it with `handler`.
    ///
    /// Return a handle to the `handler` node.
    ///
    /// Internally this will create a new node for `handler`,
    /// and a new edge from `input` to `handler` representing the underlying RabbitMQ queue `queue`.
    pub fn process<S: Into<String>>(
        &mut self,
        name: S,
        input: NodeIndex,
        queue: S,
        behaviour_module: S,
    ) -> NodeIndex {
        let handler_node = UserHandler {
            name: name.into(),
            behaviour_module: behaviour_module.into(),
        };
        let handler_node_i = self.g.add_node(handler_node);
        let edge = Edge {
            queue: queue.into(),
        };
        self.g.add_edge(input, handler_node_i, edge);
        handler_node_i
    }

    /// Add a node that waits for all messages from `inputs` that belong to a single run,
    /// and merge them for later consumption.
    ///
    /// Return a handle to the newly added node.
    pub fn wait_all<S: Into<String>>(
        &mut self,
        name: S,
        inputs: Vec<NodeIndex>,
        queue: S,
        merge_messages: S,
    ) -> NodeIndex {
        let wait_node_i = self.g.add_node(Node::WaitAll {
            name: name.into(),
            merge_messages: merge_messages.into(),
        });
        let queue = queue.into();
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
    pub fn wait_any<S: Into<String>>(
        &mut self,
        name: S,
        inputs: Vec<NodeIndex>,
        queue: S,
        merge_messages: S,
    ) -> NodeIndex {
        let wait_any_i = self.g.add_node(Node::WaitAny {
            name: name.into(),
            merge_messages: merge_messages.into(),
        });
        let queue = queue.into();
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
    pub fn fan_out<S: Into<String>>(&mut self, name: S, input: NodeIndex, queue: S) -> NodeIndex {
        let fan_out_i = self.g.add_node(Node::FanOut { name: name.into() });
        self.g.add_edge(
            input,
            fan_out_i,
            Edge {
                queue: queue.into(),
            },
        );

        fan_out_i
    }
}
