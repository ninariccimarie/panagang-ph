use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Device;

pub struct DeviceRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DeviceRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, id: Uuid, token_hash: &str) -> Result<Device, sqlx::Error> {
        sqlx::query_as::<_, Device>(
            r#"
            INSERT INTO devices (id, token_hash)
            VALUES ($1, $2)
            RETURNING id, token_hash, created_at, last_seen_at
            "#,
        )
        .bind(id)
        .bind(token_hash)
        .fetch_one(self.pool)
        .await
    }

    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Device>, sqlx::Error> {
        sqlx::query_as::<_, Device>(
            r#"
            SELECT id, token_hash, created_at, last_seen_at
            FROM devices
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn touch_last_seen(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE devices
            SET last_seen_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
