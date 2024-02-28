//! # Airup Event Source: Timer

mod app;
mod scanner;
mod timer;

use crate::app::airup_eventsourced;
use airup_sdk::system::Event;

pub fn start() {
    tokio::spawn(main());
}

async fn main() -> anyhow::Result<()> {
    app::init().await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        airup_eventsourced()
            .trigger_event(&Event::new("".into(), "".into()))
            .await
            .ok();
    }
}
