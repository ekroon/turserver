use std::env;
use std::path::Path;
use std::sync::Arc;

use libsql::{Builder, Database, Value, params};
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::db::error::{DbError, DbResult};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Database URL, can be local file path or Turso URL
    pub url: String,
    /// Auth token for Turso cloud database
    pub auth_token: Option<String>,
    /// Replica configuration (for embedded replication)
    pub replica: Option<ReplicaConfig>,
}

/// Configuration for embedded replicas
#[derive(Debug, Clone)]
pub struct ReplicaConfig {
    /// Primary database URL to sync from
    pub primary_url: String,
    /// Auth token for the primary database
    pub auth_token: String,
    /// Local replica file path
    pub local_path: String,
}

impl DbConfig {
    /// Load database configuration from environment variables
    pub fn from_env() -> DbResult<Self> {
        // Try to load from .env file if present, but don't fail if not
        let _ = dotenv::dotenv();

        // Get base database URL
        let url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            debug!("DATABASE_URL not set, using default local database");
            "./turserver.db".to_string()
        });

        // Get auth token for cloud database
        let auth_token = env::var("DATABASE_AUTH_TOKEN").ok();

        // Check for replica configuration
        let replica = if env::var("USE_REPLICA").unwrap_or_default() == "true" {
            // When using a replica, we need both primary URL and local path
            let primary_url = env::var("PRIMARY_DATABASE_URL").map_err(|_| {
                DbError::Configuration(
                    "PRIMARY_DATABASE_URL must be set when USE_REPLICA=true".into(),
                )
            })?;

            let auth_token = env::var("DATABASE_AUTH_TOKEN").map_err(|_| {
                DbError::Configuration(
                    "DATABASE_AUTH_TOKEN must be set when USE_REPLICA=true".into(),
                )
            })?;

            let local_path = env::var("REPLICA_LOCAL_PATH").unwrap_or_else(|_| {
                debug!("REPLICA_LOCAL_PATH not set, using ./replica.db");
                "./replica.db".to_string()
            });

            Some(ReplicaConfig {
                primary_url,
                auth_token,
                local_path,
            })
        } else {
            None
        };

        Ok(DbConfig {
            url,
            auth_token,
            replica,
        })
    }

    /// Check if this is a local database configuration
    pub fn is_local(&self) -> bool {
        !self.url.starts_with("libsql://")
            && !self.url.starts_with("http://")
            && !self.url.starts_with("https://")
    }

    /// Check if this is using embedded replication
    pub fn is_replica(&self) -> bool {
        self.replica.is_some()
    }
}

/// Database connection pool
pub type DbPool = Arc<Mutex<Database>>;

/// Create a new database connection pool
pub async fn create_pool() -> DbResult<DbPool> {
    let config = DbConfig::from_env()?;

    info!("Initializing database connection");
    debug!("Database config: {:?}", config);

    let db = if config.is_replica() {
        debug!("Setting up embedded replica database");
        setup_embedded_replica(&config).await?
    } else if config.is_local() {
        debug!("Using local database at: {}", config.url);
        // For local SQLite file
        Builder::new_local(&config.url)
            .build()
            .await
            .map_err(|e| DbError::Connection(format!("Failed to build local database: {}", e)))?
    } else {
        debug!("Using remote Turso database at: {}", config.url);
        let auth_token = config.auth_token.ok_or_else(|| {
            DbError::Configuration(
                "DATABASE_AUTH_TOKEN is required for remote Turso database".into(),
            )
        })?;

        // Create a remote connection
        Builder::new_remote(config.url.clone(), auth_token.clone())
            .build()
            .await
            .map_err(|e| DbError::Connection(format!("Failed to build remote database: {}", e)))?
    };

    // Test the connection with a basic query
    let conn = db
        .connect()
        .map_err(|e| DbError::Connection(format!("Failed to connect to database: {}", e)))?;

    // Execute a test query to ensure the database is operational
    if let Err(e) = conn.execute("SELECT 1", params![]).await {
        if e.to_string() != "Execute returned rows" {
            return Err(DbError::Connection(format!(
                "Failed to execute test query: {}",
                e
            )));
        }
    }

    info!("Database connection successful");

    Ok(Arc::new(Mutex::new(db)))
}

/// Set up an embedded replica database that syncs with a Turso cloud database
async fn setup_embedded_replica(config: &DbConfig) -> DbResult<Database> {
    let replica_config = config.replica.as_ref().ok_or_else(|| {
        DbError::Configuration(
            "Replica configuration is required for embedded replica setup".into(),
        )
    })?;

    info!(
        "Setting up embedded replica from {}",
        replica_config.primary_url
    );
    debug!("Replica local path: {}", replica_config.local_path);

    // Make sure the directory exists for the replica
    if let Some(parent) = Path::new(&replica_config.local_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DbError::Configuration(format!("Failed to create directory for replica: {}", e))
            })?;
        }
    }

    // Create the replica database with synchronization
    let builder = Builder::new_synced_database(
        &replica_config.local_path,
        replica_config.primary_url.clone(),
        replica_config.auth_token.clone(),
    );

    let db = builder
        .build()
        .await
        .map_err(|e| DbError::Connection(format!("Failed to build replica database: {}", e)))?;

    // Test the connection and trigger an initial sync
    let conn = db.connect().map_err(|e| {
        DbError::Connection(format!("Failed to connect to replica database: {}", e))
    })?;

    // Execute a query to test and trigger sync
    let sync_result = conn.execute("SELECT 1", params![]).await;
    if let Err(e) = &sync_result {
        debug!("Initial sync had issues: {}", e);
    }

    info!("Embedded replica setup successful");

    Ok(db)
}

/// Check database connection health
pub async fn check_connection(pool: &DbPool) -> DbResult<()> {
    let db = pool.lock().await;

    let conn = db
        .connect()
        .map_err(|e| DbError::Connection(format!("Failed to connect to database: {}", e)))?;

    conn.execute("SELECT 1", params![])
        .await
        .map_err(|e| DbError::Connection(format!("Failed to execute test query: {}", e)))?;

    Ok(())
}

/// Execute a single SQL query and return the rows
#[allow(dead_code)]
pub async fn execute_query(pool: &DbPool, query: &str) -> DbResult<libsql::Statement> {
    let db = pool.lock().await;

    let conn = db
        .connect()
        .map_err(|e| DbError::Connection(format!("Failed to connect to database: {}", e)))?;

    conn.prepare(query)
        .await
        .map_err(|e| DbError::Query(format!("Query preparation error: {}", e)))
}

/// Execute a parameterized SQL query
#[allow(dead_code)]
pub async fn execute_parameterized_query(
    pool: &DbPool,
    query: &str,
    _params: &[Value],
) -> DbResult<libsql::Statement> {
    let db = pool.lock().await;

    let conn = db
        .connect()
        .map_err(|e| DbError::Connection(format!("Failed to connect to database: {}", e)))?;

    conn.prepare(query)
        .await
        .map_err(|e| DbError::Query(format!("Query preparation error: {}", e)))
}
