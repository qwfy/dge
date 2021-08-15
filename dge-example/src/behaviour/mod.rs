pub mod accept_failure;
pub mod double;
pub mod square;
pub mod error;
pub mod multiply;
pub mod data;
pub mod rest_call;

pub fn get_rmq_uri() -> String {
    String::from("amqp://rmq-user:rmq-password@192.168.3.68:5672/alpha-server")
}

pub fn setup_logger() {
    use fern;
    use chrono;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d] [%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}