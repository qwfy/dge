use futures::Future;
use lapin::options::BasicQosOptions;
use lapin::Channel;
use lapin::Connection;
use lapin::ConnectionProperties;
use log::debug;
use log::info;
use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::fmt::Display;

use super::error::Error;
use super::error::Result;
use super::lib_rmq_primitive::constant::*;
use super::lib_rmq_primitive::create_channel;
use super::lib_rmq_primitive::unreliable_ack_or_reject;
use super::lib_rmq_primitive::AckType;
use super::lib_rmq_primitive::Responsibility;

/// Read a message of type `InputMsg` from `input_queue`, and process it with `handler`.
///
/// During the processing, the `handler` can access its state of type `HandlerState`,
/// and use the `Channel` to publish messages to other queues.
///
/// The `handler` can return `Ok(Responsibility::Accept)` to indicate that
/// the handler has taken care of the message been processed,
/// (i.e. the responsibility has transferred from RabbitMQ to the handler code),
/// this will cause an RabbitMQ ack to be send back to the RabbitMQ server,
/// rendering the consumption of the message.
///
/// The `handler` can also return `Ok<Responsibility::Reject>` to indicate that
/// the handler rejects the message,
/// in this case an RabbitMQ nack will be sent back to the RabbitMQ server,
/// and the message will be redelivered later.
///
/// For convenience, and `Err(_)` is treated largely in a similar way with a rejection,
/// the difference is that an `Ok<Responsibility::Reject>` is considered as an intentional rejection,
/// thus no warnings are logged, whereas an `Err(_)` is treated as an unintentional rejection,
/// which will cause warnings to be logged.
///
/// This function never terminates.
///
/// If the connection to the RabbitMQ server drops, it will be retried.
pub async fn consume_forever<InputMsg, HandlerState, HandlerResult>(
    input_queue: &'static str,
    handler: fn(HandlerState, Channel, InputMsg) -> HandlerResult,
    handler_state: HandlerState,
    prefetch_count: u16,
) where
    InputMsg: DeserializeOwned + Send + 'static,
    HandlerState: Clone + Send + 'static,
    HandlerResult: Future<Output = Result<Responsibility>> + Send + 'static,
{
    loop {
        // establish connection to rmq server and consume the queue
        match consume_queue(&input_queue, handler, handler_state.clone(), prefetch_count).await {
            Ok(()) => (),
            Err(e) => {
                warn!(
                    "error happened when consuming queue {}, will retry: {}",
                    &input_queue, e
                );
            }
        }

        // sleep for a while before reconnecting to avoid rapid fire
        let duration = std::time::Duration::from_millis(2000);
        info!(
            "sleep for {} seconds before reconnecting to queue {}",
            &duration.as_secs(),
            &input_queue
        );
        tokio::time::sleep(duration).await
    }
}

async fn consume_queue<InputMsg, HandlerState, HandlerResult>(
    input_queue: &'static str,
    handler: fn(HandlerState, Channel, InputMsg) -> HandlerResult,
    handler_state: HandlerState,
    prefetch_count: u16,
) -> Result<()>
where
    InputMsg: DeserializeOwned + Send + 'static,
    HandlerState: Clone + Send + 'static,
    HandlerResult: Future<Output = Result<Responsibility>> + Send + 'static,
{
    // establish communication
    info!("creating channel for consuming queue {}", input_queue);
    let channel = create_channel().await?;
    info!("setting prefetch to be {}", prefetch_count);
    channel
        .basic_qos(prefetch_count, BasicQosOptions { global: false })
        .await?;

    // create consumer
    // since each queue has exactly one consumer,
    // we can use the queue name as the identifier
    let consumer_tag = format!("dgec-{}", input_queue);
    info!("creating consumer {}", &consumer_tag);
    let consumer = channel
        .basic_consume(
            input_queue,
            &consumer_tag,
            RMQ_BASIC_CONSUME_OPTIONS,
            lapin::types::FieldTable::default(),
        )
        .await?;

    // consuming loop
    info!("entering consuming loop for queue {}", input_queue);
    for delivery in consumer {
        let (channel, msg) = delivery?;
        tokio::spawn(handle_one_delivery(
            channel,
            msg,
            handler,
            handler_state.clone(),
        ));
    }

    Ok(())
}

async fn handle_one_delivery<InputMsg, HandlerState, HandlerResult>(
    channel: Channel,
    delivery: lapin::message::Delivery,
    handle: fn(HandlerState, Channel, InputMsg) -> HandlerResult,
    handler_state: HandlerState,
) where
    InputMsg: DeserializeOwned + Send + 'static,
    HandlerState: Clone + Send + 'static,
    HandlerResult: Future<Output = Result<Responsibility>> + Send + 'static,
{
    debug!("processing message of tag: {}", delivery.delivery_tag);
    match serde_json::from_slice::<InputMsg>(&delivery.data) {
        Err(e) => {
            // json parse failed, we just warn and drop the message.
            // in theory, there are many ways to signal the error, either to the sender, or to a human,
            // but this shouldn't happen often, and probably due to a programming error made by a human,
            // so we just issue a warning
            warn!(
                "failed to parse json when processing delivery: {}, msg will be dropped, error is: {}, data is: {:?}",
                &delivery.delivery_tag, e, &delivery.data
            );
            unreliable_ack_or_reject(channel, AckType::Ack, delivery.delivery_tag).await
        }
        Ok(msg) => match handle(handler_state, channel.clone(), msg).await {
            Err(e) => {
                warn!(
                    "an error occurred while handling message {}, will requeue it, error is: {}",
                    &delivery.delivery_tag, e
                );
                unreliable_ack_or_reject(channel, AckType::Nack, delivery.delivery_tag).await
            }
            Ok(Responsibility::Reject) => {
                debug!("explicitly rejecting message {}", &delivery.delivery_tag);
                unreliable_ack_or_reject(channel, AckType::Nack, delivery.delivery_tag).await
            }

            Ok(Responsibility::Accept) => {
                debug!("accepting message {}", &delivery.delivery_tag);
                unreliable_ack_or_reject(channel, AckType::Ack, delivery.delivery_tag).await
            }
        },
    }
}
