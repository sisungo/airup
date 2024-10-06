use crate::build::BuildManifest;

/// An extension trait to provide `info.*` API invocation.
pub trait ConnectionExt<'a>: crate::Connection {
    fn build_manifest(&'a mut self) -> Self::Invoke<'a, BuildManifest> {
        self.invoke("info.build_manifest", ())
    }

    fn version(&'a mut self) -> Self::Invoke<'a, String> {
        self.invoke("info.version", ())
    }
}
impl<T> ConnectionExt<'_> for T where T: crate::Connection {}
