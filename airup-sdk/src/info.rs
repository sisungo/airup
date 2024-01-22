use crate::build::BuildManifest;

pub trait ConnectionExt<'a>: crate::Connection {
    fn build_manifest(&'a mut self) -> Self::Invoke<'a, BuildManifest> {
        self.invoke("info.build_manifest", ())
    }
}
impl<'a, T> ConnectionExt<'a> for T where T: crate::Connection {}
