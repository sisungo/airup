//! The Airup SDK preludes.

pub use crate::error::IntoApiError;
pub use crate::info::ConnectionExt as _;
pub use crate::system::{ConnectionExt as _, QueryService, QuerySystem, Status};

cfg_if::cfg_if! {
    if #[cfg(feature = "nonblocking")] {
        pub use crate::nonblocking::Connection;
        pub use crate::nonblocking::fs::DirChain;
        pub use crate::nonblocking::files::*;
    }
}
