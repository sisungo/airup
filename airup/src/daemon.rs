use clap::Parser;

/// Start a new Airup daemon
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(last = true)]
    args: Vec<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let temp = mktemp::Temp::new_file()?;
    let mut file = std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp)?;
    ciborium::into_writer(airup_sdk::build::manifest(), &mut file)?;

    std::process::Command::new("airupd")
        .args(&cmdline.args)
        .arg("--build-manifest")
        .arg(temp.as_os_str())
        .spawn()?
        .wait()?;

    Ok(())
}
