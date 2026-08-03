use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::models::Report;
use crate::repositories::ReportRepository;
use crate::services::phone::{normalize_phone, PhoneError};

#[derive(Debug, Clone)]
pub struct SubmitReportInput {
    pub country_code: String,
    pub phone_number: String,
    pub sms_content: Option<String>,
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error(transparent)]
    Phone(#[from] PhoneError),
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

pub struct ReportService<'a> {
    pool: &'a PgPool,
}

impl<'a> ReportService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn submit(
        &self,
        device_id: Uuid,
        input: SubmitReportInput,
    ) -> Result<Report, ReportError> {
        let normalized = normalize_phone(&input.country_code, &input.phone_number)?;
        let sms = input
            .sms_content
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let report = ReportRepository::new(self.pool)
            .insert(
                Uuid::new_v4(),
                device_id,
                &normalized.e164,
                &normalized.country_code,
                &normalized.national_number,
                sms.as_deref(),
            )
            .await?;

        // AI classification is enqueued in a later issue; persist only for now.
        Ok(report)
    }

    pub async fn list_for_device(&self, device_id: Uuid) -> Result<Vec<Report>, ReportError> {
        Ok(ReportRepository::new(self.pool)
            .list_by_device(device_id)
            .await?)
    }
}
