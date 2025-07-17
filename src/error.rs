// src/error.rs

/// A common Result type for the application.
/// It uses anyhow::Error to allow for flexible error handling.
pub type AppResult<T> = Result<T, anyhow::Error>;
