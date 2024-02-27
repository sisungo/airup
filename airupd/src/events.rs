//! Event subsystem of the Airup daemon.

use airup_sdk::system::Event;

/// The event bus.
#[derive(Debug)]
pub struct Bus {
    sender: async_broadcast::Sender<Event>,
    _receiver: async_broadcast::InactiveReceiver<Event>,
}
impl Bus {
    /// Creates a new [`Bus`] instance.
    pub fn new() -> Self {
        let (sender, _receiver) = async_broadcast::broadcast(16);
        let _receiver = _receiver.deactivate();
        Self { sender, _receiver }
    }

    /// Subscribes to the bus.
    pub fn subscribe(&self) -> async_broadcast::Receiver<Event> {
        self.sender.new_receiver()
    }

    /// Triggers an event in the bus.
    pub async fn trigger(&self, event: Event) {
        self.sender.broadcast(event).await.ok();
    }
}
impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}
