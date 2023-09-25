use crate::{process::ExitStatus, signal::SIGTERM};
use std::{collections::HashMap, hash::BuildHasher, future::Future};
use tokio::sync::mpsc;

pub type BuiltinModule = fn(args: Vec<String>) -> mpsc::Receiver<i32>;

pub fn init<H: BuildHasher>(builtins: &mut HashMap<&'static str, BuiltinModule, H>) {
    builtins.insert("noop", noop);
    builtins.insert("console.info", console_info);
    builtins.insert("console.warn", console_warn);
    builtins.insert("console.error", console_error);
}

pub fn console_info(args: Vec<String>) -> mpsc::Receiver<i32> {
    tracing::info!(target: "console", "{}", merge_args(&args));
    builtin_impl(async { 0 })
}

pub fn console_warn(args: Vec<String>) -> mpsc::Receiver<i32> {
    tracing::warn!(target: "console", "{}", merge_args(&args));
    builtin_impl(async { 0 })
}

pub fn console_error(args: Vec<String>) -> mpsc::Receiver<i32> {
    tracing::error!(target: "console", "{}", merge_args(&args));
    builtin_impl(async { 0 })
}

pub fn noop(_: Vec<String>) -> mpsc::Receiver<i32> {
    builtin_impl(async { 0 })
}

pub async fn wait(rx: &mut mpsc::Receiver<i32>) -> ExitStatus {
    match rx.recv().await {
        Some(code) => ExitStatus::Exited(code as _),
        None => ExitStatus::Signaled(SIGTERM),
    }
}

fn builtin_impl<F: Future<Output = i32> + Send + Sync + 'static>(future: F) -> mpsc::Receiver<i32> {
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        ret(tx, future.await).await;
    });
    rx
}

async fn ret(tx: mpsc::Sender<i32>, val: i32) {
    while let Ok(_) = tx.send(val).await {}
}

fn merge_args(args: &[String]) -> String {
    let mut result = String::with_capacity(args.len() * 12);
    for arg in args {
        result.push_str(arg);
        result.push(' ');
    }
    result.pop();
    result
}