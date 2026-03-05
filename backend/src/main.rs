mod config;

use axum::Router;
use axum::routing::get;
use tower_http::cors::CorsLayer;

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

/// A handler for handling the most basic request possible from the frontend,
/// which verifies if the backend is correctly set up and can respond to requests from the frontend.
///
/// ## Args
/// - None
///
/// ## Returns
/// - A static string `Hello frontend! This is Rust. Nou2X^mZwMq!4F$t` as the response body,
///     which can be used to verify that the backend is correctly set up
///     and can respond to requests from the frontend.
///
/// ## Method
/// - `GET /vue-connect` - Gets a message confirming the connection between the frontend and backend.
async fn vue_connect_handler() -> &'static str {
    "Hello frontend! This is Rust. Nou2X^mZwMq!4F$t"
}

#[tokio::main]
async fn main() {
    // Get Configuration
    let config = config::Config::new().expect("Failed to load configuration.");

    // Initialize Tracing
    tracing_subscriber::fmt::init();

    // Build Service Router
    let service = Router::new()
        .route("/", get(root))
        .route("/vue-connect", get(vue_connect_handler))
        .layer(CorsLayer::permissive());

    // Build the TCP listener for the service and start serving the application.
    let listener = tokio::net::TcpListener::bind(config.server.get_addr())
        .await
        .unwrap();
    tracing::info!("Server is running on http://{}", config.server.get_addr());
    // It is safe to unwrap, as per the documentation,
    // "Although this future resolves to io::Result<()>, it will never actually complete or return an error".
    axum::serve(listener, service).await.unwrap()
}
