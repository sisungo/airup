use airup_sdk::prelude::*;
use clap::Parser;
use console::{style, Emoji};
use std::fmt::Display;

/// Query system information
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    /// Queries all data, including which is not loaded
    #[arg(short, long, conflicts_with = "unit")]
    all: bool,

    unit: Option<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = BlockingConnection::connect(airup_sdk::socket_path())?;
    match cmdline.unit {
        Some(x) => {
            let queried = conn.query_service(&x)??;
            print_query_service(&queried);
        }
        None => {
            let query_system = conn.query_system()??;
            print_query_system(&mut conn, &query_system, &cmdline)?;
        }
    }
    Ok(())
}

/// Prints a [QueryService] to console, in human-friendly format.
fn print_query_service(query_service: &QueryService) {
    let status = PrintedStatus::of_service(query_service);

    println!(
        "{} {} ({})",
        status.theme_dot(),
        query_service.service.display_name(),
        &query_service.service.name
    );
    if let Some(x) = &query_service.service.service.description {
        println!("{:>14} {}", "Description:", x);
    }
    println!("{:>14} {}", "Status:", status);
    println!(
        "{:>14} {}",
        "Main PID:",
        query_service
            .pid
            .map(|x| x.to_string())
            .unwrap_or_else(|| String::from("null"))
    );
}

/// Prints a [QuerySystem] to console, in human-friendly format.
fn print_query_system(
    conn: &mut BlockingConnection<'_>,
    query_system: &QuerySystem,
    cmdline: &Cmdline,
) -> anyhow::Result<()> {
    let mut services = Vec::with_capacity(query_system.services.len());
    for i in query_system.services.iter() {
        let query_service = conn.query_service(i)?.ok();
        services.push((
            i.clone(),
            query_service.map(|x| PrintedStatus::of_service(&x)),
        ));
    }
    if cmdline.all {
        for name in conn.list_services()?? {
            services.push((
                name.clone(),
                conn.query_service(&name)?
                    .ok()
                    .map(|x| PrintedStatus::of_service(&x)),
            ));
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
            Some(status) => println!("{1:>0$} ({2})", 18 + name.len(), name, status.fmt_simple()),
            None => println!(
                "{1:>0$} ({2})",
                16 + name.len(),
                style(name).strikethrough(),
                style("deleted").dim()
            ),
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum PrintedStatus {
    Active,
    Stopped,
    Failed(String),
    Starting,
    Stopping,
}
impl PrintedStatus {
    fn of_service(query_service: &QueryService) -> Self {
        let mut result = match query_service.status {
            Status::Active => Self::Active,
            Status::Stopped => Self::Stopped,
        };
        if let Some(x) = &query_service.last_error {
            result = Self::Failed(x.to_string());
        }
        if let Some(x) = query_service.task.as_deref() {
            match x {
                "StartService" => result = Self::Starting,
                "StopService" => result = Self::Stopping,
                _ => {}
            }
        }

        result
    }

    fn of_system(query_system: &QuerySystem) -> Self {
        match query_system.status {
            Status::Active => Self::Active,
            Status::Stopped => Self::Stopped,
        }
    }

    fn theme_dot(&self) -> String {
        let theme_dot = style(Emoji("●", "*"));
        match self {
            PrintedStatus::Active => theme_dot.green(),
            PrintedStatus::Stopped => theme_dot,
            PrintedStatus::Failed(_) => theme_dot.red(),
            PrintedStatus::Starting | PrintedStatus::Stopping => theme_dot.blue(),
        }
        .to_string()
    }

    fn fmt_simple(&self) -> impl Display {
        match self {
            Self::Active => style("active").bold().green(),
            Self::Stopped => style("stopped").bold(),
            Self::Failed(_) => style("failed").bold().red(),
            Self::Starting => style("starting").bold().blue(),
            Self::Stopping => style("starting").bold().blue(),
        }
    }
}
impl Display for PrintedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let extra = match self {
            Self::Failed(why) => format!("{}", why),
            _ => format!(""),
        };
        write!(f, "{} ({})", self.fmt_simple(), extra)?;
        Ok(())
    }
}
