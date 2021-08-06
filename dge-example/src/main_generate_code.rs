use dge_gen;

fn main() {
    let mut graph = dge_gen::Graph::new(
        "dge_example::behaviour::accept_failure::accept_failure",
        "dge_example::behaviour::error::Error",
    );
    let start = graph.start("start");
    let fan_out = graph.fan_out(start, "input", "i32", "duplicate_input", 10);
    let double = graph.process(
        fan_out,
        "input_copy_1".into(),
        "i32",
        "double",
        "dge_example::behaviour::double".into(),
        11,
    );
    let square = graph.process(
        fan_out,
        "input_copy_2".into(),
        "i32",
        "square",
        "dge_example::behaviour::square".into(),
        12,
    );
    let multiply = graph.aggregate(
        vec![double, square],
        "multiply".into(),
        "i32",
        "multiply",
        "dge_example::behaviour::multiply::multiply".into(),
        13,
    );

    let () = graph.terminate(
        multiply,
        "result",
        "f32",
        "terminate",
        1
    );

    graph
        .generate(
            "dge-example/src/generated",
            "dge_example::behaviour::get_rmq_uri",
            "dge_example_work_exchange",
            "dge_example_retry_exchange",
            "retry_",
            "",
        )
        .unwrap()
}
