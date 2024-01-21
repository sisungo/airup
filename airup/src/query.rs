use airup_sdk::{
    blocking::{system::ConnectionExt as _, Connection},
    system::{QueryService, QuerySystem, Status},
};
use anyhow::anyhow;
use chrono::prelude::*;
use clap::Parser;
use console::{style, Emoji};
use std::{collections::HashMap, fmt::Display, ops::Deref};

/// Query system information
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    /// Queries all information, including which is not loaded
    #[arg(short, long, conflicts_with = "selector")]
    all: bool,

    selector: Option<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    match cmdline.selector {
        Some(x) if x.starts_with("pid=") => {
            todo!()
        }
        Some(x) if x.ends_with(".airm") => {
            todo!()
        }
        Some(x) => {
            let queried = conn
                .query_service(&x)?
                .map_err(|e| anyhow!("failed to query service `{}`: {}", x, e))?;
            print_query_service(&queried);

            if let Ok(Ok(logs)) = conn.tail_logs(&format!("airup_service_{}", x), 4) {
                println!("\n{}", style("Logs:").bold().underlined());
                for log in logs {
                    println!("{}", log.message);
                }
            }
        }
        None => {
            let query_system = conn.query_system()??;
            print_query_system(&mut conn, &query_system, &cmdline)?;
        }
    }
    Ok(())
}

/// Prints a [`QueryService`] to console, in human-friendly format.
fn print_query_service(query_service: &QueryService) {
    let status = PrintedStatus::of_service(query_service);

    println!(
        "{} {} ({})",
        status.theme_dot(),
        query_service.definition.display_name(),
        &query_service.definition.name
    );

    if let Some(x) = &query_service.definition.service.description {
        println!("{:>14} {}", "Description:", x);
    }
    if let Some(x) = &query_service.definition.service.homepage {
        println!("{:>14} {}", "Homepage:", x);
    }
    if let Some(x) = &query_service.definition.service.docs {
        println!("{:>14} {}", "Docs:", x);
    }

    println!("{:>14} {}", "Status:", status);

    println!(
        "{:>14} {}",
        "Main PID:",
        query_service
            .pid
            .map(|x| x.to_string())
            .unwrap_or_else(|| format!("{}", style("(null)").dim()))
    );
}

/// Prints a [`QuerySystem`] to console, in human-friendly format.
fn print_query_system(
    conn: &mut Connection,
    query_system: &QuerySystem,
    cmdline: &Cmdline,
) -> anyhow::Result<()> {
    let mut services = HashMap::with_capacity(query_system.services.len());
    for i in query_system.services.iter() {
        let query_service = conn.query_service(i)?.ok();
        services.insert(
            i.clone(),
            query_service.map(|x| PrintedStatus::of_service(&x)),
        );
    }
    if cmdline.all {
        for name in conn.list_services()?? {
            services.insert(
                name.clone(),
                conn.query_service(&name)?
                    .ok()
                    .map(|x| PrintedStatus::of_service(&x)),
            );
        }
    }

    let status = PrintedStatus::of_system(query_system);
    println!(
        "{} {}",
        status.theme_dot(),
        query_system.hostname.as_deref().unwrap_or("localhost")
    );
    println!("{:>14} {}", "Status:", status);
    println!("{:>14} /", "Services:");
    for (name, status) in &services {
        match status {
            Some(status) => println!("{}{} ({})", " ".repeat(16), name, status.kind),
            None => println!(
                "{}{} ({})",
                " ".repeat(16),
                style(name).strikethrough(),
                style("deleted").dim()
            ),
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum PrintedStatusKind {
    Active,
    Stopped,
    Failed,
    Starting,
    Stopping,
}
impl PrintedStatusKind {
    fn of_service(query_service: &QueryService) -> Self {
        let mut result = match query_service.status {
            Status::Active => Self::Active,
            Status::Stopped => Self::Stopped,
        };
        if query_service.last_error.is_some() {
            result = Self::Failed;
        }
        if let Some(x) = query_service.task_class.as_deref() {
            match x {
                "StartService" => result = Self::Starting,
                "StopService" => result = Self::Stopping,
                _ => {}
            }
        }

        result
    }

    fn of_system(query_system: &QuerySystem) -> Self {
        if query_system.is_booting {
            return Self::Starting;
        }

        match query_system.status {
            Status::Active => Self::Active,
            Status::Stopped => Self::Stopped,
        }
    }

    fn theme_dot(&self) -> String {
        let theme_dot = style(Emoji("●", "*"));
        match self {
            Self::Active => theme_dot.green(),
            Self::Stopped => theme_dot,
            Self::Failed => theme_dot.red(),
            Self::Starting | Self::Stopping => theme_dot.blue(),
        }
        .to_string()
    }
}
impl Display for PrintedStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "{}", style("active").bold().green()),
            Self::Stopped => write!(f, "{}", style("stopped").bold()),
            Self::Failed => write!(f, "{}", style("failed").bold().red()),
            Self::Starting => write!(f, "{}", style("starting").bold().blue()),
            Self::Stopping => write!(f, "{}", style("starting").bold().blue()),
        }
    }
}

#[derive(Debug, Clone)]
struct PrintedStatus {
    kind: PrintedStatusKind,
    since: Option<i64>,
    error: Option<String>,
}
impl PrintedStatus {
    fn of_service(query_service: &QueryService) -> Self {
        let kind = PrintedStatusKind::of_service(query_service);
        let error = query_service.last_error.as_ref().map(ToString::to_string);
        let since = query_service.status_since;
        Self { kind, since, error }
    }

    fn of_system(query_system: &QuerySystem) -> Self {
        let kind = PrintedStatusKind::of_system(query_system);
        let since = query_system
            .booted_since
            .unwrap_or(query_system.boot_timestamp);
        Self {
            kind,
            since: Some(since),
            error: None,
        }
    }
}
impl Deref for PrintedStatus {
    type Target = PrintedStatusKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}
impl Display for PrintedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.kind, f)?;
        let ps = format!(
            "{}; since {}",
            self.error.as_deref().unwrap_or_default(),
            self.since
                .map(|x| {
                    let dt = DateTime::from_timestamp(x / 1000, 0);
                    dt.map(|x| Local.from_utc_datetime(&x.naive_utc()).to_string())
                        .unwrap_or_else(|| x.to_string())
                })
                .unwrap_or_default(),
        );
        let ps = ps.trim_start_matches("; ");
        if ps != "since " {
            write!(f, " ({ps})")?;
        }
        Ok(())
    }
}
