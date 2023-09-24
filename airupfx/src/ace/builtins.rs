use crate::{process::ExitStatus, signal::SIGTERM};
use std::{collections::HashMap, hash::BuildHasher};
use tokio::sync::mpsc;

pub type BuiltinModule = fn(args: &[String]) -> mpsc::Receiver<i32>;

pub fn init<H: BuildHasher>(builtins: &mut HashMap<&'static str, BuiltinModule, H>) {
    builtins.insert("noop", noop);
}

pub fn noop(_: &[String]) -> mpsc::Receiver<i32> {
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(ret(tx, 0));
    rx
}

pub async fn wait(rx: &mut mpsc::Receiver<i32>) -> ExitStatus {
    match rx.recv().await {
        Some(code) => ExitStatus::Exited(code as _),
        None => ExitStatus::Signaled(SIGTERM),
    }
}

async fn ret(tx: mpsc::Sender<i32>, val: i32) {
    while let Ok(_) = tx.send(val).await {}
}
