use async_graphql::{InputObject, SimpleObject, ID};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::Report;

#[derive(SimpleObject)]
pub struct RegisterDevicePayload {
    pub device_id: ID,
    pub token: String,
}

#[derive(InputObject)]
pub struct SubmitScamReportInput {
    pub country_code: String,
    pub phone_number: String,
    pub sms_content: Option<String>,
}

#[derive(SimpleObject)]
pub struct ScamReport {
    pub id: ID,
    pub phone_e164: String,
    pub country_code: String,
    pub national_number: String,
    pub sms_content: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Report> for ScamReport {
    fn from(report: Report) -> Self {
        Self {
            id: ID(report.id.to_string()),
            phone_e164: report.phone_e164,
            country_code: report.country_code,
            national_number: report.national_number,
            sms_content: report.sms_content,
            created_at: report.created_at,
        }
    }
}

pub fn parse_uuid(id: &Uuid) -> ID {
    ID(id.to_string())
}
