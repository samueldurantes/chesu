pub mod auth;
pub mod error;
pub mod extractor;

pub use error::{Error, ResultExt};

pub type Result<T, E = Error> = std::result::Result<T, E>;
