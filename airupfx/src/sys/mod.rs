#[cfg(target_family = "unix")]
pub(crate) mod unix;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::*;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
        pub use macos::*;
    } else if #[cfg(any(target_os = "freebsd"))] {
        mod freebsd;
        pub use freebsd::*;
    } else if #[cfg(target_family = "unix")] {
        pub use unix::*;
    } else if #[cfg(target_family = "windows")] {
        mod windows;
        pub use windows::*;
    } else {
        std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
    }
}
