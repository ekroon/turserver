use tracing::{debug, info};

use crate::db::DbPool;
use crate::db::error::{DbError, DbResult};

/// Run all necessary migrations to bring the database schema up to date
pub async fn run_migrations(pool: &DbPool) -> DbResult<()> {
    info!("Running database migrations");

    let mut db = pool.acquire().await.map_err(|e| {
        DbError::Connection(format!("Failed to acquire database connection: {}", e))
    })?;

    // Create files table if it doesn't exist
    debug!("Creating files table if it doesn't exist");
    create_files_table(&mut db).await?;

    info!("Database migrations completed successfully");
    Ok(())
}

/// Create the files table if it doesn't exist
async fn create_files_table(conn: &mut sqlx::SqliteConnection) -> DbResult<()> {
    let create_table_sql = r#"
    CREATE TABLE IF NOT EXISTS files (
        id TEXT PRIMARY KEY,
        path TEXT NOT NULL UNIQUE,
        content BLOB NOT NULL,
        content_type TEXT NOT NULL,
        size INTEGER NOT NULL,
        last_modified INTEGER NOT NULL,
        created_at INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
    "#;

    sqlx::query(create_table_sql)
        .execute(conn)
        .await
        .map_err(|e| DbError::Query(format!("Failed to create files table: {}", e)))?;

    debug!("Files table created or already exists");
    Ok(())
}

/// Add a test file to the database (for development purposes)
#[allow(dead_code)]
pub async fn add_test_file(pool: &DbPool) -> DbResult<()> {
    debug!("Adding test file to database");

    // Create a sample test file
    let insert_sql = r#"
    INSERT OR REPLACE INTO files (
        id, path, content, content_type, size, last_modified, created_at
    ) VALUES (
        'test-file',
        'test.txt',
        X'54686973206973206A75737420612074657374206669',  -- "This is just a test fi" in hex
        'text/plain',
        20,
        strftime('%s', 'now'),
        strftime('%s', 'now')
    );
    "#;

    sqlx::query(insert_sql)
        .execute(pool)
        .await
        .map_err(|e| DbError::Query(format!("Failed to insert test file: {}", e)))?;

    debug!("Test file added to database");
    Ok(())
}
