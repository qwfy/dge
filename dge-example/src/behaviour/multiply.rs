use super::error::Error;
use dge_runtime::component::aggregate::AggregationStatus;

pub async fn multiply(input: &i32) -> Result<AggregationStatus<f32>, Error> {
    Ok(AggregationStatus::FreshAggregation(1.0))
}
