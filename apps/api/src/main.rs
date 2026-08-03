//! Panagang PH API — Phase 1 skeleton (`health` GraphQL).

mod config;
mod graphql;
#[allow(dead_code)]
mod jobs;
#[allow(dead_code)]
mod llm;
#[allow(dead_code)]
mod models;
#[allow(dead_code)]
mod repositories;
#[allow(dead_code)]
mod services;

use std::net::SocketAddr;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::graphql::graphql_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("panagang_api=info,tower_http=info")),
        )
        .init();

    let config = Config::from_env();
    let app = Router::new()
        .merge(graphql_router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from((config.host, config.port));
    tracing::info!("listening on http://{addr}");
    tracing::info!("GraphQL endpoint: http://{addr}/graphql");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind API address");

    axum::serve(listener, app).await.expect("server error");
}
