use super::error::Error;
use dge_runtime::component::aggregate::MergeStatus;

pub(crate) async fn merge(input: &i32) -> Result<MergeStatus<String>, Error> {
    Ok(MergeStatus::FreshMerge(String::from("some value")))
}
