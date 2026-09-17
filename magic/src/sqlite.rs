use std::time::Duration;

use sqlx::Connection;

pub use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

// ---------------------------------------------------------------------------
// SqliteConfig — configuración para conexiones SQLite
// ---------------------------------------------------------------------------

pub struct SqliteConfig {
    pub journal_mode: SqliteJournalMode,
    pub foreign_keys: bool,
    pub busy_timeout: Duration,
    pub synchronous: SqliteSynchronous,
}

pub enum SqliteSynchronous {
    Off,
    Normal,
    Full,
    Extra,
}

impl SqliteSynchronous {
    fn as_str(&self) -> &'static str {
        match self {
            SqliteSynchronous::Off => "OFF",
            SqliteSynchronous::Normal => "NORMAL",
            SqliteSynchronous::Full => "FULL",
            SqliteSynchronous::Extra => "EXTRA",
        }
    }
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            journal_mode: SqliteJournalMode::Wal,
            foreign_keys: true,
            busy_timeout: Duration::from_secs(5),
            synchronous: SqliteSynchronous::Normal,
        }
    }
}

impl SqliteConfig {
    pub fn in_memory() -> Self {
        Self {
            journal_mode: SqliteJournalMode::Memory,
            foreign_keys: true,
            busy_timeout: Duration::from_secs(5),
            synchronous: SqliteSynchronous::Normal,
        }
    }
}

// ---------------------------------------------------------------------------
// Sqlite — entry point para conexión SQLite
// ---------------------------------------------------------------------------

pub struct Sqlite;

impl Sqlite {
    pub async fn open(path: &str) -> anyhow::Result<sqlx::SqliteConnection> {
        Self::open_with_config(path, SqliteConfig::default()).await
    }

    pub async fn open_with_config(
        path: &str,
        config: SqliteConfig,
    ) -> anyhow::Result<sqlx::SqliteConnection> {
        let options = build_options(path, &config);
        let conn = sqlx::SqliteConnection::connect_with(&options).await?;
        Ok(conn)
    }

    pub async fn open_memory() -> anyhow::Result<sqlx::SqliteConnection> {
        Self::open_memory_with_config(SqliteConfig::in_memory()).await
    }

    pub async fn open_memory_with_config(
        config: SqliteConfig,
    ) -> anyhow::Result<sqlx::SqliteConnection> {
        let options = build_options(":memory:", &config);
        let conn = sqlx::SqliteConnection::connect_with(&options).await?;
        Ok(conn)
    }

    pub async fn pool(path: &str) -> anyhow::Result<sqlx::SqlitePool> {
        Self::pool_with_config(path, SqliteConfig::default()).await
    }

    pub async fn pool_with_config(
        path: &str,
        config: SqliteConfig,
    ) -> anyhow::Result<sqlx::SqlitePool> {
        // For in-memory databases with a pool, use shared cache to allow
        // multiple connections to access the same in-memory database
        let actual_path = if path == ":memory:" {
            "file::memory:?cache=shared"
        } else {
            path
        };
        let options = build_options(actual_path, &config);
        let pool = sqlx::SqlitePool::connect_with(options).await?;
        Ok(pool)
    }

    /// Run VACUUM to rebuild the database file and reclaim space.
    ///
    /// Should be called after large deletions to reduce file size.
    /// This operation locks the database and may take time for large databases.
    pub async fn vacuum(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
        sqlx::query("VACUUM").execute(pool).await?;
        Ok(())
    }

    /// Get the current database size in bytes.
    ///
    /// Uses `PRAGMA page_count * page_size` for accurate measurement.
    pub async fn database_size(pool: &sqlx::SqlitePool) -> anyhow::Result<u64> {
        let page_size: (i64,) = sqlx::query_as("PRAGMA page_size").fetch_one(pool).await?;
        let page_count: (i64,) = sqlx::query_as("PRAGMA page_count").fetch_one(pool).await?;
        Ok((page_size.0 * page_count.0) as u64)
    }

    /// Get the number of free pages (pages marked for reuse after VACUUM).
    pub async fn freelist_count(pool: &sqlx::SqlitePool) -> anyhow::Result<u64> {
        let count: (i64,) = sqlx::query_as("PRAGMA freelist_count")
            .fetch_one(pool)
            .await?;
        Ok(count.0 as u64)
    }
}

fn build_options(path: &str, config: &SqliteConfig) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(config.journal_mode)
        .foreign_keys(config.foreign_keys)
        .busy_timeout(config.busy_timeout)
        .pragma("synchronous", config.synchronous.as_str())
}
