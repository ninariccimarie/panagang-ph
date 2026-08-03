use async_graphql::{SimpleObject, ID};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct RegisterDevicePayload {
    pub device_id: ID,
    pub token: String,
}

pub fn parse_uuid(id: &Uuid) -> ID {
    ID(id.to_string())
}
