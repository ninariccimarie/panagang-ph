use sqlx::PgPool;
use uuid::Uuid;

use crate::report::Report;

pub struct ReportRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ReportRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        id: Uuid,
        device_id: Uuid,
        phone_e164: &str,
        country_code: &str,
        national_number: &str,
        sms_content: Option<&str>,
    ) -> Result<Report, sqlx::Error> {
        sqlx::query_as::<_, Report>(
            r#"
            INSERT INTO reports (
                id, device_id, phone_e164, country_code, national_number, sms_content
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id, device_id, phone_e164, country_code, national_number, sms_content,
                created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(device_id)
        .bind(phone_e164)
        .bind(country_code)
        .bind(national_number)
        .bind(sms_content)
        .fetch_one(self.pool)
        .await
    }

    pub async fn list_by_device(&self, device_id: Uuid) -> Result<Vec<Report>, sqlx::Error> {
        sqlx::query_as::<_, Report>(
            r#"
            SELECT
                id, device_id, phone_e164, country_code, national_number, sms_content,
                created_at, updated_at
            FROM reports
            WHERE device_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(device_id)
        .fetch_all(self.pool)
        .await
    }
}
