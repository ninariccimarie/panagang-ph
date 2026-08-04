//! Panagang PH API — device identity and scam report intake.

mod config;
mod device;
mod graphql;
#[allow(dead_code)]
mod jobs;
#[allow(dead_code)]
mod llm;
mod migrate;
mod report;
mod shared;

use std::net::SocketAddr;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
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
                .unwrap_or_else(|_| EnvFilter::new("panagang_api=info,tower_http=info,sqlx=warn")),
        )
        .init();

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to Postgres");

    migrate::run(&pool)
        .await
        .expect("failed to run database migrations");

    let addr = SocketAddr::from((config.host, config.port));
    let app = Router::new()
        .merge(graphql_router(pool, config))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    tracing::info!("listening on http://{addr}");
    tracing::info!("GraphQL endpoint: http://{addr}/graphql");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind API address");

    axum::serve(listener, app).await.expect("server error");
}
