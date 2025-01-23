pub mod error;
pub mod extractor;
pub mod game;
pub mod user;
pub mod wallet;

pub use error::{Error, ResultExt};

pub type Result<T, E = Error> = std::result::Result<T, E>;
