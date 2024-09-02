use std::{net::SocketAddr, process::Command};

use axum::Router;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use axum::routing::get;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug, tower_http=debug", env!("CARGO_CRATE_NAME")).into()
        }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

    tokio::join!(
        serve(using_serve_dir(), 3001),
    );
    Ok(())
}

fn using_serve_dir() -> Router {
    Router::new()
    .route("/archive_db", get(make_archive))
    .nest_service("/static", ServeDir::new("static"))
}

async fn make_archive() -> &'static str {
    let current_dir = env::current_dir().unwrap();
    let file_dir = format!("{}\\static\\",current_dir.display());
    if let Err(_) = Command::new("7z")
        .args(["a", "-tzip", "static/db.zip", &file_dir])
        .output() {
            "Archiving failed"
        } else {
            "Compression successfull"
        }
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127,0,0,1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listeneing on {}",listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}