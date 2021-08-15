pub mod accept_failure;
pub mod double;
pub mod square;
pub mod error;
pub mod multiply;
pub mod data;
pub mod rest_call;

pub fn get_rmq_uri() -> String {
    String::from("amqp://rmq-user:rmq-password@192.168.3.68:5672/alpha-server")
}
