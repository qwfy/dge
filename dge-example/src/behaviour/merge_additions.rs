use super::error::Error;
use dge_runtime::component::aggregate::AggregationStatus;

pub async fn merge(input: &i32) -> Result<AggregationStatus<String>, Error> {
    println!("merged");
    Ok(AggregationStatus::FreshAggregation(String::from(
        "some value",
    )))
}
