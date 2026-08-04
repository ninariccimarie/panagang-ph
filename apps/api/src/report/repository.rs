use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::DbPool;
use crate::report::models::{NewReport, Report};
use crate::schema::reports;

pub struct ReportRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> ReportRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    async fn conn(
        &self,
    ) -> Result<diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncPgConnection>, String>
    {
        self.pool
            .get()
            .await
            .map_err(|e| format!("database pool error: {e}"))
    }

    pub async fn insert(
        &self,
        id: Uuid,
        device_id: Uuid,
        phone_e164: &str,
        country_code: &str,
        national_number: &str,
        sms_content: Option<&str>,
    ) -> Result<Report, String> {
        let mut conn = self.conn().await?;
        let new_report = NewReport {
            id,
            device_id,
            phone_e164,
            country_code,
            national_number,
            sms_content,
        };
        diesel::insert_into(reports::table)
            .values(&new_report)
            .returning(Report::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_by_device(&self, device_id: Uuid) -> Result<Vec<Report>, String> {
        let mut conn = self.conn().await?;
        reports::table
            .filter(reports::device_id.eq(device_id))
            .order(reports::created_at.desc())
            .select(Report::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }
}
