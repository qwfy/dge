use std::collections::HashMap;

use crate::error::Error;
use crate::error::Result;

pub(crate) fn generate(
    outputs: &mut HashMap<String, String>,
    input_queue: String,
    merge_messages: String,
    output_queue: Option<String>,
) -> Result<()> {
    unimplemented!()
}
