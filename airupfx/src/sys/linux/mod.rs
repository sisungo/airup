#[cfg(feature = "power")]
pub mod power;

#[cfg(feature = "power")]
pub use power::Linux as PowerManager;