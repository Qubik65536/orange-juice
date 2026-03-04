mod config;

use axum::Router;
use axum::routing::get;

/// A simple "Hello, World!" handler for the root path that responds with a static string.
///
/// ## Args
/// - None
///
/// ## Returns
/// - A static string "Hello, World!" as the response body.
///
/// ## Method
/// - `GET /` - Gets the greeting message.
async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    // Get Configuration
    let config = config::Config::new().expect("Failed to load configuration.");

    // Initialize Tracing
    tracing_subscriber::fmt::init();

    // Build Service Router
    let service = Router::new()
        .route("/", get(root));

    // Build the TCP listener for the service and start serving the application.
    let listener = tokio::net::TcpListener::bind(config.server.get_addr())
        .await
        .unwrap();
    tracing::info!("Server is running on http://{}", config.server.get_addr());
    // It is safe to unwrap, as per the documentation,
    // "Although this future resolves to io::Result<()>, it will never actually complete or return an error".
    axum::serve(listener, service).await.unwrap()
}
