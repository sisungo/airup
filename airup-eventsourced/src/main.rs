mod app;
mod timer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    app::update_manifest()?;
    app::init().await?;

    timer::start();

    std::process::exit(app::airup_eventsourced().wait_for_exit_request().await);
}
