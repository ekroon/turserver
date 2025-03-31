pub mod connection;
pub mod error;
pub mod migrations;

// Test modules
#[cfg(test)]
mod connection_tests;
#[cfg(test)]
mod error_tests;

pub use connection::DbPool;
pub use error::{DbError, DbResult};

/// Initialize the database connection pool
pub async fn init() -> DbResult<DbPool> {
    connection::create_pool().await
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> DbResult<()> {
    migrations::run_migrations(pool).await
}

/// Add a test file to the database (for development purposes)
#[allow(dead_code)]
pub async fn add_test_file(pool: &DbPool) -> DbResult<()> {
    migrations::add_test_file(pool).await
}

/// Health check for the database connection
pub async fn health_check(pool: &DbPool) -> DbResult<()> {
    connection::check_connection(pool).await
}
