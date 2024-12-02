use error::ApiError;

pub mod error;
pub mod router;
pub mod utils;

pub type Result<T> = std::result::Result<T, ApiError>;
