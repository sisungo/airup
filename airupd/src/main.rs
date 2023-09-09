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
    airupfx::process::init(); // Initializes the process system
    self::env::Cmdline::init(); // Parses command-line arguments for use of `crate::env::cmdline()`
    airupfx::config::init().await; // Initializes the main configuration
    airupfx::log::Builder::new()
        .name("airupd")
        .quiet(self::env::cmdline().quiet)
        .color(!self::env::cmdline().no_color)
        .init(); // Configures and initializes the logger
    app::Airupd::init().await; // Initializes the Airupd app
    let _lock = app::airupd()
        .storage
        .runtime
        .lock()
        .await
        .unwrap_log("unable to lock database"); // Locks the database
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
