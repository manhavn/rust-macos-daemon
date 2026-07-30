pub mod api;
pub mod static_assets;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

pub async fn start_web_server(host: &str, port: u16, open_browser: bool) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { Html(static_assets::INDEX_HTML) }))
        .route("/api/status", get(api::get_system_status))
        .route(
            "/api/services",
            get(api::list_services).post(api::create_or_update_service),
        )
        .route("/api/services/raw", post(api::save_raw_service))
        .route(
            "/api/services/:label",
            get(api::get_service_detail).delete(api::delete_service),
        )
        .route("/api/services/:label/action", post(api::service_action))
        .route("/api/logs", get(api::get_service_log))
        .layer(cors);

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse()?;

    let url = format!("http://{}", addr_str);
    println!("🚀 Web UI running at {}", url);
    println!("   Privilege Level: {}", if crate::privilege::is_root() { "ROOT (Elevated)" } else { "User Mode" });

    if open_browser {
        let _ = open::that(&url);
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
