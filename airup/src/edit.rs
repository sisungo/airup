use anyhow::anyhow;
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

fn _get_editor() -> anyhow::Result<String> {
    if let Ok(x) = std::env::var("EDITOR") {
        Ok(x)
    } else if let Ok(x) = std::env::var("VISUAL") {
        Ok(x)
    } else {
        Err(anyhow!(
            "cannot find an editor: neither `$EDITOR` nor `$VISUAL` is set"
        ))
    }
}
