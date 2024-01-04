use crate::blocking::fs::DirChain;
use crate::files::{milestone, Milestone, ReadError};
use std::path::Path;

pub trait MilestoneExt {
    fn read_from<P: AsRef<Path>>(path: P) -> Result<Milestone, ReadError>;
    fn items(&self) -> Vec<milestone::Item>;
}
impl MilestoneExt for Milestone {
    fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        read_from(path.as_ref())
    }

    fn items(&self) -> Vec<milestone::Item> {
        let mut services = Vec::new();
        let chain = DirChain::new(&self.base_dir);

        let Ok(read_chain) = chain.read_chain() else {
            return services;
        };

        for i in read_chain {
            if !i.to_string_lossy().ends_with(".list.airf") {
                continue;
            }
            let Some(path) = chain.find(&i) else {
                continue;
            };
            let Ok(list_str) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in list_str.lines() {
                if let Ok(item) = line.parse() {
                    services.push(item);
                }
            }
        }

        services
    }
}

fn read_from(path: &Path) -> Result<Milestone, ReadError> {
    let get_name = |p: &Path| -> Result<String, ReadError> {
        Ok(p.file_stem()
            .ok_or_else(|| ReadError::from("invalid milestone path"))?
            .to_string_lossy()
            .into())
    };
    let manifest = toml::from_str(&std::fs::read_to_string(
        path.join(milestone::Manifest::FILE_NAME),
    )?)?;
    let mut name = get_name(path)?;
    if name == "default" {
        name = get_name(&std::fs::canonicalize(path)?)?;
    }

    Ok(Milestone {
        name,
        manifest,
        base_dir: path.into(),
    })
}
