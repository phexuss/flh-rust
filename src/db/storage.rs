use anyhow::Result;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tracing::info;

#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    pub async fn init(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let db_str = db_path.to_str().unwrap_or("bot.db");
        let connection_options = SqliteConnectOptions::from_str(&format!("sqlite:{}?mode=rwc", db_str))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        sqlx::query("PRAGMA journal_mode=WAL;").execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS seen_projects (
                project_id INTEGER PRIMARY KEY,
                seen_at    TEXT NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await?;

        info!("SQLite storage initialized at: {}", db_str);

        Ok(Self { pool })
    }

    pub async fn is_seen(&self, project_id: i64) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT project_id FROM seen_projects WHERE project_id = ?")
            .bind(project_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.is_some())
    }

    pub async fn mark_seen(&self, project_id: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT OR IGNORE INTO seen_projects (project_id, seen_at) VALUES (?, ?)")
            .bind(project_id)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_seen_batch(&self, project_ids: &[i64]) -> Result<()> {
        if project_ids.is_empty() {
            return Ok(());
        }

        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;

        for &id in project_ids {
            sqlx::query("INSERT OR IGNORE INTO seen_projects (project_id, seen_at) VALUES (?, ?)")
                .bind(id)
                .bind(&now)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn count_seen(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM seen_projects")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }

    pub async fn cleanup_old(&self, days: i64) -> Result<u64> {
        let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();
        let result = sqlx::query("DELETE FROM seen_projects WHERE seen_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            info!("Cleaned up {} old records from seen_projects", deleted);
        }

        Ok(deleted)
    }
}
