use crate::db::connection::{DbPool, execute_parameterized_query, execute_query};

#[tokio::test]
async fn test_execute_query() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();
    sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT);")
        .execute(&pool)
        .await
        .unwrap();

    execute_query(&pool, "INSERT INTO test (value) VALUES ('test_value');")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_execute_parameterized_query() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();
    sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT);")
        .execute(&pool)
        .await
        .unwrap();

    execute_parameterized_query(
        &pool,
        "INSERT INTO test (id, value) VALUES (?, ?);",
        (1, "test_value"),
    )
    .await
    .unwrap();
}
