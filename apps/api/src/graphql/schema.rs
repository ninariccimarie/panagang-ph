//! Root GraphQL schema.

use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use sqlx::PgPool;

use crate::config::Config;
use crate::graphql::auth::AuthContext;
use crate::graphql::types::{parse_uuid, RegisterDevicePayload, ScamReport, SubmitScamReportInput};
use crate::services::{DeviceService, ReportService, SubmitReportInput};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    /// Liveness probe for clients and load balancers.
    async fn health(&self) -> &str {
        "ok"
    }

    /// Reports submitted by the authenticated device (newest first).
    async fn my_reports(&self, ctx: &Context<'_>) -> Result<Vec<ScamReport>> {
        let device = require_device(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let reports = ReportService::new(pool)
            .list_for_device(device.id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(reports.into_iter().map(ScamReport::from).collect())
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

    /// Submit a scam/call report. Requires `Authorization: Bearer <device token>`.
    async fn submit_scam_report(
        &self,
        ctx: &Context<'_>,
        input: SubmitScamReportInput,
    ) -> Result<ScamReport> {
        let device = require_device(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let report = ReportService::new(pool)
            .submit(
                device.id,
                SubmitReportInput {
                    country_code: input.country_code,
                    phone_number: input.phone_number,
                    sms_content: input.sms_content,
                },
            )
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(ScamReport::from(report))
    }
}

fn require_device<'a>(ctx: &'a Context<'_>) -> Result<&'a crate::models::Device> {
    let auth = ctx.data::<AuthContext>()?;
    auth.device
        .as_ref()
        .ok_or_else(|| async_graphql::Error::new("unauthorized: missing or invalid device token"))
}

pub fn build_schema_with_data(pool: PgPool, config: Config) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .data(config)
        .finish()
}

