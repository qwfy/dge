#[macro_use]
mod helper_macro;

pub mod fan_out;
pub mod lib_rmq;
pub mod lib_rmq_primitive;
pub mod user_handler;
pub mod wait_all;

mod error;
pub use error::Error;
pub use error::Result;
