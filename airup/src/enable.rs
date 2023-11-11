use clap::Parser;

/// Enable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    unit: String,
}

pub async fn main(_: Cmdline) -> anyhow::Result<()> {
    todo!()
}
