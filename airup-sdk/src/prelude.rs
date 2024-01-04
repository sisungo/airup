//! The Airup SDK prelude

pub use crate::error::IntoApiError;
pub use crate::system::{QueryService, QuerySystem, Status};

cfg_if::cfg_if! {
    if #[cfg(feature = "nonblocking")] {
        pub use crate::nonblocking::Connection;
        pub use crate::nonblocking::{info::ConnectionExt as _, system::ConnectionExt as _};
        pub use crate::nonblocking::fs::DirChain;
        pub use crate::nonblocking::files::*;
    }
}
