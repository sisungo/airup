cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        #[path = "linux.rs"]
        mod sys;
    } else {
        #[path = "fallback.rs"]
        mod sys;
    }
}

/// A realm that holds isolated processes.
///
/// # Destruction
/// When the realm is dropped, all processes in the realm are released from the realm, but are not killed.
#[derive(Debug)]
pub struct Realm(sys::Realm);
impl Realm {
    pub async fn new() -> std::io::Result<Self> {
        sys::Realm::new().await.map(Self)
    }

    pub async fn set_cpu_limit(&self, max: u64) -> std::io::Result<()> {
        self.0.set_cpu_limit(max).await
    }

    pub async fn set_mem_limit(&self, max: usize) -> std::io::Result<()> {
        self.0.set_mem_limit(max).await
    }

    /// Adds a process to the realm.
    pub async fn add(&self, pid: i64) -> std::io::Result<()> {
        self.0.add(pid).await
    }

    /// Force-kills all processes in the realm.
    pub async fn kill(&self) -> std::io::Result<()> {
        self.0.kill().await
    }
}
