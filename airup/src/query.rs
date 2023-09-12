use airup_sdk::prelude::*;
use clap::Parser;
use console::{style, Emoji};
use std::fmt::Display;

/// Query system information
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: Option<String>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airup_sdk::socket_path()).await?;
    match cmdline.service {
        Some(x) => {
            let queried = conn.query_service(&x).await??;
            print_query_service(&queried);
        }
        None => {
            let queried = conn.query_system().await??;
            print_query_system(&queried);
        }
    }
    Ok(())
}

/// Prints a [QueryService] to console, in human-friendly format.
fn print_query_service(query_result: &QueryService) {
    let status = PrintedStatus::of(query_result);

    println!(
        "{} {} ({})",
        status.theme_dot(),
        query_result.service.display_name(),
        &query_result.service.name
    );
    println!("{:>12} {}", "Status:", status);
    println!(
        "{:>12} {}",
        "Main PID:",
        query_result
            .pid
            .map(|x| x.to_string())
            .unwrap_or_else(|| String::from("null"))
    );
}

/// Prints a [QuerySystem] to console, in human-friendly format.
fn print_query_system(query_system: &QuerySystem) {
    let status = PrintedStatus::Active;
    println!(
        "{} {}",
        status.theme_dot(),
        query_system.hostname.as_deref().unwrap_or("localhost")
    );
    println!("{:>12} {}", "Status:", status);
    println!("{:>12} /", "Services:");
    for i in &query_system.services {
        println!("{1:>0$}", 14 + i.len(), i);
    }
}

#[derive(Debug, Clone)]
enum PrintedStatus {
    Active,
    Stopped,
    Failed,
    Starting,
    Stopping,
}
impl PrintedStatus {
    pub fn of(query_result: &QueryService) -> Self {
        let mut result = match query_result.status {
            Status::Active => Self::Active,
            Status::Stopped => Self::Stopped,
        };
        if let Some(_) = &query_result.last_error {
            result = Self::Failed;
        }
        if let Some(x) = query_result.task.as_deref() {
            match x {
                "StartService" => result = Self::Starting,
                "StopService" => result = Self::Stopping,
                _ => {}
            }
        }

        result
    }

    pub fn theme_dot(&self) -> String {
        let theme_dot = style(Emoji("●", "*"));
        match self {
            PrintedStatus::Active => theme_dot.green(),
            PrintedStatus::Stopped => theme_dot,
            PrintedStatus::Failed => theme_dot.red(),
            PrintedStatus::Starting | PrintedStatus::Stopping => theme_dot.blue(),
        }
        .to_string()
    }
}
impl Display for PrintedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "{}", style("active").bold().green()),
            Self::Stopped => write!(f, "{}", style("stopped").bold()),
            Self::Failed => write!(f, "{}", style("failed").bold().red()),
            Self::Starting => write!(f, "{}", style("starting").bold().blue()),
            Self::Stopping => write!(f, "{}", style("stopping").bold().blue()),
        }
    }
}