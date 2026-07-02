use axum::{routing::get_service, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn axum_init() {
    // Initialize tracing (for logging)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_file_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // --- Configuration ---
    // The URL path prefix where files will be served
    let serve_path_prefix = "/soft";
    // The actual directory path on your filesystem.
    // IMPORTANT: Use double backslashes on Windows or raw strings.
    let directory_to_serve = r"D:\Software"; // Using raw string literal
                                             // Alternatively: "D:\\Software";

    // Create the ServeDir service
    // ServeDir::new will handle serving individual files AND
    // generating directory listings when a directory is requested.
    let serve_dir_service = ServeDir::new(directory_to_serve)
        // Optional: Customize behavior if a file is not found under this path
        .append_index_html_on_directories(false); // Set to true if you want to serve index.html instead of listing

    // Create the Axum router
    // Use `nest_service` to map the `serve_path_prefix` to the `ServeDir` service.
    // Any request starting with `/soft` will be handled by ServeDir,
    // looking for files relative to `directory_to_serve`.
    let app = Router::new().nest_service(serve_path_prefix, get_service(serve_dir_service));

    // Define the address to listen on
    let addr = SocketAddr::from(([0, 0, 0, 0], 8654)); // Listen on all interfaces, port 3000
    tracing::debug!(
        "Serving files from '{}' at http://{}{}",
        directory_to_serve,
        addr,
        serve_path_prefix
    );
    println!("Listening on http://{}", addr);
    println!("Serving files from directory: {}", directory_to_serve);
    println!(
        "Access files under: http://{}:{}{}/",
        "127.0.0.1",
        addr.port(),
        serve_path_prefix
    );

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
