use lapin::Channel;
use lapin::Connection;
use lapin::ConnectionProperties;
use log::debug;
use log::info;
use log::warn;
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
    Reject,
}

pub async fn unreliable_ack_or_reject(
    channel: Channel,
    ack_type: AckType,
    tag: lapin::types::LongLongUInt,
) {
    let (fut, action) = match ack_type {
        AckType::Ack => (channel.basic_ack(tag, RMQ_BASIC_ACK_OPTIONS), "acking"),
        AckType::Reject => (
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
                    "failed to close channel, message {} may be left as un-acked/un-rejected: {}",
                    tag, e
                ),
            }
        }
    }
}

pub async fn create_channel<S: AsRef<str>>(rmq_uri: S) -> Result<Channel> {
    let rmq_uri = rmq_uri.as_ref();
    info!("creating connection and channel channel");
    let conn = Connection::connect(rmq_uri, ConnectionProperties::default().with_tokio()).await?;

    let channel = conn.create_channel().await?;

    channel.confirm_select(RMQ_CONFIRM_SELECT_OPTIONS).await?;
    Ok(channel)
}

pub fn name_of_retry_queue(q: &str) -> String {
    format!("dge_retry_{}", q)
}

/// Publish the `msg` to `queue`.
pub async fn publish<S: AsRef<str>>(
    channel: Channel,
    exchange: S,
    queue: S,
    msg: Vec<u8>,
) -> Result<()> {
    let exchange = exchange.as_ref();
    let queue = queue.as_ref();

    let confirm = channel
        .basic_publish(
            exchange,
            &queue,
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
