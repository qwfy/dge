use super::error::Error;
use dge_runtime::component::wait_all::MergeStatus;

pub(crate) async fn merge_messages(input: i32) -> Result<MergeStatus<String>, Error> {
    Ok(MergeStatus::FreshMerge(String::from("some value")))
}
