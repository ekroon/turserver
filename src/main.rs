use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import our db module
mod db;

// Custom error type for the application
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] db::DbError),
}

// Convert AppError to an HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = format!("Error: {}", error_message);

        (status, body).into_response()
    }
}

// Type alias for using Result with our custom error type
type Result<T> = std::result::Result<T, AppError>;

// Application state that will be shared with handlers
#[derive(Clone)]
struct AppState {
    db_pool: db::DbPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "turserver=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initializing turserver");

    // Initialize database
    info!("Initializing database");
    let db_pool = db::init().await.map_err(|e| {
        error!("Failed to initialize database: {}", e);
        anyhow::anyhow!("Database initialization error: {}", e)
    })?;

    // Run database migrations to ensure schema is up to date
    info!("Running database migrations");
    db::run_migrations(&db_pool).await.map_err(|e| {
        error!("Failed to run database migrations: {}", e);
        anyhow::anyhow!("Database migration error: {}", e)
    })?;

    // Create application state
    let state = AppState { db_pool };

    // Create router with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/{path}", get(path_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Set up the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await.map_err(|e| {
        error!("Server error: {}", e);
        anyhow::anyhow!("Server error: {}", e)
    })?;

    Ok(())
}

// Handler for the root path
async fn root_handler() -> &'static str {
    "Turserver - File server powered by Turso"
}

// Handler for other paths - will be replaced with database lookup logic later
async fn path_handler(State(state): State<AppState>, Path(path): Path<String>) -> Result<String> {
    info!("Request for path: {}", path);

    // Perform health check to ensure the database is working
    db::health_check(&state.db_pool).await?;

    // This is a placeholder - will be replaced with DB lookup later
    if path == "test" {
        Ok("This is a test file".to_string())
    } else {
        Err(AppError::NotFound(format!("Path '{}' not found", path)))
    }
}
