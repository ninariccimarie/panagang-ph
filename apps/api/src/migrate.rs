//! Folder-based SQL migrations: `yyyy-mm-dd-hhmmss-short-description/{up,down}.sql`.

use std::fs;
use std::path::{Path, PathBuf};

use sqlx::PgPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrateError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration `{0}` is missing up.sql")]
    MissingUp(String),
    #[error("migration `{0}` is missing down.sql")]
    MissingDown(String),
    #[error("no applied migrations to revert")]
    NothingToRevert,
}

#[derive(Debug, Clone)]
struct MigrationDir {
    /// Directory name; also the version key stored in `schema_migrations`.
    id: String,
    path: PathBuf,
}

fn migrations_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

fn list_migration_dirs(root: &Path) -> Result<Vec<MigrationDir>, MigrateError> {
    let mut dirs = Vec::new();
    if !root.exists() {
        return Ok(dirs);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        // Expect: yyyy-mm-dd-hhmmss-short-description
        if id.len() < 18 || &id[4..5] != "-" || &id[7..8] != "-" || &id[10..11] != "-" {
            continue;
        }
        dirs.push(MigrationDir { id, path });
    }

    dirs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(dirs)
}

async fn ensure_schema_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn applied_versions(pool: &PgPool) -> Result<Vec<String>, MigrateError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT version FROM schema_migrations ORDER BY version ASC")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(v,)| v).collect())
}

/// Apply all pending `up.sql` migrations in chronological order.
pub async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    ensure_schema_migrations(pool).await?;
    let applied = applied_versions(pool).await?;
    let dirs = list_migration_dirs(&migrations_root())?;

    for dir in dirs {
        if applied.iter().any(|v| v == &dir.id) {
            continue;
        }

        let up_path = dir.path.join("up.sql");
        if !up_path.exists() {
            return Err(MigrateError::MissingUp(dir.id));
        }
        // Enforce down.sql exists so every migration stays reversible.
        let down_path = dir.path.join("down.sql");
        if !down_path.exists() {
            return Err(MigrateError::MissingDown(dir.id));
        }

        let sql = fs::read_to_string(&up_path)?;
        let mut tx = pool.begin().await?;
        sqlx::raw_sql(&sql).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(&dir.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        tracing::info!(migration = %dir.id, "applied migration");
    }

    Ok(())
}

/// Revert the most recently applied migration using its `down.sql`.
#[allow(dead_code)] // exposed for future CLI / admin rollback helpers
pub async fn revert_last(pool: &PgPool) -> Result<String, MigrateError> {
    ensure_schema_migrations(pool).await?;
    let applied = applied_versions(pool).await?;
    let Some(version) = applied.last().cloned() else {
        return Err(MigrateError::NothingToRevert);
    };

    let dir = migrations_root().join(&version);
    let down_path = dir.join("down.sql");
    if !down_path.exists() {
        return Err(MigrateError::MissingDown(version));
    }

    let sql = fs::read_to_string(&down_path)?;
    let mut tx = pool.begin().await?;
    sqlx::raw_sql(&sql).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
        .bind(&version)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    tracing::info!(migration = %version, "reverted migration");
    Ok(version)
}
