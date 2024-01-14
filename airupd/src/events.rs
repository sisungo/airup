//! Event subsystem of the Airup daemon.

/// The event bus.
#[derive(Debug)]
pub struct Bus {
    sender: async_broadcast::Sender<String>,
}
impl Bus {
    /// Creates a new [`Bus`] instance.
    pub fn new() -> Self {
        Self {
            sender: async_broadcast::broadcast(16).0,
        }
    }

    /// Subscribes to the bus.
    pub fn subscribe(&self) -> async_broadcast::Receiver<String> {
        self.sender.new_receiver()
    }

    /// Triggers an event in the bus.
    pub async fn trigger(&self, s: String) {
        self.sender.broadcast(s).await.ok();
    }
}
impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}
