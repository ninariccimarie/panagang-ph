use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::reports;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = reports)]
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

#[derive(Debug, Insertable)]
#[diesel(table_name = reports)]
pub struct NewReport<'a> {
    pub id: Uuid,
    pub device_id: Uuid,
    pub phone_e164: &'a str,
    pub country_code: &'a str,
    pub national_number: &'a str,
    pub sms_content: Option<&'a str>,
}
