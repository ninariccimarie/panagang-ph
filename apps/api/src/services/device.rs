use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::models::Device;
use crate::repositories::DeviceRepository;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct RegisteredDevice {
    pub device: Device,
    /// Plaintext token returned only at registration time.
    pub token: String,
}

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("invalid device token")]
    InvalidToken,
}

pub struct DeviceService<'a> {
    pool: &'a PgPool,
    token_secret: &'a str,
}

impl<'a> DeviceService<'a> {
    pub fn new(pool: &'a PgPool, token_secret: &'a str) -> Self {
        Self { pool, token_secret }
    }

    pub fn hash_token(&self, token: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.token_secret.as_bytes())
            .expect("HMAC key length is valid");
        mac.update(token.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn generate_token() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    pub async fn register(&self) -> Result<RegisteredDevice, DeviceError> {
        let token = Self::generate_token();
        let token_hash = self.hash_token(&token);
        let id = Uuid::new_v4();
        let device = DeviceRepository::new(self.pool)
            .insert(id, &token_hash)
            .await?;
        Ok(RegisteredDevice { device, token })
    }

    pub async fn authenticate(&self, bearer_token: &str) -> Result<Device, DeviceError> {
        let token = bearer_token.trim();
        if token.is_empty() {
            return Err(DeviceError::InvalidToken);
        }
        let token_hash = self.hash_token(token);
        let repo = DeviceRepository::new(self.pool);
        let device = repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(DeviceError::InvalidToken)?;
        repo.touch_last_seen(device.id).await?;
        Ok(device)
    }
}
