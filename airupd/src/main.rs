//! # airupd

pub mod app;
pub mod env;
pub mod ipc;
pub mod lifetime;
pub mod logger;
pub mod milestones;
pub mod storage;
pub mod supervisor;

use airupfx::prelude::*;

/// Entrypoint of the program.
#[tokio::main]
async fn main() {
    // Initializes Airup subsystems
    airupfx::sys::init();
    env::Cmdline::init();
    airupfx::config::init().await;
    airupfx::log::Builder::new()
        .name("airupd")
        .quiet(self::env::cmdline().quiet)
        .color(!self::env::cmdline().no_color)
        .verbose(self::env::cmdline().verbose)
        .init();
    milestones::early_boot::enter().await;
    app::Airupd::init().await;

    // Creates Airup runtime primitives
    let _lock = app::airupd()
        .storage
        .runtime
        .lock()
        .await
        .unwrap_log("unable to lock database");
    app::airupd()
        .storage
        .runtime
        .ipc_server()
        .await
        .unwrap_log("failed to create airupd ipc socket")
        .start();
    app::airupd().listen_signals();

    if !env::cmdline().quiet {
        println!(
            "Welcome to {}!\n",
            airupfx::config::system_conf().system.os_name
        );
    }

    tokio::spawn(app::airupd().enter_milestone(env::cmdline().milestone.to_string()));

    let mut lifetime = app::airupd().lifetime.subscribe();
    if let Ok(event) = lifetime.recv().await {
        drop(_lock);
        event.handle();
    }
}
