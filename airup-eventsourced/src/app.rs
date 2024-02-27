use airup_sdk::{
    nonblocking::Connection,
    system::{ConnectionExt as _, Event},
};
use anyhow::anyhow;
use std::sync::OnceLock;
use tokio::sync::watch;

static AIRUP_EVENTSOURCED: OnceLock<AirupEventSourced> = OnceLock::new();

#[derive(Debug)]
pub struct AirupEventSourced {
    connection: tokio::sync::Mutex<Connection>,
    exit_flag: watch::Sender<Option<i32>>,
}
impl AirupEventSourced {
    pub async fn trigger_event(&self, event: &Event) -> Result<(), airup_sdk::Error> {
        self.review_result(self.connection.lock().await.trigger_event(event).await)
            .await
    }

    async fn review_result<T>(&self, val: Result<T, airup_sdk::ipc::Error>) -> T {
        match val {
            Ok(x) => x,
            Err(_) => {
                self.exit(1);
                std::future::pending::<T>().await
            }
        }
    }

    pub fn exit(&self, code: i32) {
        self.exit_flag.send(Some(code)).ok();
    }

    pub async fn wait_for_exit_request(&self) -> i32 {
        let mut exit_flag = self.exit_flag.subscribe();
        let code = exit_flag
            .wait_for(|x| x.is_some())
            .await
            .expect("the `exit_flag` channel should never be closed")
            .expect("`Receiver::wait_for(|x| x.is_some())` should return `Some(_)`");
        code
    }
}

/// Gets a reference to the unique, global [`AirupEventSourced`] instance.
///
/// # Panics
/// This method would panic if [`init`] was not previously called.
pub fn airup_eventsourced() -> &'static AirupEventSourced {
    AIRUP_EVENTSOURCED.get().unwrap()
}

/// Initializes the Airupd app for use of [`airup_eventsourced`].
pub async fn init() -> anyhow::Result<()> {
    let connection = tokio::sync::Mutex::new(Connection::connect(airup_sdk::socket_path()).await?);
    let object = AirupEventSourced {
        connection,
        exit_flag: watch::channel(None).0,
    };
    AIRUP_EVENTSOURCED.set(object).unwrap();
    Ok(())
}

pub fn update_manifest() -> anyhow::Result<()> {
    if let Some(path) = std::env::var_os("AIRUP_OVERRIDE_MANIFEST_PATH") {
        airup_sdk::build::set_manifest(
            serde_json::from_slice(
                &std::fs::read(path)
                    .map_err(|err| anyhow!("Failed to read overridden build manifest: {err}"))?,
            )
            .map_err(|err| anyhow!("failed to parse overridden build manifest: {err}"))?,
        );
    }

    Ok(())
}
