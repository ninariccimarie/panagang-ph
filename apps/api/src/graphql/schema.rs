//! Root GraphQL schema.

use async_graphql::{Context, EmptySubscription, Object, Result, Schema};

use crate::config::Config;
use crate::db::DbPool;
use crate::device::DeviceService;
use crate::graphql::auth::AuthContext;
use crate::graphql::types::{parse_uuid, RegisterDevicePayload, ScamReport, SubmitScamReportInput};
use crate::report::{ReportService, SubmitReportInput};

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
        let pool = ctx.data::<DbPool>()?;
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
        let pool = ctx.data::<DbPool>()?;
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
        let pool = ctx.data::<DbPool>()?;
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

fn require_device<'a>(ctx: &'a Context<'_>) -> Result<&'a crate::device::Device> {
    let auth = ctx.data::<AuthContext>()?;
    auth.device
        .as_ref()
        .ok_or_else(|| async_graphql::Error::new("unauthorized: missing or invalid device token"))
}

pub fn build_schema_with_data(pool: DbPool, config: Config) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .data(config)
        .finish()
}

#[cfg(test)]
mod tests {
    use async_graphql::Request;
    use uuid::Uuid;

    use super::*;
    use crate::db::{create_pool, run_migrations, DbPool};
    use crate::device::DeviceService;
    use crate::graphql::auth::AuthContext;

    fn test_config() -> Config {
        Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 8080,
            database_url: "postgres://unused".into(),
            device_token_secret: "test-secret".into(),
        }
    }

    async fn exec_sql(
        conn: &mut diesel_async::AsyncPgConnection,
        sql: String,
    ) -> Result<usize, diesel::result::Error> {
        diesel_async::RunQueryDsl::execute(diesel::sql_query(sql), conn).await
    }

    /// Create an isolated Postgres database, migrate it, and return a pool.
    /// Caller must drop the pool before `drop_test_database` can succeed.
    async fn setup_test_db() -> (String, String, DbPool) {
        let base_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
        let db_name = format!("panagang_test_{}", Uuid::new_v4().simple());
        let test_url = replace_database_name(&base_url, &db_name);

        {
            let manager = diesel_async::pooled_connection::AsyncDieselConnectionManager::<
                diesel_async::AsyncPgConnection,
            >::new(&base_url);
            let admin_pool = diesel_async::pooled_connection::bb8::Pool::builder()
                .max_size(1)
                .build(manager)
                .await
                .expect("admin pool");
            let mut conn = admin_pool.get().await.expect("admin conn");
            exec_sql(&mut conn, format!("CREATE DATABASE \"{db_name}\""))
                .await
                .expect("create test database");
        }

        run_migrations(&test_url).expect("migrate test database");
        let pool = create_pool(&test_url).await.expect("test pool");
        (base_url, db_name, pool)
    }

    async fn drop_test_database(base_url: &str, db_name: &str) {
        let manager = diesel_async::pooled_connection::AsyncDieselConnectionManager::<
            diesel_async::AsyncPgConnection,
        >::new(base_url);
        let admin_pool = diesel_async::pooled_connection::bb8::Pool::builder()
            .max_size(1)
            .build(manager)
            .await
            .expect("admin pool for drop");
        let mut conn = admin_pool.get().await.expect("admin conn for drop");
        // Terminate leftover connections, then drop.
        let _ = exec_sql(
            &mut conn,
            format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
            ),
        )
        .await;
        exec_sql(&mut conn, format!("DROP DATABASE IF EXISTS \"{db_name}\""))
            .await
            .expect("drop test database");
    }

    fn replace_database_name(database_url: &str, db_name: &str) -> String {
        let mut url = url::Url::parse(database_url).expect("valid DATABASE_URL");
        url.set_path(&format!("/{db_name}"));
        url.to_string()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let (base_url, db_name, pool) = setup_test_db().await;
        let schema = build_schema_with_data(pool.clone(), test_config());
        let response = AppSchema::execute(
            &schema,
            Request::new("{ health }").data(AuthContext { device: None }),
        )
        .await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let data = response.data.into_json().expect("json");
        assert_eq!(data["health"], "ok");
        drop(pool);
        drop_test_database(&base_url, &db_name).await;
    }

    #[tokio::test]
    async fn register_submit_and_list_reports() {
        let (base_url, db_name, pool) = setup_test_db().await;
        let config = test_config();
        let schema = build_schema_with_data(pool.clone(), config.clone());

        let registered = AppSchema::execute(
            &schema,
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

        let submitted = AppSchema::execute(
            &schema,
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

        let listed = AppSchema::execute(
            &schema,
            Request::new("{ myReports { phoneE164 smsContent } }").data(AuthContext {
                device: Some(device),
            }),
        )
        .await;
        assert!(listed.errors.is_empty(), "{:?}", listed.errors);
        let rows = listed.data.into_json().unwrap();
        assert_eq!(rows["myReports"].as_array().unwrap().len(), 1);
        assert_eq!(rows["myReports"][0]["phoneE164"], "+639171234567");

        drop(pool);
        drop_test_database(&base_url, &db_name).await;
    }

    #[tokio::test]
    async fn my_reports_requires_auth() {
        let (base_url, db_name, pool) = setup_test_db().await;
        let schema = build_schema_with_data(pool.clone(), test_config());
        let response = AppSchema::execute(
            &schema,
            Request::new("{ myReports { id } }").data(AuthContext { device: None }),
        )
        .await;
        assert!(!response.errors.is_empty());
        assert!(response.errors[0].message.contains("unauthorized"));
        drop(pool);
        drop_test_database(&base_url, &db_name).await;
    }

    #[tokio::test]
    async fn submit_requires_auth() {
        let (base_url, db_name, pool) = setup_test_db().await;
        let schema = build_schema_with_data(pool.clone(), test_config());
        let response = AppSchema::execute(
            &schema,
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
        drop(pool);
        drop_test_database(&base_url, &db_name).await;
    }

    #[tokio::test]
    async fn submit_rejects_invalid_phone() {
        let (base_url, db_name, pool) = setup_test_db().await;
        let config = test_config();
        let schema = build_schema_with_data(pool.clone(), config.clone());
        let device = DeviceService::new(&pool, &config.device_token_secret)
            .register()
            .await
            .expect("register")
            .device;

        let response = AppSchema::execute(
            &schema,
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
        drop(pool);
        drop_test_database(&base_url, &db_name).await;
    }
}
