use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Custom error type for the application
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
}

// Convert AppError to an HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = format!("Error: {}", error_message);

        (status, body).into_response()
    }
}

// Type alias for using Result with our custom error type
type Result<T> = std::result::Result<T, AppError>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "turserver=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initializing turserver");

    // Create router with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/{path}", get(path_handler))
        .layer(TraceLayer::new_for_http());

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
async fn path_handler(Path(path): Path<String>) -> Result<String> {
    info!("Request for path: {}", path);

    // This is a placeholder - will be replaced with DB lookup later
    if path == "test" {
        Ok("This is a test file".to_string())
    } else {
        Err(AppError::NotFound(format!("Path '{}' not found", path)))
    }
}