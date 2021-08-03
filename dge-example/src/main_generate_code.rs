use dge_gen;

fn main() {
    let mut graph = dge_gen::Graph::new(
        "crate::behaviour::accept_failure::accept_failure",
        "crate::behaviour::error::Error",
    );
    let start = graph.start("start_node");
    let fan_out = graph.fan_out("duplicate_input_msg", start, "input_msg", "i32");
    let add_1 = graph.process(
        "add_1",
        fan_out,
        "input_msg_copy_1".into(),
        "crate::behaviour::add_1".into(),
        "i32",
    );
    let add_2 = graph.process(
        "add_2",
        fan_out,
        "input_msg_copy_2".into(),
        "crate::behaviour::add_2".into(),
        "i32",
    );
    let wait_all = graph.aggregate(
        "wait_additions",
        vec![add_1, add_2],
        "additions".into(),
        "crate::behaviour::merge_additions::merge".into(),
        "i32",
    );

    graph.generate("src/generated").unwrap()
}
