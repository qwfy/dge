pub mod accept_failure;
pub mod add_1;
pub mod add_2;
pub mod error;
pub mod merge_additions;

pub fn get_rmq_uri() -> String {
    String::from("amqp://rmq-user:rmq-password@192.168.3.68:5672/alpha-server")
}
