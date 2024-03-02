//! # Airup Event Source: Timer

mod app;
mod runner;
mod scanner;

/// Starts a new task to run the timer event source.
pub fn start() {
    tokio::spawn(main());
}

async fn main() -> anyhow::Result<()> {
    app::init().await?;
    scanner::scan().await.ok();

    loop {
        crate::app::airup_eventsourced()
            .wait_for_reload_request()
            .await;
        scanner::scan().await.ok();
    }
}
