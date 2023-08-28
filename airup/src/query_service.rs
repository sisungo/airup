use airupfx::sdk::prelude::*;
use clap::Parser;
use console::style;
use std::fmt::Display;

/// Query service information
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: Option<String>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airupfx::sdk::socket_path()).await?;
    match cmdline.service {
        Some(x) => {
            let query_result = conn.query_service(&x).await??;
            print_query_result(&query_result);
        }
        None => {
            let supervisors = conn.supervisors().await??;
            for supervisor in supervisors {
                println!("{supervisor}");
            }
        }
    }
    Ok(())
}

fn print_query_result(query_result: &QueryResult) {
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

#[derive(Debug, Clone)]
enum PrintedStatus {
    Active,
    Stopped,
    Failed,
    Starting,
    Stopping,
}
impl PrintedStatus {
    pub fn of(query_result: &QueryResult) -> Self {
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
        match self {
            PrintedStatus::Active => style("●").green(),
            PrintedStatus::Stopped => style("●"),
            PrintedStatus::Failed => style("●").red(),
            PrintedStatus::Starting | PrintedStatus::Stopping => style("●").blue(),
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
