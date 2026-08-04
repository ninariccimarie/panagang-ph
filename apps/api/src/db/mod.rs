//! Diesel connection pool and embedded migrations.

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = Pool<AsyncPgConnection>;

/// Apply pending Diesel migrations (sync; uses `__diesel_schema_migrations`).
pub fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = PgConnection::establish(database_url)?;
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub async fn create_pool(
    database_url: &str,
) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder().max_size(10).build(manager).await?;
    Ok(pool)
}
