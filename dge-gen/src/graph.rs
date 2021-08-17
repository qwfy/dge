use petgraph;
use std::path::Path;

pub use petgraph::graph::EdgeIndex;
pub use petgraph::graph::NodeIndex;

use super::generate;

pub(crate) type PetGraph = petgraph::Graph<Node, Edge>;

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
    /// A terminating node the indicates the termination of the computation.
    Terminate {
        name: String,
    },
    /// An aggregation node that consume one or more messages,
    /// and outputs zero or more messages.
    Aggregate {
        name: String,
        behaviour_module: String,
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
    /// A node that polls the incoming messages.
    Poll {
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
            Node::Poll { name, .. } => name.clone(),
        }
    }
}

/// An edge represents a RabbitMQ queue carrying a specific type of message.
#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub(crate) struct Edge {
    pub(crate) queue: String,
    pub(crate) msg_type: String,
    pub(crate) retry_interval_in_seconds: u32,
}

/// A computational graph
///
/// - edges of the graph represent messages of static types delivered between nodes via RabbitMQ queue
/// - nodes of the graph represent computations that transform the input message into output message
pub struct Graph {
    pub(crate) g: PetGraph,
    pub(crate) accept_failure: String,
    pub(crate) type_error: String,
}

impl Graph {
    /// Create a new computational graph.
    ///
    /// - `accept_failure : fn(Context, Error) -> Result<(), Error>`
    /// - `type_error: Error`
    ///
    /// where:
    ///
    /// - `Error` is your global error type
    ///   (dge assumes that you use an global error type for your application)
    /// - `Context` is a data type representing the current context,
    ///   this data type is defined by you,
    ///   and should be `From<T>` for all of your messages `T` carried by an edge
    ///
    /// Different edges can carry different types of messages,
    /// but they all have to be serializable and deserializable by serde_json,
    /// this serialization requirement is strictly for convenience,
    /// and can be removed if there is enough motivation.
    pub fn new<S: Into<String>>(accept_failure: S, type_error: S) -> Graph {
        Graph {
            g: petgraph::Graph::new(),
            accept_failure: accept_failure.into(),
            type_error: type_error.into(),
        }
    }

    /// Represent the start of the computation.
    ///
    /// Usually this is the first function been called to acquire a starting point
    /// for later operations after the graph is created.
    ///
    /// Return a handle to the start node.
    pub fn start<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        self.g.add_node(Node::Start { name: name.into() })
    }

    /// Read a message of type `type_input` from node `input` via RabbitMQ queue `queue`,
    /// and process it with this node, which is named `name`,
    /// using behaviour defined by `behaviour_module`.
    /// Retry after `retry_interval_in_seconds` if some error happened during the processing,
    /// (transient or non-transient), and dge decides that the processing should be retried.
    ///
    /// Internally this will create a new node for executing code defined by `behaviour_module`,
    /// and a new edge from `input` to node `name` representing the underlying RabbitMQ queue `queue`.
    ///
    /// The arguments can be read as:
    ///
    /// `input -- queue carrying message of type_input --> name`
    pub fn process<S: Into<String>>(
        &mut self,
        input: NodeIndex,
        queue: S,
        type_input: S,
        name: S,
        behaviour_module: S,
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

    /// Add a node that aggregates messages from `inputs` that belong to a single run,
    /// and aggregate them for later consumption.
    ///
    /// `behaviour_module` defines how the input messages should be aggregated.
    pub fn aggregate<S: Into<String>>(
        &mut self,
        inputs: Vec<NodeIndex>,
        queue: S,
        type_input: S,
        name: S,
        behaviour_module: S,
        retry_interval_in_seconds: u32,
    ) -> NodeIndex {
        let type_input = type_input.into();
        let wait_node_i = self.g.add_node(Node::Aggregate {
            name: name.into(),
            behaviour_module: behaviour_module.into(),
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

    /// Create a node that will copy messages of `input`
    /// to all outgoing edges of the newly created node.
    pub fn fan_out<S: Into<String>>(
        &mut self,
        input: NodeIndex,
        queue: S,
        type_input: S,
        name: S,
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

    /// Add a node that polls some external system using the input message as the argument.
    ///
    /// `behaviour_module` defines the function that will perform the polling,
    /// this nodes provides scheduling for the actual polling function.
    ///
    /// For example this can be used to query an third-party REST service for the availability
    /// of the resources corresponding the input messages.
    pub fn poll<S: Into<String>>(
        &mut self,
        input: NodeIndex,
        queue: S,
        type_input: S,
        name: S,
        behaviour_module: S,
        retry_interval_in_seconds: u32,
    ) -> NodeIndex {
        let poll_node = Node::Poll {
            name: name.into(),
            behaviour_module: behaviour_module.into(),
        };
        let poll_node_i = self.g.add_node(poll_node);
        let edge = Edge {
            queue: queue.into(),
            msg_type: type_input.into(),
            retry_interval_in_seconds,
        };
        self.g.add_edge(input, poll_node_i, edge);
        poll_node_i
    }

    /// A no-op node that terminates the computation.
    pub fn terminate<S: Into<String>>(&mut self, input: NodeIndex, queue: S, type_input: S, name: S, retry_interval_in_seconds: u32) -> () {
        let terminate_node = self.g.add_node(Node::Terminate { name: name.into() });
        self.g.add_edge(input, terminate_node, Edge {
            queue: queue.into(),
            msg_type: type_input.into(),
            retry_interval_in_seconds
        });
    }


    /// Generate code represented by the graph.
    ///
    /// - the generated code will be written to `output_dir`
    /// - `get_rmq_uri` is used to get an url to connect to the RabbitMQ server
    /// - `work_exchange` and `retry_exchange` are direct RabbitMQ exchanges,
    ///   for every edge in the graph, there will be a queue bound to `work_exchange`
    ///   to deliver the message, and a retry queue bound to `retry_exchange` to handle the retry,
    ///   (the retry is backed by RabbitMQ's dead lettering mechanism)
    /// - if the `init_input_queue` is true, then queues originated from start nodes
    ///   are also declared by the `init-exchanges-and-queues` subcommand.
    /// - `init_output_queue` controls the initialization of queues leading to termination nodes
    /// - `main_init` is a function that will be run prior to the start of the computation,
    ///   this can be used for things like setting up the logger
    pub fn generate<P: AsRef<Path>, S: AsRef<str>>(
        self,
        output_dir: P,
        get_rmq_uri: S,
        work_exchange: S,
        retry_exchange: S,
        retry_queue_prefix: S,
        retry_queue_suffix: S,
        init_input_queue: bool,
        init_output_queue: bool,
        main_init: S,
    ) -> Result<()> {
        let rmq_options = generate::graph::RmqOptions {
            get_rmq_uri: get_rmq_uri.as_ref().into(),
            work_exchange: work_exchange.as_ref().into(),
            retry_exchange: retry_exchange.as_ref().into(),
            retry_queue_prefix: retry_queue_prefix.as_ref().into(),
            retry_queue_suffix: retry_queue_suffix.as_ref().into(),
        };
        generate::graph::generate(self, output_dir, rmq_options, init_input_queue, init_output_queue, main_init)
    }
}
