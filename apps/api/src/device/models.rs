use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::devices;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = devices)]
pub struct Device {
    pub id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice<'a> {
    pub id: Uuid,
    pub token_hash: &'a str,
}
