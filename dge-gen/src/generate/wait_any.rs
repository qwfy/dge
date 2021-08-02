use askama::Template;

use super::rust::gen_opt_string;
use super::rust::gen_string;
use super::rust::gen_u32;
use super::rust::gen_vec_string;
use crate::Error;
use crate::Result;

pub(crate) fn generate(
    input_queue: String,
    merge_messages: String,
    output_queue: Option<String>,
) -> Result<String> {
    unimplemented!()
}
