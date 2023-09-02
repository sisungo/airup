//! # airupd

pub mod app;
pub mod env;
pub mod ipc;
pub mod lifetime;
pub mod milestones;
pub mod storage;
pub mod supervisor;

use airupfx::prelude::*;
use milestones::AirupdExt;

/// Entrypoint of the program.
#[tokio::main]
async fn main() {
    airupfx::process::ChildQueue::init(); // Initializes the child queue
    airupfx::config::BuildManifest::init(); // Parses the built-in `build_manifest.json` for use of `airupfx::build::manifest()`
    self::env::Cmdline::init(); // Parses command-line arguments for use of `crate::env::cmdline()`
    airupfx::config::init().await; // Initializes the main configuration
    let _guard = airupfx::log::Builder::new()
        .name("airupd")
        .quiet(self::env::cmdline().quiet)
        .color(!self::env::cmdline().no_color)
        .location(airupfx::config::system_conf().locations.logs.clone())
        .init(); // Configures and initializes the logger
    app::Airupd::init().await; // Initializes the Airupd app
    let _lock = app::airupd()
        .storage
        .runtime
        .lock()
        .await
        .unwrap_log("failed to lock `airupd.lock`"); // Locks `airupd.lock`
    app::airupd()
        .storage
        .runtime
        .ipc_server()
        .await
        .unwrap_log("failed to create airupd ipc socket")
        .start(); // Starts the IPC server
    app::airupd().listen_signals();

    if !env::cmdline().quiet {
        println!(
            "Welcome to {}!\n",
            airupfx::config::system_conf().system.os_name()
        );
    }

    tokio::spawn(app::airupd().enter_milestone(env::cmdline().milestone.to_string()));

    let mut lifetime = app::airupd().lifetime.subscribe();
    if let Ok(event) = lifetime.recv().await {
        drop((_guard, _lock));
        event.deal().await;
    }
}
