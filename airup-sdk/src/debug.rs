/// An extension trait to provide `debug.*` API invocation.
pub trait ConnectionExt<'a>: crate::Connection {
    fn is_forking_supervisable(&'a mut self) -> Self::Invoke<'a, bool> {
        self.invoke("debug.is_forking_supervisable", ())
    }

    fn dump(&'a mut self) -> Self::Invoke<'a, String> {
        self.invoke("debug.dump", ())
    }
}
impl<T> ConnectionExt<'_> for T where T: crate::Connection {}
