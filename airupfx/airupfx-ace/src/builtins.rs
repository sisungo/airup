//! Built-in commands of ACE.

use airupfx_process::ExitStatus;
use libc::SIGTERM;
use std::{collections::HashMap, hash::BuildHasher, time::Duration};
use tokio::task::JoinHandle;

pub type BuiltinModule = fn(args: Vec<String>) -> JoinHandle<i32>;

pub fn init<H: BuildHasher>(builtins: &mut HashMap<&'static str, BuiltinModule, H>) {
    builtins.insert("noop", noop);
    builtins.insert("console.setup", console_setup);
    builtins.insert("console.info", console_info);
    builtins.insert("console.warn", console_warn);
    builtins.insert("console.error", console_error);
    builtins.insert("builtin.sleep", sleep);
}

pub fn console_setup(args: Vec<String>) -> JoinHandle<i32> {
    tokio::spawn(async move {
        let path = match args.first() {
            Some(x) => x,
            None => return 1,
        };
        match airupfx_env::setup_stdio(path.as_ref()).await {
            Ok(()) => 0,
            Err(_) => 2,
        }
    })
}

pub fn console_info(args: Vec<String>) -> JoinHandle<i32> {
    tracing::info!(target: "console", "{}", merge_args(&args));
    tokio::spawn(async { 0 })
}

pub fn console_warn(args: Vec<String>) -> JoinHandle<i32> {
    tracing::warn!(target: "console", "{}", merge_args(&args));
    tokio::spawn(async { 0 })
}

pub fn console_error(args: Vec<String>) -> JoinHandle<i32> {
    tracing::error!(target: "console", "{}", merge_args(&args));
    tokio::spawn(async { 0 })
}

pub fn noop(_: Vec<String>) -> JoinHandle<i32> {
    tokio::spawn(async { 0 })
}

pub fn sleep(args: Vec<String>) -> JoinHandle<i32> {
    tokio::spawn(async move {
        let duration = match args.first() {
            Some(x) => x,
            None => return 1,
        };
        let duration: u64 = match duration.parse() {
            Ok(x) => x,
            Err(_) => return 2,
        };

        tokio::time::sleep(Duration::from_millis(duration)).await;

        0
    })
}

pub async fn wait(rx: &mut JoinHandle<i32>) -> ExitStatus {
    (rx.await).map_or(ExitStatus::Signaled(SIGTERM), |code| {
        ExitStatus::Exited(code as _)
    })
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
