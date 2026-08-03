//! Root GraphQL schema (MVP operations grow here).

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Liveness probe for clients and load balancers.
    async fn health(&self, _ctx: &Context<'_>) -> String {
        "ok".to_string()
    }
}

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_returns_ok() {
        let schema = build_schema();
        let response = schema.execute("{ health }").await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let data = response.data.into_json().expect("json");
        assert_eq!(data["health"], "ok");
    }
}
