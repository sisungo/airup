//! Inspection and manipulation of the system's lifetime.

use airupfx::prelude::*;
use tokio::sync::broadcast;

/// Airupd's lifetime manager.
#[derive(Debug)]
pub struct System(broadcast::Sender<Event>);
impl System {
    /// Creates a new instance with default settings.
    pub fn new() -> Self {
        Self(broadcast::channel(2).0)
    }

    /// Creates a new [broadcast::Receiver] handle that will receive events sent after this call to `subscribe`.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.0.subscribe()
    }

    /// Makes `airupd` exit.
    pub fn exit(&self, code: i32) {
        self.send(Event::Exit(code));
    }

    /// Powers the device off.
    pub fn poweroff(&self) {
        self.send(Event::Poweroff);
    }

    /// Reboots the device.
    pub fn reboot(&self) {
        self.send(Event::Reboot);
    }

    /// Halts the device.
    pub fn halt(&self) {
        self.send(Event::Halt);
    }

    /// Reloads `airupd` process image.
    pub fn reload_image(&self) {
        self.send(Event::ReloadImage);
    }

    /// Sends an Airupd lifetime event.
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
    Poweroff,

    /// Reboots the device.
    Reboot,

    /// Halts the device.
    Halt,

    /// Reloads `airupd` process image.
    ReloadImage,
}
impl Event {
    /// Handles the event.
    pub fn handle(&self) -> ! {
        match self {
            Self::Exit(code) => std::process::exit(*code),
            Self::Poweroff => power_manager().poweroff().unwrap_log("poweroff() failed"),
            Self::Reboot => power_manager().reboot().unwrap_log("reboot() failed"),
            Self::Halt => power_manager().halt().unwrap_log("halt() failed"),
            Self::ReloadImage => {
                airupfx::process::reload_image().unwrap_log("reload_image() failed")
            }
        };

        unreachable!()
    }
}
