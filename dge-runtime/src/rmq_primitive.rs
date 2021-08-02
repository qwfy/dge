use lapin::options::BasicQosOptions;
use lapin::Channel;
use lapin::Connection;
use lapin::ConnectionProperties;
use log::debug;
use log::info;
use log::warn;
use serde::de::DeserializeOwned;
use std::future::Future;
use tokio_amqp::*;

use crate::Error;
use crate::Result;
use constant::*;

pub enum Responsibility {
    Accept,
    Reject,
}

pub enum AckType {
    Ack,
    Nack,
}

pub async fn unreliable_ack_or_reject(
    channel: Channel,
    ack_type: AckType,
    tag: lapin::types::LongLongUInt,
) {
    let (fut, action) = match ack_type {
        AckType::Ack => (channel.basic_ack(tag, RMQ_BASIC_ACK_OPTIONS), "acking"),
        AckType::Nack => (
            channel.basic_reject(tag, RMQ_BASIC_REJECT_DELAYED_REQUEUE_OPTIONS),
            "rejecting",
        ),
    };
    debug!("{} message {}", action, tag);
    match fut.await {
        Ok(()) => (),
        Err(e) => {
            // if we failed to ack, then we try to close the channel.
            // (a failed ack won't cause the message to be requeued
            // you need a connection close for that,
            // so we try close the channel),
            // if it closes, then the message will be requeued,
            // if it failed to close, then probably something is wrong with the connection,
            // hopefully this will cause the connection to be re-established
            // TODO @incomplete: handle the case where the close() failed,
            // but the connection is good. i.e. find a way to close the connection reliably
            warn!(
                "failed to {} message {}: {}, trying to close channel",
                action, tag, e
            );
            match channel
                .close(
                    lapin::protocol::constants::REPLY_SUCCESS as u16,
                    &format!("rejecting message {}", tag),
                )
                .await
            {
                Ok(()) => info!("channel closed"),
                // TODO @incomplete: how to force a close
                Err(e) => warn!(
                    "failed to close channel, message {} may be left as un-acked/un-nacked: {}",
                    tag, e
                ),
            }
        }
    }
}

pub async fn create_channel() -> Result<Channel> {
    info!("creating connection and channel channel");
    let conn = Connection::connect(
        "TODO @incomplete: rmq uri here",
        ConnectionProperties::default().with_tokio(),
    )
    .await?;

    let channel = conn.create_channel().await?;

    channel.confirm_select(RMQ_CONFIRM_SELECT_OPTIONS).await?;
    Ok(channel)
}

pub fn name_of_retry_queue(q: &str) -> String {
    format!("dge_retry_{}", q)
}

pub fn name_of_delay_queue(q: &str) -> String {
    format!("dge_delay_{}", q)
}

/// Publish the `msg` to `queue` at a later point in time.
pub async fn publish_delayed(channel: Option<Channel>, queue: &str, msg: Vec<u8>) -> Result<()> {
    let channel = match channel {
        None => create_channel().await?,
        Some(c) => c,
    };

    let delay_routing_key = name_of_delay_queue(queue);
    let confirm = channel
        .basic_publish(
            RMQ_DELAY_EXCHANGE,
            &delay_routing_key,
            RMQ_BASIC_PUBLISH_OPTIONS,
            msg,
            // 2 means durable
            lapin::BasicProperties::default().with_delivery_mode(2),
        )
        .await?
        .wait()?;

    if confirm.is_ack() {
        Ok(())
    } else {
        let error_msg = confirm
            .take_message()
            .map(|msg| String::from(msg.reply_text.as_str()))
            .unwrap_or(String::from("<no error message specified by RabbitMQ>"));
        Err(Error::FailedToPublishRmqMsg {
            queue: queue.into(),
            error: error_msg,
        })
    }
}

pub(crate) mod constant {
    use lapin::options::*;

    pub static RMQ_EXCHANGE: &str = "amq.direct";
    pub static RMQ_RETRY_EXCHANGE: &str = "dlx.retry";
    pub static RMQ_DELAY_EXCHANGE: &str = "delay";
    pub static RMQ_RETRY_INTERVAL_SECONDS: u32 = 60;
    pub static RMQ_DELAY_INTERVAL_SECONDS: u32 = 2;

    pub static RMQ_QUEUE_DECLARE_OPTIONS: QueueDeclareOptions = QueueDeclareOptions {
        passive: false, // create if not exist
        durable: true,
        exclusive: false,
        auto_delete: false,
        nowait: false, // wait for declare-ok
    };

    pub static RMQ_BASIC_CONSUME_OPTIONS: BasicConsumeOptions = BasicConsumeOptions {
        no_local: false,
        no_ack: false, // use explict ack
        exclusive: false,
        nowait: false,
    };

    pub static RMQ_QUEUE_BIND_OPTIONS: QueueBindOptions = QueueBindOptions { nowait: false };

    pub static RMQ_BASIC_ACK_OPTIONS: BasicAckOptions = BasicAckOptions { multiple: false };

    pub static RMQ_BASIC_REJECT_DELAYED_REQUEUE_OPTIONS: BasicRejectOptions =
        BasicRejectOptions { requeue: false };

    pub static RMQ_BASIC_PUBLISH_OPTIONS: BasicPublishOptions = BasicPublishOptions {
        mandatory: true,  // require that the msg should be routed
        immediate: false, // false to allow the message to be queued
    };

    pub static RMQ_CONFIRM_SELECT_OPTIONS: ConfirmSelectOptions =
        ConfirmSelectOptions { nowait: false };
}
