use log::{debug, info, warn};
use petgraph::Direction;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use crate::graph::Edge;
use crate::graph::EdgeIndex;
use crate::graph::Graph;
use crate::graph::Node;
use crate::graph::NodeIndex;
use crate::graph::PetGraph;
use crate::Error;
use crate::Result;

/// Generate codes corresponding the graph.
pub(crate) fn generate<P: AsRef<Path>>(graph: Graph, dir: P) -> Result<()> {
    let g = &graph.g;
    let mut outputs = HashMap::new();
    let dir = dir.as_ref();

    // TODO @incomplete: check the validity of the graph
    // - no cycle
    // - every node has exactly one input queue
    // - every queue has exactly one consumer
    // - ?

    // generate code for each node
    for node_i in g.node_indices() {
        let node = &g[node_i];
        match node {
            // start node doesn't need any code
            Node::Start { .. } => (),
            Node::Aggregate { name, aggregate } => {
                let content = generate_aggregate(
                    g,
                    node_i,
                    aggregate.into(),
                    graph.accept_failure.clone(),
                    graph.type_error.clone(),
                )?;
                update_outputs(&mut outputs, dir, name, content);
            }
            Node::FanOut { name } => {
                let content = generate_fan_out(g, node_i, graph.accept_failure.clone())?;
                update_outputs(&mut outputs, dir, name, content);
            }
            Node::UserHandler {
                name,
                behaviour_module,
            } => {
                let content = generate_user_handler(
                    g,
                    node_i,
                    behaviour_module.into(),
                    graph.accept_failure.clone(),
                )?;
                update_outputs(&mut outputs, dir, name, content);
            }
        }
    }

    // write the graph in dot format
    let graph_for_display = map_to_string(g);
    let dot = petgraph::dot::Dot::with_config(&graph_for_display, &[]);
    let dot_file_path = dir.join("graph.dot");
    info!("writing dot graph to {}", &dot_file_path.display());
    std::fs::write(&dot_file_path, format!("{}", dot))?;
    let f = std::fs::OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .open(&dot_file_path)?
        .sync_all()?;

    // convert dot to svg
    let svg_file_path = dir.join("graph.svg");
    let gen_svg_output = std::process::Command::new("dot")
        .arg("-Tsvg")
        .arg(
            dot_file_path
                .to_str()
                .ok_or(Error::InvalidFileName(dot_file_path.display().to_string()))?,
        )
        .arg("-o")
        .arg(
            svg_file_path
                .to_str()
                .ok_or(Error::InvalidFileName(svg_file_path.display().to_string()))?,
        )
        .output()?;
    if gen_svg_output.status.success() {
        ()
    } else {
        return Err(Error::ErrorGeneratingSvg);
    }

    let mut mods = Vec::new();
    // write each file
    for (file_path, content) in outputs {
        info!("writing to {}", &file_path.display());
        std::fs::write(&file_path, content)?;

        match &file_path.file_stem() {
            None => return Err(Error::InvalidFileName((&file_path).display().to_string())),
            Some(file_stem) => match (*file_stem).to_str() {
                None => return Err(Error::InvalidFileName((&file_path).display().to_string())),
                Some(s) => mods.push(format!("pub(crate) mod {};", s)),
            },
        }
    }

    // write mod.rs
    let mods: String = mods.join("\n");
    std::fs::write(dir.join("mod.rs"), mods)?;

    Ok(())
}

fn generate_aggregate(
    g: &PetGraph,
    node_i: NodeIndex,
    aggregate: String,
    accept_failure: String,
    type_error: String,
) -> Result<String> {
    let Edge {
        queue: input_queue,
        msg_type: type_input,
    } = expect_one_input_edge_for_aggregation_node(g, node_i)?;
    let output_queue = expect_optional_outgoing_edge(g, node_i)?.map(|e| e.queue.clone());
    super::aggregate::generate(
        input_queue,
        aggregate,
        output_queue,
        accept_failure,
        type_input,
        type_error,
    )
}

fn generate_fan_out(g: &PetGraph, node_i: NodeIndex, accept_failure: String) -> Result<String> {
    let Edge {
        queue: input_queue,
        msg_type: type_input,
    } = expect_one_incoming_edge(g, node_i)?;

    // find output queues
    // TODO @incomplete: check for duplicate queues
    let mut output_queues = Vec::new();
    let out_edges = g.edges_directed(node_i, Direction::Outgoing);
    for out_edge in out_edges {
        let output_queue = out_edge.weight().queue.clone();
        output_queues.push(output_queue)
    }

    super::fan_out::generate(
        input_queue.clone(),
        output_queues,
        accept_failure,
        type_input.clone(),
    )
}

fn generate_user_handler(
    g: &PetGraph,
    node_i: NodeIndex,
    module: String,
    accept_failure: String,
) -> Result<String> {
    let Edge {
        queue: input_queue,
        msg_type: type_input,
    } = expect_one_incoming_edge(g, node_i)?;
    let output_queue = expect_optional_outgoing_edge(g, node_i)?.map(|e| e.queue.clone());
    super::user_handler::generate(
        input_queue.clone(),
        output_queue,
        module,
        accept_failure,
        type_input.clone(),
    )
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

fn expect_one_input_edge_for_aggregation_node(g: &PetGraph, node_i: NodeIndex) -> Result<Edge> {
    let incoming_edges = g.edges_directed(node_i, Direction::Incoming);
    let mut edges = HashSet::new();
    for edge in incoming_edges {
        edges.insert(edge.weight().clone());
    }
    let edges: Vec<_> = edges.into_iter().collect();
    if edges.len() == 1 {
        Ok(edges[0].clone())
    } else {
        Err(Error::IllFormedNode {
            node: format!("{:?}", g[node_i]),
        })
    }
}

fn update_outputs<P: AsRef<Path>, S: AsRef<str>>(
    outputs: &mut HashMap<PathBuf, String>,
    dir: P,
    name: S,
    content: String,
) {
    let dir = dir.as_ref();
    let basename = format!("{}.rs", name.as_ref());
    let file_path = dir.join(basename);
    outputs.insert(file_path, content);
}

fn map_to_string(old: &PetGraph) -> petgraph::Graph<String, String> {
    old.map(
        |node_index, node| node.name(),
        |edge_index, edge| edge.queue.clone(),
    )
}
