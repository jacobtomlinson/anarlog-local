use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use sqlx::Connection;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[derive(Clone, Copy, Debug)]
pub enum DbStorage<'a> {
    Local(&'a Path),
    Memory,
}

#[derive(Clone, Copy, Debug)]
pub struct DbOpenOptions<'a> {
    pub storage: DbStorage<'a>,
    pub journal_mode_wal: bool,
    pub foreign_keys: bool,
    pub max_connections: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum DbOpenError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub type ManagedDb = std::sync::Arc<Db>;

const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Db {
    pub(crate) pool: SqlitePool,
    change_notifier: anlg_db_change::ChangeNotifier,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db")
            .field("change_notifier", &true)
            .finish_non_exhaustive()
    }
}

impl Db {
    pub async fn open(options: DbOpenOptions<'_>) -> Result<Self, DbOpenError> {
        let mut connect_options = match options.storage {
            DbStorage::Local(path) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(true)
            }
            DbStorage::Memory => SqliteConnectOptions::from_str("sqlite::memory:")?,
        }
        .busy_timeout(SQLITE_BUSY_TIMEOUT);

        if options.journal_mode_wal {
            connect_options = connect_options.pragma("journal_mode", "WAL");
        }
        if options.foreign_keys {
            connect_options = connect_options.pragma("foreign_keys", "ON");
        }

        let (change_notifier, pool_options) = anlg_db_change::ChangeNotifier::new();
        let mut pool_options = apply_internal_pool_policy(pool_options);
        match options.storage {
            DbStorage::Memory => pool_options = pool_options.max_connections(1),
            DbStorage::Local(_) => {
                if let Some(max) = options.max_connections {
                    pool_options = pool_options.max_connections(max);
                }
            }
        }

        let pool = pool_options.connect_with(connect_options).await?;
        Ok(Self {
            pool,
            change_notifier,
        })
    }

    pub async fn connect_local(path: impl AsRef<Path>) -> Result<Self, sqlx::Error> {
        Self::open(DbOpenOptions {
            storage: DbStorage::Local(path.as_ref()),
            journal_mode_wal: true,
            foreign_keys: true,
            max_connections: Some(4),
        })
        .await
        .map_err(db_open_error_to_sqlx)
    }

    pub async fn connect_local_plain(path: impl AsRef<Path>) -> Result<Self, sqlx::Error> {
        Self::open(DbOpenOptions {
            storage: DbStorage::Local(path.as_ref()),
            journal_mode_wal: false,
            foreign_keys: true,
            max_connections: Some(4),
        })
        .await
        .map_err(db_open_error_to_sqlx)
    }

    pub async fn connect_local_read_only(path: impl AsRef<Path>) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .read_only(true)
            .pragma("foreign_keys", "ON")
            .pragma("query_only", "ON")
            .busy_timeout(SQLITE_BUSY_TIMEOUT);
        let (change_notifier, pool_options) = anlg_db_change::ChangeNotifier::new();
        let pool = apply_internal_pool_policy(pool_options)
            .connect_with(options)
            .await?;
        Ok(Self {
            pool,
            change_notifier,
        })
    }

    pub async fn connect_memory_plain() -> Result<Self, sqlx::Error> {
        Self::open(DbOpenOptions {
            storage: DbStorage::Memory,
            journal_mode_wal: false,
            foreign_keys: true,
            max_connections: Some(1),
        })
        .await
        .map_err(db_open_error_to_sqlx)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn change_notifier(&self) -> &anlg_db_change::ChangeNotifier {
        &self.change_notifier
    }
}

fn db_open_error_to_sqlx(error: DbOpenError) -> sqlx::Error {
    match error {
        DbOpenError::Sqlx(error) => error,
        DbOpenError::Io(error) => sqlx::Error::Io(error),
    }
}

fn apply_internal_pool_policy(pool_options: SqlitePoolOptions) -> SqlitePoolOptions {
    pool_options.after_release(|connection, _| {
        Box::pin(async move {
            if !connection.is_in_transaction() {
                return Ok(true);
            }

            tracing::warn!("sqlite_connection_returned_in_transaction");
            if let Err(error) = connection.ping().await {
                tracing::error!(%error, "sqlite_transaction_repair_failed");
                return Ok(false);
            }

            if connection.is_in_transaction() {
                tracing::error!("sqlite_connection_rejected_in_transaction");
                return Ok(false);
            }

            tracing::info!("sqlite_transaction_repaired_before_pool_return");
            Ok(true)
        })
    })
}

#[cfg(test)]
mod tests;
