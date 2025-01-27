pub mod client;
pub mod error;
pub mod extractor;

pub use error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
