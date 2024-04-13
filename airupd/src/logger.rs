use crate::app::airupd;
use airup_sdk::{ipc::Request, system::LogRecord};

/// Invokes [`Logger::write`] on the inner logger.
pub async fn write(subject: &str, module: &str, msg: &[u8]) -> Result<(), airup_sdk::Error> {
    airupd()
        .extensions
        .rpc_invoke(Request::new("logger.append", (subject, module, msg)))
        .await
        .into_result()
}

/// Invokes [`Logger::tail`] on the inner logger.
pub async fn tail(subject: &str, n: usize) -> Result<Vec<LogRecord>, airup_sdk::Error> {
    airupd()
        .extensions
        .rpc_invoke(Request::new("logger.tail", (subject, n)))
        .await
        .into_result()
}
