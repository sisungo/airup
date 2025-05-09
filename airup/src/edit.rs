use airup_sdk::{
    blocking::{files, fs::DirChain},
    files::Service,
};
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
            files::read_merge::<Service>(vec![s.into()])?;
            Ok(())
        })?;
    } else if let Some(x) = cmdline.file.strip_suffix(".airc") {
        let service = find_or_create_service(&format!("{x}.airs"))?;
        do_edit(&editor, &find_or_create_config(&cmdline.file)?, |s| {
            files::read_merge::<Service>(vec![service, s.into()])?;
            Ok(())
        })?;
    } else {
        let (n, name) = cmdline.file.split('.').enumerate().last().unwrap();
        if n > 0 {
            return Err(anyhow!("unknown file suffix `{name}`"));
        } else {
            return Err(anyhow!("file suffix must be specified to edit"));
        }
    }

    println!("note: You may run `airup self-reload` to ensure necessary cache to be refreshed.");

    Ok(())
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

fn find_or_create_config(name: &str) -> std::io::Result<PathBuf> {
    DirChain::new(&airup_sdk::build::manifest().config_dir).find_or_create(name)
}
