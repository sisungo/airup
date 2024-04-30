//! # airupd

mod ace;
mod app;
mod env;
mod events;
mod extension;
mod ipc;
mod lifetime;
mod logging;
mod milestones;
mod storage;
mod supervisor;

use airupfx::prelude::*;

/// Entrypoint of the program.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmdline = self::env::cmdline();
    logging::Builder::new()
        .name("airupd")
        .quiet(cmdline.quiet)
        .color(!cmdline.no_color)
        .verbose(cmdline.verbose)
        .init();
    app::set_manifest_at(cmdline.build_manifest.as_deref()).await;
    milestones::early_boot::enter().await;
    app::init().await;

    // Creates Airup runtime primitives
    app::airupd().storage.config.override_env();
    let _lock = app::airupd()
        .storage
        .runtime
        .lock()
        .await
        .unwrap_log("unable to lock database")
        .await;
    app::airupd()
        .storage
        .runtime
        .ipc_server()
        .await
        .unwrap_log("failed to create airupd ipc socket")
        .await
        .start();
    app::airupd().listen_signals();

    if std::process::id() == 1 && !env::cmdline().quiet {
        println!(
            "Welcome to {}!\n",
            app::airupd().storage.config.system_conf.system.os_name
        );
    }

    app::airupd().bootstrap_milestone(env::cmdline().milestone.to_string());

    let mut lifetime = app::airupd().lifetime.subscribe();
    if let Ok(event) = lifetime.recv().await {
        drop(_lock);
        event.handle().await;
    }
}
