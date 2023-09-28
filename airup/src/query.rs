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
            let query_system = conn.query_system().await??;
            let printed_query_system =
                PrintedQuerySystem::from_query_system(&mut conn, query_system).await?;
            print_query_system(&printed_query_system);
        }
    }
    Ok(())
}

/// Prints a [QueryService] to console, in human-friendly format.
fn print_query_service(query_service: &QueryService) {
    let status = PrintedStatus::of(query_service);

    println!(
        "{} {} ({})",
        status.theme_dot(),
        query_service.service.display_name(),
        &query_service.service.name
    );
    if let Some(x) = &query_service.service.service.description {
        println!("{:>16} {}", "Description:", x);
    }
    println!("{:>16} {}", "Status:", status);
    println!(
        "{:>16} {}",
        "Main PID:",
        query_service
            .pid
            .map(|x| x.to_string())
            .unwrap_or_else(|| String::from("null"))
    );
}

/// Prints a [QuerySystem] to console, in human-friendly format.
fn print_query_system(printed: &PrintedQuerySystem) {
    let status = PrintedStatus::Active;
    println!(
        "{} {}",
        status.theme_dot(),
        printed
            .query_system
            .hostname
            .as_deref()
            .unwrap_or("localhost")
    );
    println!("{:>16} {}", "Status:", status);
    println!("{:>16} /", "Services:");
    for (name, status) in &printed.services {
        match status {
            Some(status) => println!("{1:>0$} ({2})", 18 + name.len(), name, status.fmt_simple()),
            None => println!(
                "{1:>0$} ({2})",
                18 + name.len(),
                style(name).strikethrough(),
                style("deleted").dim()
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct PrintedQuerySystem {
    query_system: QuerySystem,
    services: Vec<(String, Option<PrintedStatus>)>,
}
impl PrintedQuerySystem {
    async fn from_query_system(
        conn: &mut Connection<'_>,
        query_system: QuerySystem,
    ) -> anyhow::Result<Self> {
        let mut services = Vec::with_capacity(query_system.services.len());

        for i in query_system.services.iter() {
            let query_service = conn.query_service(i).await?.ok();
            services.push((i.clone(), query_service.map(|x| PrintedStatus::of(&x))));
        }

        Ok(Self {
            query_system,
            services,
        })
    }
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
    fn of(query_result: &QueryService) -> Self {
        let mut result = match query_result.status {
            Status::Active => Self::Active,
            Status::Stopped => Self::Stopped,
        };
        if let Some(x) = &query_result.last_error {
            result = Self::Failed(x.to_string());
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
        write!(f, "{}", self.fmt_simple())?;
        if let Self::Failed(why) = self {
            write!(f, " {}", why)?;
        }
        Ok(())
    }
}
