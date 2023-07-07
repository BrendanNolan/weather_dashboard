pub fn setup_logger() {
    let file_appender = tracing_appender::rolling::hourly(".", "log.txt");
    let subscriber = tracing_subscriber::fmt::fmt()
        .with_writer(file_appender)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
