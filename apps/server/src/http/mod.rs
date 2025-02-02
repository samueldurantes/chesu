mod client;
pub use client::*;

mod error;
pub use error::*;

mod config;
pub use config::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;
