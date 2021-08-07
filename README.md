# DGE: Distributed Graph Execution

## What's this?

Conceptually:

- It provides building blocks to build a computational graph,
  and generates the corresponding rust codes to execute the graph
  
- The nodes in the graph represent heterogeneous computations

- The edges represent data flow between nodes

- The computational graph is static, meaning it cannot be changed once defined
  
- The execution is distributed, meaning that the nodes can be executed on different machines

Concretely:

- You define the graph via crate `dge-gen` in a `.rs` file,
  compile & run it to generate the codes (including `main.rs`, depends on `dge-runtime`)
  that when compiled, can be used to execute the graph

- You compile the `main.rs` to an executable binary,
  each subcommand in this binary corresponds to a node in the graph.

- You are responsible to launch the binary with appropriate subcommands.
  (i.e. DGE does not handle the deployment and launching of the binary)

- You can launch arbitrary amounts (`>= 1`) of instances of the same subcommand,
  potentially on different machines
  
- Each edge is backed by exactly one RabbitMQ queue, the nodes are consumers of this queue

## An example

The following graph corresponds to the computation `(2 * x) * (x * x)`:

![](dge-example/src/generated/graph.svg)

It is generated with code in `dge-example/src/main_generate_code.rs`:

```rust
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

    // ... some code omitted for brevity ...

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
```

When the above code is compiled and run, a `main.rs` will be generated,
when the `main.rs` is compiled, you get an executable with these subcommands:

```shell
dge-example

USAGE:
    example <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    double                       # when executed, it creates an OS process that doubles the input
    duplicate-input              # send x to node double and square
    init-exchanges-and-queues    # initialize necessary RabbitMQ entities
    multiply                     # creates a node that does multiplication
    square                       # creates a node that calculates the square
```

These source files are generated for the above graph:

```
dge-example/src/generated/
├── double.rs
├── duplicate_input.rs
├── graph.dot
├── graph.svg
├── init_exchanges_and_queues.rs
├── main.rs
├── multiply.rs
└── square.rs
```