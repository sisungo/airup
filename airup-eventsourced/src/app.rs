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
    /// Calls `self.trigger_event(Event::new("airup-eventsourced_run-command".into(), command))`.
    ///
    /// The event `airup-eventsourced_run-command` is usually registered to execute a command using system default shell.
    pub fn run_command(&'static self, command: String) {
        self.trigger_event(Event::new("airup-eventsourced_run-command".into(), command))
    }

    /// Triggers an event in the event bus.
    ///
    /// If a network error occured, this will internally set the `exit_flag` to `Some(1)` and keep pending until the program
    /// exited.
    pub fn trigger_event(&'static self, event: Event) {
        tokio::spawn(async move {
            self.review_result(self.connection.lock().await.trigger_event(&event).await)
                .await
        });
    }

    /// Notifies the program to exit by setting `exit_flag` to `Some(code)`.
    pub async fn exit(&self, code: i32) -> ! {
        self.exit_flag.send(Some(code)).ok();
        std::future::pending().await
    }

    /// Waits until `exit_flag` is set to `Some(code)`. Returns the exit code.
    pub async fn wait_for_exit_request(&self) -> i32 {
        let mut exit_flag = self.exit_flag.subscribe();
        let code = exit_flag
            .wait_for(|x| x.is_some())
            .await
            .expect("the `exit_flag` channel should never be closed")
            .expect("`Receiver::wait_for(|x| x.is_some())` should return `Some(_)`");
        code
    }

    async fn review_result<T>(&self, val: Result<T, airup_sdk::ipc::Error>) -> T {
        match val {
            Ok(x) => x,
            Err(_) => self.exit(1).await,
        }
    }
}

/// Gets a reference to the unique, global [`AirupEventSourced`] instance.
///
/// # Panics
/// This method would panic if [`init`] was not previously called.
pub fn airup_eventsourced() -> &'static AirupEventSourced {
    AIRUP_EVENTSOURCED.get().unwrap()
}

/// Initializes the Airup EventSourced app for use of [`airup_eventsourced`].
pub async fn init() -> anyhow::Result<()> {
    let connection = tokio::sync::Mutex::new(Connection::connect(airup_sdk::socket_path()).await?);
    let object = AirupEventSourced {
        connection,
        exit_flag: watch::channel(None).0,
    };
    AIRUP_EVENTSOURCED.set(object).unwrap();
    Ok(())
}

/// Overrides default Airup SDK Build Manifest if environment variable `AIRUP_OVERRIDE_MANIFEST_PATH` is set.
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
