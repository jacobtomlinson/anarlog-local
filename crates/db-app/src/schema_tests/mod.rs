use super::*;
use anlg_db_core::Db;

async fn test_db() -> Db {
    let db = Db::open(anlg_db_core::DbOpenOptions {
        storage: anlg_db_core::DbStorage::Memory,
        journal_mode_wal: true,
        foreign_keys: true,
        max_connections: Some(1),
    })
    .await
    .unwrap();
    prepare_schema(&db).await.unwrap();
    db
}

#[tokio::test]
async fn prepare_schema_is_idempotent_for_local_databases() {
    let db = test_db().await;
    sqlx::query("INSERT INTO sessions (id, title) VALUES ('session-1', 'Local note')")
        .execute(db.pool())
        .await
        .unwrap();

    prepare_schema(&db).await.unwrap();

    let title: String = sqlx::query_scalar("SELECT title FROM sessions WHERE id = 'session-1'")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(title, "Local note");
}

#[tokio::test]
async fn prepare_schema_creates_local_core_tables() {
    let db = test_db().await;
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('sessions', 'templates', 'shared_session_cache') ORDER BY name",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert_eq!(
        tables,
        vec!["sessions", "shared_session_cache", "templates"]
    );

    let retired_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('e2ee_records', 'replica_state', 'synced_preferences', 'attachment_transfer_jobs') ORDER BY name",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert!(retired_tables.is_empty());
}
