//! Inspection and manipulation of the system's lifetime.

use airupfx::prelude::*;
use tokio::sync::broadcast;

/// Airupd's lifetime manager.
#[derive(Debug)]
pub struct System(broadcast::Sender<Event>);
impl System {
    /// Creates a new instance with default settings.
    pub fn new() -> Self {
        Self(broadcast::channel(1).0)
    }

    /// Creates a new [`broadcast::Receiver`] handle that will receive events sent after this call to `subscribe`.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.0.subscribe()
    }

    /// Makes `airupd` exit.
    pub fn exit(&self, code: i32) {
        self.send(Event::Exit(code));
    }

    /// Powers the device off.
    pub fn poweroff(&self) {
        self.send(Event::PowerOff);
    }

    /// Reboots the device.
    pub fn reboot(&self) {
        self.send(Event::Reboot);
    }

    /// Halts the device.
    pub fn halt(&self) {
        self.send(Event::Halt);
    }

    /// Reboots the system's userspace.
    pub fn userspace_reboot(&self) {
        self.send(Event::UserspaceReboot);
    }

    /// Sends an process-wide lifetime event.
    fn send(&self, event: Event) {
        self.0.send(event).ok();
    }
}
impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

/// An event related to Airupd's lifetime.
#[derive(Debug, Clone)]
pub enum Event {
    /// Makes `airupd` exit.
    Exit(i32),

    /// Powers the device off.
    PowerOff,

    /// Reboots the device.
    Reboot,

    /// Halts the device.
    Halt,

    /// Reboots the system's userspace.
    UserspaceReboot,
}
impl Event {
    /// Handles the event.
    pub async fn handle(&self) -> ! {
        _ = match self {
            Self::Exit(code) => std::process::exit(*code),
            Self::PowerOff => power_manager().poweroff().await,
            Self::Reboot => power_manager().reboot().await,
            Self::Halt => power_manager().halt().await,
            Self::UserspaceReboot => power_manager().userspace().await,
        };

        std::process::exit(1);
    }
}
