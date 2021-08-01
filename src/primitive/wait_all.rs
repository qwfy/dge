use crate::error::Error;
use std::collections::HashMap;

pub(crate) fn generate(
    outputs: &mut HashMap<String, String>,
    input_queue: String,
    merge_messages: String,
    output_queue: Option<String>,
) -> Result<(), Error> {
    unimplemented!()
}
