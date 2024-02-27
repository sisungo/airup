use airup_sdk::system::{ConnectionExt as _, Event};
use clap::Parser;

/// Trigger an event in the event bus
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    id: String,

    #[arg(short, long, default_value_t)]
    payload: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    let event = Event::new(cmdline.id, cmdline.payload);
    conn.trigger_event(&event)??;

    Ok(())
}
