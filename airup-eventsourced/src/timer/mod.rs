//! # Airup Event Source: Timer

mod app;
mod runner;
mod scanner;

pub fn start() {
    tokio::spawn(main());
}

async fn main() -> anyhow::Result<()> {
    app::init().await?;
    scanner::scan().await.ok();

    loop {
        // TODO: Wait for reload signal, than call rescan
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
