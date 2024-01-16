//! Event subsystem of the Airup daemon.

/// The event bus.
#[derive(Debug)]
pub struct Bus {
    sender: async_broadcast::Sender<String>,
    _receiver: async_broadcast::InactiveReceiver<String>,
}
impl Bus {
    /// Creates a new [`Bus`] instance.
    pub fn new() -> Self {
        let (sender, _receiver) = async_broadcast::broadcast(16);
        let _receiver = _receiver.deactivate();
        Self { sender, _receiver }
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
