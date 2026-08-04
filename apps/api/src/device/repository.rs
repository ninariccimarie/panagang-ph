use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::DbPool;
use crate::device::models::{Device, NewDevice};
use crate::schema::devices;

pub struct DeviceRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> DeviceRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    async fn conn(
        &self,
    ) -> Result<diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncPgConnection>, String>
    {
        self.pool
            .get()
            .await
            .map_err(|e| format!("database pool error: {e}"))
    }

    pub async fn insert(&self, id: Uuid, token_hash: &str) -> Result<Device, String> {
        let mut conn = self.conn().await?;
        let new_device = NewDevice { id, token_hash };
        diesel::insert_into(devices::table)
            .values(&new_device)
            .returning(Device::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Device>, String> {
        let mut conn = self.conn().await?;
        devices::table
            .filter(devices::token_hash.eq(token_hash))
            .select(Device::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())
    }

    pub async fn touch_last_seen(&self, id: Uuid) -> Result<(), String> {
        let mut conn = self.conn().await?;
        diesel::update(devices::table.find(id))
            .set(devices::last_seen_at.eq(diesel::dsl::now))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
