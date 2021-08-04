pub(crate) mod accept_failure;
pub(crate) mod add_1;
pub(crate) mod add_2;
pub(crate) mod error;
pub(crate) mod merge_additions;

pub(crate) fn get_rmq_uri() -> String {
    String::from("rmq://example.com/example-host")
}
