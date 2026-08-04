use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Report {
    pub id: Uuid,
    pub device_id: Uuid,
    pub phone_e164: String,
    pub country_code: String,
    pub national_number: String,
    pub sms_content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
