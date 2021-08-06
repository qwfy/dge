use petgraph;
use std::path::Path;

pub use petgraph::graph::EdgeIndex;
pub use petgraph::graph::NodeIndex;

use super::generate;

pub(crate) type PetGraph = petgraph::Graph<Node, Edge>;

use crate::Error;
use crate::Result;

/// A node represents the computation.
///
/// Every node has an unique name associated with it,
/// which will be used as the file name of the generated file.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    /// A no-op node that indicates the start of the computation.
    Start {
        name: String,
    },
    Terminate {
        name: String,
    },
    Aggregate {
        name: String,
        aggregate: String,
    },
    /// Duplicate the output of one node to multiple nodes.
    FanOut {
        name: String,
    },
    /// A user-provided handler that transform the input message into the output message.
    UserHandler {
        name: String,
        behaviour_module: String,
    },
}

impl Node {
    pub(crate) fn name(&self) -> String {
        match self {
            Node::Start { name, .. } => name.clone(),
            Node::Terminate { name, .. } => name.clone(),
            Node::Aggregate { name, .. } => name.clone(),
            Node::FanOut { name, .. } => name.clone(),
            Node::UserHandler { name, .. } => name.clone(),
        }
    }
}

/// An edge represents a RabbitMQ queue.
#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub(crate) struct Edge {
    pub(crate) queue: String,
    pub(crate) msg_type: String,
    pub(crate) retry_interval_in_seconds: u32,
}

/// A computational graph where:
///
/// - edges represent messages delivered via RabbitMQ queue
/// - nodes represent computations that transform the input message into output message
pub struct Graph {
    pub(crate) g: PetGraph,
    pub(crate) accept_failure: String,
    pub(crate) type_error: String,
}

impl Graph {
    /// Create a new computational graph.
    pub fn new<S: Into<String>>(accept_failure: S, type_error: S) -> Graph {
        Graph {
            g: petgraph::Graph::new(),
            accept_failure: accept_failure.into(),
            type_error: type_error.into(),
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

    pub fn terminate<S: Into<String>>(&mut self, name: S, input: NodeIndex, queue: S, type_input: S, retry_interval_in_seconds: u32) -> () {
        let terminate_node = self.g.add_node(Node::Terminate { name: name.into() });
        self.g.add_edge(input, terminate_node, Edge {
            queue: queue.into(),
            msg_type: type_input.into(),
            retry_interval_in_seconds
        });
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
        type_input: S,
        retry_interval_in_seconds: u32,
    ) -> NodeIndex {
        let handler_node = Node::UserHandler {
            name: name.into(),
            behaviour_module: behaviour_module.into(),
        };
        let handler_node_i = self.g.add_node(handler_node);
        let edge = Edge {
            queue: queue.into(),
            msg_type: type_input.into(),
            retry_interval_in_seconds,
        };
        self.g.add_edge(input, handler_node_i, edge);
        handler_node_i
    }

    /// Add a node that aggregate messages from `inputs` that belong to a single run,
    /// and aggregate them for later consumption.
    ///
    /// Return a handle to the newly added node.
    pub fn aggregate<S: Into<String>>(
        &mut self,
        name: S,
        inputs: Vec<NodeIndex>,
        queue: S,
        aggregate: S,
        type_input: S,
        retry_interval_in_seconds: u32,
    ) -> NodeIndex {
        let type_input = type_input.into();
        let wait_node_i = self.g.add_node(Node::Aggregate {
            name: name.into(),
            aggregate: aggregate.into(),
        });
        let queue = queue.into();
        for input_i in inputs {
            self.g.add_edge(
                input_i,
                wait_node_i,
                Edge {
                    queue: queue.clone(),
                    msg_type: type_input.clone(),
                    retry_interval_in_seconds,
                },
            );
        }
        wait_node_i
    }

    /// Create a node that will copy messages of `input` delivered via `queue`
    /// to all outgoing edges of the newly created node.
    ///
    /// Return a handle to the newly created node
    pub fn fan_out<S: Into<String>>(
        &mut self,
        name: S,
        input: NodeIndex,
        queue: S,
        type_input: S,
        retry_interval_in_seconds: u32,
    ) -> NodeIndex {
        let fan_out_i = self.g.add_node(Node::FanOut { name: name.into() });
        self.g.add_edge(
            input,
            fan_out_i,
            Edge {
                queue: queue.into(),
                msg_type: type_input.into(),
                retry_interval_in_seconds,
            },
        );

        fan_out_i
    }

    /// Generate code represented by the graph, write the code generated to `output_dir`.
    pub fn generate<P: AsRef<Path>, S: AsRef<str>>(
        self,
        output_dir: P,
        get_rmq_uri: S,
        work_exchange: S,
        retry_exchange: S,
        retry_queue_prefix: S,
        retry_queue_suffix: S,
    ) -> Result<()> {
        let rmq_options = generate::graph::RmqOptions {
            get_rmq_uri: get_rmq_uri.as_ref().into(),
            work_exchange: work_exchange.as_ref().into(),
            retry_exchange: retry_exchange.as_ref().into(),
            retry_queue_prefix: retry_queue_prefix.as_ref().into(),
            retry_queue_suffix: retry_queue_suffix.as_ref().into(),
        };
        generate::graph::generate(self, output_dir, rmq_options)
    }
}
