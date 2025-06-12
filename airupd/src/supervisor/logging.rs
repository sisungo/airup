use airup_sdk::rpc::Request;
use airupfx::io::line_piper::Callback as LinePiperCallback;
use std::pin::Pin;

#[derive(Clone)]
pub struct LogCallback {
    name: String,
    module: &'static str,
}
impl LogCallback {
    pub fn new(name: String, module: &'static str) -> Self {
        Self { name, module }
    }
}
impl LinePiperCallback for LogCallback {
    fn invoke<'a>(
        &'a self,
        msg: &'a [u8],
    ) -> Pin<Box<dyn for<'b> Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            send(self.name.clone(), self.module, msg.to_vec());
        })
    }

    fn clone_boxed(&self) -> Box<dyn LinePiperCallback> {
        Box::new(self.clone())
    }
}

pub fn send(name: String, module: &'static str, msg: Vec<u8>) {
    tokio::spawn(async move {
        _ = crate::app::airupd()
            .extensions
            .rpc_invoke(Request::new("logger.append", (&name, &module, &msg)))
            .await;
    });
}
