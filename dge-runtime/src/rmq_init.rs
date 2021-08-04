use lapin;
use lapin::types::AMQPValue;
use lapin::types::FieldTable;
use lapin::types::LongString;
use lapin::types::ShortString;
use log::info;

use crate::Error;
use crate::Result;

use crate::rmq_primitive;
use crate::rmq_primitive::constant::RMQ_QUEUE_BIND_OPTIONS;
use crate::rmq_primitive::constant::RMQ_QUEUE_DECLARE_OPTIONS;

/// Create the necessary RabbitMQ exchanges and queues.
///
/// For a message published to `work_queue` of direct exchange `work_direct_exchange`:
///
/// - if the consumer of `work_queue` `ack`s it, then this is the end of this message
/// - if the consumer `nack`ed it, then it will be dead-lettered  to `retry_queue` of
///   direct exchange `retry_direct_exchange`, after `retry_interval_in_seconds`,
///   the message will be delivered to `work_queue` again, the cycle continues
///
/// In short, the user should only be interested in `work_queue`, the `retry_queue` is
/// is just there to enable retry with configurable delay.
///
/// (`work_direct_exchange`, `work_queue`) and (`retry_direct_exchange`, `retry_queue`)
/// mutually dead-letters each other, with different ways of triggering it.
///
/// Both `work_direct_exchange` and `retry_direct_exchange` should be direct exchanges,
/// the implementation depends on this.
pub async fn init_with_retry<S: AsRef<str>>(
    work_direct_exchange: S,
    work_queue: S,
    retry_direct_exchange: S,
    retry_queue: S,
    retry_interval_in_seconds: u32,
) -> Result<()> {
    let work_direct_exchange = work_direct_exchange.as_ref();
    let retry_direct_exchange = retry_direct_exchange.as_ref();
    let work_queue = work_queue.as_ref();
    let retry_queue = retry_queue.as_ref();

    let channel = rmq_primitive::create_channel().await?;

    let exchanges: [&str; 2] = [work_direct_exchange, retry_direct_exchange];
    for exchange in exchanges.into_iter() {
        info!("declaring exchange {}", exchange);
        channel
            .exchange_declare(
                exchange,
                lapin::ExchangeKind::Direct,
                lapin::options::ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
    }

    info!("declaring work queue {}", work_queue);
    let mut queue_args = FieldTable::default();
    queue_args.insert(
        ShortString::from("x-dead-letter-exchange"),
        AMQPValue::from(LongString::from(retry_direct_exchange)),
    );
    queue_args.insert(
        ShortString::from("x-dead-letter-routing-key"),
        AMQPValue::from(LongString::from(retry_queue)),
    );
    channel
        .queue_declare(work_queue, RMQ_QUEUE_DECLARE_OPTIONS, queue_args)
        .await?;

    info!(
        "binding work queue {} to exchange {}",
        work_queue, work_direct_exchange
    );
    channel
        .queue_bind(
            work_queue,
            work_direct_exchange,
            work_queue,
            RMQ_QUEUE_BIND_OPTIONS,
            FieldTable::default(),
        )
        .await?;

    info!(
        "declaring retry queue {} for work queue {}",
        retry_queue, work_queue
    );
    let mut queue_args = FieldTable::default();
    queue_args.insert(
        ShortString::from("x-dead-letter-exchange"),
        AMQPValue::from(LongString::from(work_direct_exchange)),
    );
    queue_args.insert(
        ShortString::from("x-dead-letter-routing-key"),
        AMQPValue::from(LongString::from(work_queue)),
    );
    queue_args.insert(
        ShortString::from("x-message-ttl"),
        AMQPValue::from(retry_interval_in_seconds * 1000),
    );
    channel
        .queue_declare(retry_queue, RMQ_QUEUE_DECLARE_OPTIONS, queue_args)
        .await?;

    info!(
        "binding retry queue {} to exchange {}",
        retry_queue, retry_direct_exchange
    );
    channel
        .queue_bind(
            retry_queue,
            retry_direct_exchange,
            retry_queue,
            RMQ_QUEUE_BIND_OPTIONS,
            FieldTable::default(),
        )
        .await?;

    channel
        .close(
            lapin::protocol::constants::REPLY_SUCCESS as lapin::types::ShortUInt,
            "normal close of channel",
        )
        .await?;

    info!(
        "done creating exchanges and queues for work queue {}",
        work_queue
    );

    Ok(())
}
