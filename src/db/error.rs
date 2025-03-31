use thiserror::Error;

/// Database specific errors
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Database query error: {0}")]
    Query(String),

    #[error("Database configuration error: {0}")]
    Configuration(String),

    #[error("Database initialization error: {0}")]
    #[allow(dead_code)]
    Initialization(String),
}

/// Result type for database operations
pub type DbResult<T> = std::result::Result<T, DbError>;

// Conversion from anyhow error to our custom DbError
impl From<anyhow::Error> for DbError {
    fn from(error: anyhow::Error) -> Self {
        DbError::Query(error.to_string())
    }
}

// Conversion from libsql error to our custom DbError
impl From<libsql::Error> for DbError {
    fn from(error: libsql::Error) -> Self {
        // In libsql 0.9.1, Error doesn't have variants, it's a wrapper around io::Error
        // We'll map all libsql errors to query errors
        DbError::Query(error.to_string())
    }
}
