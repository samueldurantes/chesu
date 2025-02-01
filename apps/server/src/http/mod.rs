mod client;
pub use client::*;

mod error;
pub use error::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;
