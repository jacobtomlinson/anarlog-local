use std::sync::Arc;

use anlg_db_core::Db;

const DB_FILENAME: &str = "app.db";
const DB_OPEN_LOCK_RETRIES: u32 = 12;
const DB_OPEN_LOCK_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

pub async fn open_desktop_db(identifier: &str) -> Result<Arc<Db>, String> {
    let dir = desktop_db_dir(identifier)
        .ok_or_else(|| "application data directory is unavailable".to_string())?;
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("failed to create application data directory: {error}"))?;

    let db_path = dir.join(DB_FILENAME);

    // During an update relaunch the previous process can hold the database for
    // several seconds while it flushes and exits; retry instead of failing the
    // whole startup on a transient lock.
    let mut attempts = 0u32;
    let db = loop {
        match tauri_plugin_db::open_app_db_unmigrated(Some(&db_path)).await {
            Ok(db) => break db,
            Err(error) if attempts < DB_OPEN_LOCK_RETRIES && is_transient_lock_error(&error) => {
                attempts += 1;
                eprintln!(
                    "application database is locked by another process; \
                     retrying ({attempts}/{DB_OPEN_LOCK_RETRIES}): {error}"
                );
                tokio::time::sleep(DB_OPEN_LOCK_RETRY_DELAY).await;
            }
            Err(error) => {
                return Err(format!("failed to open application database: {error}"));
            }
        }
    };

    Ok(Arc::new(db))
}

pub fn is_transient_lock_error(error: &impl std::fmt::Display) -> bool {
    let message = error.to_string();
    message.contains("database is locked") || message.contains("database table is locked")
}

// Matches MigrateError::SchemaFromNewerApp, which reaches startup as a string
// after crossing the tauri plugin setup boundary.
pub fn is_newer_schema_error(error: &impl std::fmt::Display) -> bool {
    error
        .to_string()
        .contains("created by a newer version of Anarlog")
}
pub(crate) fn desktop_db_dir(identifier: &str) -> Option<std::path::PathBuf> {
    let data_dir = dirs::data_dir()?;
    let default_dir = anlg_storage::global::compute_default_base(identifier)?;
    let identifier_dir = data_dir.join(identifier);

    if identifier_dir.join(DB_FILENAME).is_file() && !default_dir.join(DB_FILENAME).is_file() {
        Some(identifier_dir)
    } else {
        Some(default_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_lock_errors_are_recognized() {
        assert!(is_transient_lock_error(
            &"error returned from database: (code: 5) database is locked"
        ));
        assert!(is_transient_lock_error(
            &"error returned from database: (code: 6) database table is locked"
        ));
        assert!(!is_transient_lock_error(
            &"unable to open database file: /tmp/app.db"
        ));
    }

    #[test]
    fn newer_schema_errors_are_recognized() {
        // Rendered form of MigrateError::SchemaFromNewerApp after crossing the
        // plugin setup boundary as a string.
        assert!(is_newer_schema_error(
            &"plugin db failed: the database was created by a newer version of Anarlog: it requires migration 20260901000000, but this build only includes migrations up to 20260816100100"
        ));
        assert!(!is_newer_schema_error(
            &"unable to open database file: /tmp/app.db"
        ));
    }

    #[test]
    fn dev_uses_an_isolated_persistent_database() {
        let db_dir = desktop_db_dir("com.hyprnote.dev").unwrap();

        assert!(db_dir.ends_with("com.hyprnote.dev"));
    }
}
