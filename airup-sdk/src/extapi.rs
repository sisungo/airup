use crate::system::LogRecord;

/// An extension trait to provide invocation for conventional `extapi.*` APIs.
pub trait ConnectionExt<'a>: crate::Connection {
    fn append_log(
        &'a mut self,
        subject: &'a str,
        module: &'a str,
        msg: &'a [u8],
    ) -> Self::Invoke<'a, ()> {
        self.invoke("extapi.logger.append", (subject, module, msg))
    }

    fn tail_logs(&'a mut self, subject: &'a str, n: usize) -> Self::Invoke<'a, Vec<LogRecord>> {
        self.invoke("extapi.logger.tail", (subject, n))
    }
}
impl<'a, T> ConnectionExt<'a> for T where T: crate::Connection {}
