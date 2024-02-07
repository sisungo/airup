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
    pub fn new() -> std::io::Result<Self> {
        sys::Realm::new().map(Self)
    }

    pub fn set_cpu_limit(&self, max: u64) -> std::io::Result<()> {
        self.0.set_cpu_limit(max)
    }

    pub fn set_mem_limit(&self, max: usize) -> std::io::Result<()> {
        self.0.set_mem_limit(max)
    }

    /// Adds a process to the realm.
    pub fn add(&self, pid: i64) -> std::io::Result<()> {
        self.0.add(pid)
    }

    /// Force-kills all processes in the realm.
    pub fn kill(&self) -> std::io::Result<()> {
        self.0.kill()
    }
}
