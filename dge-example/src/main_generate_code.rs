use dge_gen;

fn main() {
    let mut graph = dge_gen::Graph::new(
        "dge_example::behaviour::accept_failure::accept_failure",
        "dge_example::behaviour::error::Error",
    );
    let start = graph.start("start");
    let fan_out = graph.fan_out(start, "input", "i32", "duplicate_input", 10);
    let add_1 = graph.process(
        fan_out,
        "input_copy_1".into(),
        "i32",
        "add_1",
        "dge_example::behaviour::add_1".into(),
        11,
    );
    let add_2 = graph.process(
        fan_out,
        "input_copy_2".into(),
        "i32",
        "add_2",
        "dge_example::behaviour::add_2".into(),
        12,
    );
    let wait_all = graph.aggregate(
        vec![add_1, add_2],
        "additions".into(),
        "i32",
        "wait_additions",
        "dge_example::behaviour::merge_additions::merge".into(),
        13,
    );

    let () = graph.terminate(
        wait_all,
        "some_output_queue",
        "String",
        "terminate",
        1
    );

    graph
        .generate(
            "dge-example/src/generated",
            "dge_example::behaviour::get_rmq_uri",
            "some_work_exchange",
            "some_retry_exchange",
            "pre_",
            "_post",
        )
        .unwrap()
}
