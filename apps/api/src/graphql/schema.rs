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

#[cfg(test)]
mod tests {
    use async_graphql::Request;
    use sqlx::PgPool;

    use super::*;
    use crate::graphql::auth::AuthContext;
    use crate::services::DeviceService;

    fn test_config() -> Config {
        Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 8080,
            database_url: "postgres://unused".into(),
            device_token_secret: "test-secret".into(),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn health_returns_ok(pool: PgPool) {
        let schema = build_schema_with_data(pool, test_config());
        let response = schema
            .execute(Request::new("{ health }").data(AuthContext { device: None }))
            .await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let data = response.data.into_json().expect("json");
        assert_eq!(data["health"], "ok");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn register_submit_and_list_reports(pool: PgPool) {
        let config = test_config();
        let schema = build_schema_with_data(pool.clone(), config.clone());

        let registered = schema
            .execute(
                Request::new(
                    r#"
                    mutation {
                      registerDevice {
                        deviceId
                        token
                      }
                    }
                    "#,
                )
                .data(AuthContext { device: None }),
            )
            .await;
        assert!(registered.errors.is_empty(), "{:?}", registered.errors);
        let reg = registered.data.into_json().unwrap();
        let token = reg["registerDevice"]["token"].as_str().unwrap().to_string();

        let device = DeviceService::new(&pool, &config.device_token_secret)
            .authenticate(&token)
            .await
            .expect("auth");

        let submitted = schema
            .execute(
                Request::new(
                    r#"
                    mutation {
                      submitScamReport(input: {
                        countryCode: "+63"
                        phoneNumber: "9171234567"
                        smsContent: "Congrats winner claim prize"
                      }) {
                        phoneE164
                        countryCode
                        nationalNumber
                        smsContent
                      }
                    }
                    "#,
                )
                .data(AuthContext {
                    device: Some(device.clone()),
                }),
            )
            .await;
        assert!(submitted.errors.is_empty(), "{:?}", submitted.errors);
        let report = submitted.data.into_json().unwrap();
        assert_eq!(report["submitScamReport"]["phoneE164"], "+639171234567");
        assert_eq!(report["submitScamReport"]["nationalNumber"], "9171234567");

        let listed = schema
            .execute(
                Request::new("{ myReports { phoneE164 smsContent } }").data(AuthContext {
                    device: Some(device),
                }),
            )
            .await;
        assert!(listed.errors.is_empty(), "{:?}", listed.errors);
        let rows = listed.data.into_json().unwrap();
        assert_eq!(rows["myReports"].as_array().unwrap().len(), 1);
        assert_eq!(rows["myReports"][0]["phoneE164"], "+639171234567");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn my_reports_requires_auth(pool: PgPool) {
        let schema = build_schema_with_data(pool, test_config());
        let response = schema
            .execute(Request::new("{ myReports { id } }").data(AuthContext { device: None }))
            .await;
        assert!(!response.errors.is_empty());
        assert!(response.errors[0].message.contains("unauthorized"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn submit_requires_auth(pool: PgPool) {
        let schema = build_schema_with_data(pool, test_config());
        let response = schema
            .execute(
                Request::new(
                    r#"
                    mutation {
                      submitScamReport(input: {
                        countryCode: "+63"
                        phoneNumber: "9171234567"
                      }) { id }
                    }
                    "#,
                )
                .data(AuthContext { device: None }),
            )
            .await;
        assert!(!response.errors.is_empty());
        assert!(response.errors[0].message.contains("unauthorized"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn submit_rejects_invalid_phone(pool: PgPool) {
        let config = test_config();
        let schema = build_schema_with_data(pool.clone(), config.clone());
        let device = DeviceService::new(&pool, &config.device_token_secret)
            .register()
            .await
            .expect("register")
            .device;

        let response = schema
            .execute(
                Request::new(
                    r#"
                    mutation {
                      submitScamReport(input: {
                        countryCode: "+63"
                        phoneNumber: "123"
                      }) { id }
                    }
                    "#,
                )
                .data(AuthContext {
                    device: Some(device),
                }),
            )
            .await;
        assert!(!response.errors.is_empty());
        assert!(response.errors[0].message.contains("invalid phone"));
    }
}
