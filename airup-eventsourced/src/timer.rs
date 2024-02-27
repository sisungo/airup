//! # Airup Event Source: Timer

use crate::app::airup_eventsourced;
use airup_sdk::system::Event;

pub fn start() {
    tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            airup_eventsourced()
                .trigger_event(&Event::new("".into(), "".into()))
                .await
                .ok();
        }
    });
}
