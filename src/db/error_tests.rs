use crate::db::error::DbError;
use anyhow::anyhow;

#[test]
fn test_db_error_display() {
    let connection_error = DbError::Connection("test connection error".to_string());
    assert_eq!(
        connection_error.to_string(),
        "Database connection error: test connection error"
    );

    let query_error = DbError::Query("test query error".to_string());
    assert_eq!(
        query_error.to_string(),
        "Database query error: test query error"
    );

    let config_error = DbError::Configuration("test config error".to_string());
    assert_eq!(
        config_error.to_string(),
        "Database configuration error: test config error"
    );

    let init_error = DbError::Initialization("test init error".to_string());
    assert_eq!(
        init_error.to_string(),
        "Database initialization error: test init error"
    );
}

#[test]
fn test_db_error_from_anyhow() {
    let anyhow_error = anyhow!("test anyhow error");
    let db_error: DbError = anyhow_error.into();

    assert!(matches!(db_error, DbError::Query(_)));
    assert_eq!(
        db_error.to_string(),
        "Database query error: test anyhow error"
    );
}

// We can't easily test the From<libsql::Error> implementation directly in unit tests
// since libsql::Error doesn't have public constructors. This would be better
// covered in integration tests with a real database connection.
