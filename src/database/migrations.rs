// src/database/migrations.rs
use sqlx::SqlitePool;
use anyhow::Result;


pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
  sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS threats (
          id TEXT PRIMARY KEY,
          timestamp TEXT NOT NULL,
          source_ip TEXT NOT NULL,
          dest_ip TEXT,
          severity TEXT NOT NULL,
          category TEXT NOT NULL,
          confidence REAL NOT NULL,
          details TEXT NOT NULL,
          status TEXT NOT NULL,
          metadata TEXT NOT NULL
      )
      "#,
  )
  .execute(pool)
  .await?;

  sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS threat_indicators (
          id TEXT PRIMARY KEY,
          threat_id TEXT NOT NULL,
          indicator_type TEXT NOT NULL,
          value TEXT NOT NULL,
          first_seen TEXT NOT NULL,
          last_seen TEXT NOT NULL,
          confidence REAL NOT NULL,
          FOREIGN KEY(threat_id) REFERENCES threats(id)
      )
      "#,
  )
  .execute(pool)
  .await?;

  sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS blocked_ips (
          ip TEXT PRIMARY KEY,
          first_blocked TEXT NOT NULL,
          last_blocked TEXT NOT NULL,
          block_count INTEGER NOT NULL,
          reason TEXT NOT NULL
      )
      "#,
  )
  .execute(pool)
  .await?;

  Ok(())
}
