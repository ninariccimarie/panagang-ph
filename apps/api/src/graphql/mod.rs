//! GraphQL schema and Axum routes.

mod auth;
mod schema;
mod types;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Extension, Router};
use sqlx::PgPool;

use crate::config::Config;
use crate::graphql::auth::{authenticate_from_headers, AuthContext};
use crate::graphql::schema::{build_schema_with_data, AppSchema};

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(
    schema: Extension<AppSchema>,
    pool: Extension<PgPool>,
    config: Extension<Config>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let auth = authenticate_from_headers(&headers, &pool, &config.device_token_secret).await;
    let mut request = req.into_inner();
    request = request.data(AuthContext { device: auth });
    schema.execute(request).await.into()
}

pub fn graphql_router(pool: PgPool, config: Config) -> Router {
    let schema = build_schema_with_data(pool.clone(), config.clone());
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .route("/health", get(|| async { "ok" }))
        .layer(Extension(schema))
        .layer(Extension(pool))
        .layer(Extension(config))
}
