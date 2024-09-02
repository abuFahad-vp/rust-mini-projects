use axum::{response::IntoResponse, Extension, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::{net::SocketAddr, process::Command};
use tower_http::{services::ServeDir, trace::TraceLayer};
use std::env;
use axum::routing::get;

use super::SharedChain;

pub async fn blockchain_app_run(chain: SharedChain, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug, tower_http=debug", env!("CARGO_CRATE_NAME")).into()
        }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

    tokio::join!(
        serve(api_end_points(chain), port),
    );
    Ok(())
}

fn api_end_points(chain: SharedChain) -> Router {
    Router::new()
    .route("/archive_db", get(make_archive))
    .route("/len", get(get_len))
    .nest_service("/static", ServeDir::new("static"))
    .layer(Extension(chain))
}

async fn get_len(Extension(chain): Extension<SharedChain>) -> impl IntoResponse {
    let chain = chain.lock().await;
    format!("Height = {}",chain.get_height())
}

// make an archive of db and copy to static folder
async fn make_archive() -> &'static str {
    let current_dir = env::current_dir().unwrap();
    let file_dir = format!("{}\\amanah.db\\",current_dir.display());
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