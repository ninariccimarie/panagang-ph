//! Root GraphQL schema.

use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use sqlx::PgPool;

use crate::config::Config;
use crate::graphql::types::{parse_uuid, RegisterDevicePayload};
use crate::services::DeviceService;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    /// Liveness probe for clients and load balancers.
    async fn health(&self) -> &str {
        "ok"
    }
}

#[Object]
impl MutationRoot {
    /// Register an anonymous device and receive a bearer token (shown once).
    async fn register_device(&self, ctx: &Context<'_>) -> Result<RegisterDevicePayload> {
        let pool = ctx.data::<PgPool>()?;
        let config = ctx.data::<Config>()?;
        let registered = DeviceService::new(pool, &config.device_token_secret)
            .register()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(RegisterDevicePayload {
            device_id: parse_uuid(&registered.device.id),
            token: registered.token,
        })
    }
}

pub fn build_schema_with_data(pool: PgPool, config: Config) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .data(config)
        .finish()
}
