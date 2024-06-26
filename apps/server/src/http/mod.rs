pub mod auth;
pub mod docs;
pub mod error;
pub mod extractor;
pub mod game;
pub mod user;

pub use error::{Error, ResultExt};

pub type Result<T, E = Error> = std::result::Result<T, E>;
