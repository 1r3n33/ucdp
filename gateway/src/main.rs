use crossbeam_channel::unbounded;

mod ucdp;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let (sender, receiver) = unbounded::<::ucdp::stream::events::Events>();

    // Start thread that will receive events to send them to the stream
    ::ucdp::stream::producer::spawn_stream_producer_thread(receiver);

    // Start web service
    ucdp::web::run_http_server(sender).await
}
