use crate::db::connection::DbConfig;

#[test]
fn test_db_config_local_detection() {
    let config = DbConfig {
        url: "./local.db".to_string(),
        auth_token: None,
        replica: None,
    };
    assert!(config.is_local(), "Local path should be detected as local");

    let remote_config = DbConfig {
        url: "libsql://example.turso.io".to_string(),
        auth_token: Some("token".to_string()),
        replica: None,
    };
    assert!(
        !remote_config.is_local(),
        "Remote URL should not be detected as local"
    );
}

#[test]
fn test_db_config_replica_detection() {
    let config = DbConfig {
        url: "./local.db".to_string(),
        auth_token: None,
        replica: None,
    };
    assert!(
        !config.is_replica(),
        "Should not be detected as replica without replica config"
    );

    let config_with_replica = DbConfig {
        url: "./local.db".to_string(),
        auth_token: None,
        replica: Some(crate::db::connection::ReplicaConfig {
            primary_url: "libsql://primary.turso.io".to_string(),
            auth_token: "token".to_string(),
            local_path: "./replica.db".to_string(),
        }),
    };
    assert!(
        config_with_replica.is_replica(),
        "Should be detected as replica with replica config"
    );
}
