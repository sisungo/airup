//! Port of unstable `std` features.

/// Port of unstable feature `#[feature(result_option_inspect)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait ResultExt<T, E> {
    fn inspect_err<F: FnOnce(&E)>(self, op: F) -> Self;
}
impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn inspect_err<F: FnOnce(&E)>(self, op: F) -> Self {
        self.map_err(|e| {
            op(&e);
            e
        })
    }
}

/// Port of unstable feature `#[feature(result_option_inspect)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait OptionExt<T> {
    fn inspect_none<F: FnOnce()>(self, op: F) -> Self;
}
impl<T> OptionExt<T> for Option<T> {
    fn inspect_none<F: FnOnce()>(self, op: F) -> Self {
        if self.is_none() {
            op()
        }

        self
    }
}