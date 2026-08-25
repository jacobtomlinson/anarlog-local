use super::*;

#[tokio::test]
async fn connect_local_plain_creates_parent_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("nonexistent").join("nested").join("app.db");
    let db = Db::connect_local_plain(&db_path).await.unwrap();
    assert!(db_path.exists());
    drop(db);
}

#[tokio::test]
async fn connect_local_read_only_does_not_create_missing_database() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("missing.db");

    let result = Db::connect_local_read_only(&db_path).await;

    assert!(result.is_err());
    assert!(!db_path.exists());
}

#[tokio::test]
async fn connect_local_read_only_rejects_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("app.db");
    let writable = Db::connect_local_plain(&db_path).await.unwrap();
    sqlx::query("CREATE TABLE records (id TEXT PRIMARY KEY NOT NULL)")
        .execute(writable.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO records (id) VALUES ('existing')")
        .execute(writable.pool())
        .await
        .unwrap();
    writable.pool().close().await;

    let read_only = Db::connect_local_read_only(&db_path).await.unwrap();
    let rows: Vec<String> = sqlx::query_scalar("SELECT id FROM records ORDER BY id")
        .fetch_all(read_only.pool())
        .await
        .unwrap();
    let query_only: i64 = sqlx::query_scalar("PRAGMA query_only")
        .fetch_one(read_only.pool())
        .await
        .unwrap();
    let write_result = sqlx::query("INSERT INTO records (id) VALUES ('rejected')")
        .execute(read_only.pool())
        .await;

    assert_eq!(rows, vec!["existing"]);
    assert_eq!(query_only, 1);
    assert!(write_result.is_err());
}

#[tokio::test]
async fn open_applies_requested_pragmas() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("app.db");

    let db = Db::open(DbOpenOptions {
        storage: DbStorage::Local(&db_path),
        journal_mode_wal: true,
        foreign_keys: true,
        max_connections: Some(1),
    })
    .await
    .unwrap();

    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(db.pool())
        .await
        .unwrap();

    assert_eq!(foreign_keys, 1);
    assert_eq!(journal_mode.to_lowercase(), "wal");
    assert_eq!(busy_timeout, SQLITE_BUSY_TIMEOUT.as_millis() as i64);
}

#[tokio::test]
async fn change_notifier_tracks_committed_local_transactions() {
    let db = Db::connect_memory_plain().await.unwrap();
    sqlx::query("CREATE TABLE events (id TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)")
        .execute(db.pool())
        .await
        .unwrap();
    let mut changes = db.change_notifier().subscribe();

    sqlx::query("INSERT INTO events (id, value) VALUES ('a', 'one')")
        .execute(db.pool())
        .await
        .unwrap();
    let inserted = tokio::time::timeout(std::time::Duration::from_secs(1), changes.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inserted.table, "events");
    assert_eq!(inserted.kind, anlg_db_change::TableChangeKind::Insert);

    let mut transaction = db.pool().begin().await.unwrap();
    sqlx::query("UPDATE events SET value = 'two' WHERE id = 'a'")
        .execute(&mut *transaction)
        .await
        .unwrap();
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), changes.recv())
            .await
            .is_err()
    );
    transaction.commit().await.unwrap();

    let updated = tokio::time::timeout(std::time::Duration::from_secs(1), changes.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.table, "events");
    assert_eq!(updated.kind, anlg_db_change::TableChangeKind::Update);
    assert_eq!(updated.seq, inserted.seq + 1);
}
