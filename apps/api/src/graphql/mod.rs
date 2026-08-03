//! GraphQL schema and Axum routes.

mod schema;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Extension, Router};

use schema::{build_schema, AppSchema};

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub fn graphql_router() -> Router {
    let schema = build_schema();
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .route("/health", get(|| async { "ok" }))
        .layer(Extension(schema))
}
