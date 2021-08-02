use petgraph::visit::IntoEdgesDirected;
use petgraph::Direction;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::error::Error;
use crate::error::Result;
use crate::generate;
use crate::graph::Edge;
use crate::graph::EdgeIndex;
use crate::graph::Graph;
use crate::graph::Node;
use crate::graph::NodeIndex;
use crate::graph::PetGraph;

/// Generate codes corresponding the graph.
pub fn generate_code(graph: &Graph) -> Result<()> {
    let g = &graph.g;
    let mut outputs = HashMap::new();

    // TODO @incomplete: check the validity of the graph
    // - single start node
    // - no cycle
    // - every node has exactly one input queue
    // - every queue has exactly one consumer
    // - ?

    // generate code for each node
    for node_i in g.node_indices() {
        let node = &g[node_i];
        match node {
            // start node doesn't need any code
            Node::Start => (),
            Node::WaitAll { merge_messages } => generate_wait_all(
                &mut outputs,
                g,
                node_i,
                merge_messages.into(),
                graph.accept_failure.clone(),
            )?,
            Node::WaitAny { merge_messages } => {
                generate_wait_any(&mut outputs, g, node_i, merge_messages.into())?
            }
            Node::FanOut => {
                generate_fan_out(&mut outputs, g, node_i, graph.accept_failure.clone())?
            }
            Node::UserHandler { module } => {
                generate_user_handler(&mut outputs, g, node_i, module.into())?
            }
        }
    }

    Ok(())
}

fn generate_wait_all(
    outputs: &mut HashMap<String, String>,
    g: &PetGraph,
    node_i: NodeIndex,
    merge_messages: String,
    accept_failure: String,
) -> Result<()> {
    let input_queue = expect_one_input_queue_for_aggregation_node(g, node_i)?;
    let output_queue = expect_optional_outgoing_edge(g, node_i)?.map(|e| e.queue.clone());
    generate::wait_all::generate(
        outputs,
        input_queue,
        merge_messages,
        output_queue,
        accept_failure,
    )
}

fn generate_wait_any(
    outputs: &mut HashMap<String, String>,
    g: &PetGraph,
    node_i: NodeIndex,
    merge_messages: String,
) -> Result<()> {
    let input_queue = expect_one_input_queue_for_aggregation_node(g, node_i)?;
    let output_queue = expect_optional_outgoing_edge(g, node_i)?.map(|e| e.queue.clone());
    generate::wait_any::generate(outputs, input_queue, merge_messages, output_queue)
}

fn generate_fan_out(
    outputs: &mut HashMap<String, String>,
    g: &PetGraph,
    node_i: NodeIndex,
    accept_failure: String,
) -> Result<()> {
    let Edge { queue: input_queue } = expect_one_incoming_edge(g, node_i)?;

    // find output queues
    // TODO @incomplete: check for duplicate queues
    let mut output_queues = Vec::new();
    let out_edges = g.edges_directed(node_i, Direction::Outgoing);
    for out_edge in out_edges {
        let output_queue = out_edge.weight().queue.clone();
        output_queues.push(output_queue)
    }

    generate::fan_out::generate(outputs, input_queue.clone(), output_queues, accept_failure)
}

fn generate_user_handler(
    outputs: &mut HashMap<String, String>,
    g: &PetGraph,
    node_i: NodeIndex,
    module: String,
) -> Result<()> {
    let Edge { queue: input_queue } = expect_one_incoming_edge(g, node_i)?;
    let output_queue = expect_optional_outgoing_edge(g, node_i)?.map(|e| e.queue.clone());
    generate::user_handler::generate(outputs, input_queue.clone(), output_queue)
}

fn expect_one_incoming_edge(g: &PetGraph, node_i: NodeIndex) -> Result<&Edge> {
    let in_edges: Vec<_> = g.edges_directed(node_i, Direction::Incoming).collect();
    if in_edges.len() != 1 {
        Err(Error::IllFormedNode {
            node: format!("{:?}", g[node_i]),
        })
    } else {
        Ok(in_edges[0].weight())
    }
}

fn expect_optional_outgoing_edge(g: &PetGraph, i: NodeIndex) -> Result<Option<&Edge>> {
    let edges: Vec<_> = g.edges_directed(i, Direction::Outgoing).collect();
    if edges.len() == 0 {
        Ok(None)
    } else if edges.len() == 1 {
        Ok(Some(edges[0].weight()))
    } else {
        Err(Error::IllFormedNode {
            node: format!("{:?}", g[i]),
        })
    }
}

fn expect_one_input_queue_for_aggregation_node(g: &PetGraph, node_i: NodeIndex) -> Result<String> {
    let incoming_edges = g.edges_directed(node_i, Direction::Incoming);
    let mut queues = HashSet::new();
    for edge in incoming_edges {
        let queue = edge.weight().queue.clone();
        queues.insert(queue);
    }
    let queues: Vec<_> = queues.into_iter().collect();
    if queues.len() == 1 {
        Ok(queues[0].clone())
    } else {
        Err(Error::IllFormedNode {
            node: format!("{:?}", g[node_i]),
        })
    }
}
