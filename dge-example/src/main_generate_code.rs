use dge_gen;

fn main() {
    let mut graph = dge_gen::Graph::new(
        "dge_example::behaviour::accept_failure::accept_failure",
        "dge_example::behaviour::error::Error",
    );
    let start = graph.start("start");
    let fan_out = graph.fan_out(
        start,
        "input",
        "dge_example::behaviour::data::Integer",
        "duplicate_input",
        10
    );
    let double = graph.process(
        fan_out,
        "input_copy_1".into(),
        "dge_example::behaviour::data::Integer",
        "double",
        "dge_example::behaviour::double".into(),
        11,
    );
    let square = graph.process(
        fan_out,
        "input_copy_2".into(),
        "dge_example::behaviour::data::Integer",
        "square",
        "dge_example::behaviour::square".into(),
        12,
    );
    let multiply = graph.aggregate(
        vec![double, square],
        "multiply".into(),
        "dge_example::behaviour::data::Integer",
        "multiply",
        "dge_example::behaviour::multiply".into(),
        13,
    );
    let rest_call = graph.poll(
        multiply,
        "rest_call".into(),
        "dge_example::behaviour::data::Float",
        "rest_call",
        "dge_example::behaviour::rest_call".into(),
        13,
    );

    let () = graph.terminate(
        rest_call,
        "result",
        "dge_example::behaviour::data::Integer",
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
            true,
            true,
        )
        .unwrap()
}
