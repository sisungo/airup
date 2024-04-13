use super::app::timer_app;
use airup_sdk::nonblocking::{files, fs::DirChain};

/// Scan the services directory for timers.
///
/// If a timer loaded with the same name as the scan file is different from that version on filesystem, it is reloaded. If
/// a timer was loaded, but the name no longer exists on filesystem, it is dropped. If it isn't loaded, it is loaded.
pub async fn scan() -> anyhow::Result<()> {
    let services_chain = DirChain::new(&airup_sdk::build::manifest().service_dir);
    let config_chain = DirChain::new(&airup_sdk::build::manifest().config_dir);

    let timers: Vec<_> = services_chain
        .read_chain()
        .await?
        .into_iter()
        .map(|x| x.to_string_lossy().to_string())
        .filter(|x| x.ends_with(".airt"))
        .collect();

    for timer in &timers {
        let Some(timer_path) = services_chain.find(&timer[..]).await else {
            continue;
        };
        let config = format!("{timer}.airc");
        let mut paths = vec![timer_path];
        if let Some(x) = config_chain.find(&config).await {
            paths.push(x);
        }
        let Ok(timer_def) = files::read_merge(paths).await else {
            continue;
        };
        timer_app().feed_timer(timer.to_string(), timer_def);
    }

    timer_app().retain_timers(|x| timers.contains(x));

    Ok(())
}
