use airup_sdk::blocking::{files::ServiceExt, fs::DirChain};
use anyhow::anyhow;
use clap::Parser;
use std::path::{Path, PathBuf};

/// Edit Airup files
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    file: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let editor = get_editor()?;

    if cmdline.file.strip_suffix(".airs").is_some() {
        do_edit(&editor, &find_or_create_service(&cmdline.file)?, |s| {
            airup_sdk::files::Service::read_merge(vec![s.into()])?;
            Ok(())
        })
    } else if cmdline.file.strip_suffix(".airc").is_some() {
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

fn get_editor() -> anyhow::Result<String> {
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

fn do_edit(
    editor: &str,
    path: &Path,
    check: impl FnOnce(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let temp = mktemp::Temp::new_file()?;
    std::fs::copy(path, &temp)?;
    std::process::Command::new(editor)
        .arg(temp.as_os_str())
        .status()?;
    check(&temp)?;
    std::fs::copy(&temp, path)?;
    Ok(())
}

fn find_or_create_service(name: &str) -> std::io::Result<PathBuf> {
    DirChain::new(&airup_sdk::build::manifest().service_dir).find_or_create(name)
}
