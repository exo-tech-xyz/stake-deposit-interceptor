use error::ApiError;

pub mod error;
pub mod utils;
pub mod router;

pub type Result<T> = std::result::Result<T, ApiError>;
