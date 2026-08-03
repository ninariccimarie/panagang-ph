use axum::http::HeaderMap;
use sqlx::PgPool;

use crate::models::Device;
use crate::services::DeviceService;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub device: Option<Device>,
}

pub async fn authenticate_from_headers(
    headers: &HeaderMap,
    pool: &PgPool,
    token_secret: &str,
) -> Option<Device> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    DeviceService::new(pool, token_secret)
        .authenticate(token)
        .await
        .ok()
}
