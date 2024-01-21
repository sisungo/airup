#[cfg(target_family = "unix")]
pub(crate) mod unix;

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        pub use unix::*;
    } else {
        std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
    }
}
