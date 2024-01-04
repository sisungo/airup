//! `info.*` APIs.

use crate::{build::BuildManifest, Error};

pub trait ConnectionExt {
    fn build_manifest(&mut self) -> anyhow::Result<Result<BuildManifest, Error>>;
}
impl ConnectionExt for super::Connection {
    fn build_manifest(&mut self) -> anyhow::Result<Result<BuildManifest, Error>> {
        self.invoke("info.build_manifest", ())
    }
}
