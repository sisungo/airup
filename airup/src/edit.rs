use clap::Parser;

/// Edit Airup files
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    file: String,
}

pub async fn main(_: Cmdline) -> anyhow::Result<()> {
    todo!()
}
