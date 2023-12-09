use anyhow::anyhow;
use clap::Parser;

/// Edit Airup files
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    file: String,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    if let Some(_) = cmdline.file.strip_suffix(".airs") {
        todo!()
    } else if let Some(_) = cmdline.file.strip_suffix(".airc") {
        todo!()
    } else {
        let (n, name) = cmdline.file.split('.').enumerate().last().unwrap();
        if n > 0 {
            Err(anyhow!("unknown file suffix `{name}`"))
        } else {
            Err(anyhow!("file suffix must be specified to edit"))
        }
        
    }
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
