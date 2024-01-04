//! `info.*` APIs.

use crate::{build::BuildManifest, Error};
use std::future::Future;

pub trait ConnectionExt {
    fn build_manifest(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<Result<BuildManifest, Error>>>;
}
impl ConnectionExt for super::Connection {
    async fn build_manifest(&mut self) -> anyhow::Result<Result<BuildManifest, Error>> {
        self.invoke("info.build_manifest", ()).await
    }
}
