#![forbid(unsafe_code)]

mod error;
mod migrate;
mod schema;

pub use error::MigrateError;
pub use schema::{DbSchema, MigrationScope, MigrationStep, RetiredMigration};

use anlg_db_core::Db;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationProgress {
    pub completed: usize,
    pub total: usize,
}

pub async fn migrate(db: &Db, schema: DbSchema) -> Result<(), MigrateError> {
    migrate_with_progress(db, schema, |_| {}).await
}

pub async fn migrate_with_progress(
    db: &Db,
    schema: DbSchema,
    on_progress: impl FnMut(MigrationProgress) + Send,
) -> Result<(), MigrateError> {
    migrate::run_migrations(db, schema, &[], on_progress).await
}

pub async fn migrate_with_retired_migrations_with_progress(
    db: &Db,
    schema: DbSchema,
    retired_migrations: &'static [RetiredMigration],
    on_progress: impl FnMut(MigrationProgress) + Send,
) -> Result<(), MigrateError> {
    migrate::run_migrations(db, schema, retired_migrations, on_progress).await
}

pub async fn migrate_with_retired_migrations(
    db: &Db,
    schema: DbSchema,
    retired_migrations: &'static [RetiredMigration],
) -> Result<(), MigrateError> {
    migrate_with_retired_migrations_with_progress(db, schema, retired_migrations, |_| {}).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anlg_db_core::{DbOpenOptions, DbStorage};

    fn empty_schema() -> DbSchema {
        DbSchema {
            steps: &[],
            validate_cloudsync_table: |_table| false,
        }
    }

    fn schema_of(steps: &'static [MigrationStep]) -> DbSchema {
        DbSchema {
            steps,
            validate_cloudsync_table: |_table| false,
        }
    }

    async fn open_memory_db() -> Db {
        Db::open(DbOpenOptions {
            storage: DbStorage::Memory,
            cloudsync_enabled: false,
            journal_mode_wal: true,
            foreign_keys: true,
            max_connections: Some(1),
        })
        .await
        .unwrap()
    }

    const STEP_ONE: MigrationStep = MigrationStep {
        id: "10_one",
        scope: MigrationScope::Plain,
        sql: "CREATE TABLE t_one (id INTEGER PRIMARY KEY);",
    };
    const STEP_TWO_ADDITIVE: MigrationStep = MigrationStep {
        id: "20_two",
        scope: MigrationScope::Plain,
        sql: "CREATE TABLE t_two (id INTEGER PRIMARY KEY);",
    };
    const STEP_TWO_BREAKING: MigrationStep = MigrationStep {
        id: "20_two",
        scope: MigrationScope::Plain,
        sql: "-- reworks t_one in a way older builds cannot read\n-- breaking\nCREATE TABLE t_two (id INTEGER PRIMARY KEY);",
    };
    const STEP_THREE_ADDITIVE: MigrationStep = MigrationStep {
        id: "30_three",
        scope: MigrationScope::Plain,
        sql: "CREATE TABLE t_three (id INTEGER PRIMARY KEY);",
    };
    const RETIRED_STEP_TWO: &[RetiredMigration] = &[RetiredMigration {
        version: 20,
        checksum: &[
            0x4b, 0x75, 0x94, 0x27, 0x93, 0x1b, 0xd0, 0xa5, 0xf8, 0x25, 0x8c, 0x74, 0x6b, 0xd9,
            0xd6, 0xd3, 0x98, 0xd6, 0x43, 0x33, 0xf6, 0x17, 0xab, 0x48, 0x42, 0x15, 0x11, 0x99,
            0x24, 0x92, 0xbe, 0x95, 0xfa, 0x21, 0xf7, 0x0f, 0x36, 0xa1, 0x57, 0x48, 0xba, 0x41,
            0x92, 0x42, 0x73, 0x57, 0x75, 0xee,
        ],
    }];

    #[tokio::test]
    async fn older_build_tolerates_newer_additive_migrations() {
        let db = open_memory_db().await;
        migrate(&db, schema_of(&[STEP_ONE, STEP_TWO_ADDITIVE]))
            .await
            .unwrap();

        migrate(&db, schema_of(&[STEP_ONE])).await.unwrap();

        let recorded: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(db.pool())
                .await
                .unwrap();
        assert_eq!(recorded, vec![10, 20]);
    }

    #[tokio::test]
    async fn retired_migrations_keep_existing_databases_open_without_running_again() {
        let db = open_memory_db().await;
        migrate(&db, schema_of(&[STEP_ONE, STEP_TWO_ADDITIVE]))
            .await
            .unwrap();

        migrate_with_retired_migrations(
            &db,
            schema_of(&[STEP_ONE, STEP_THREE_ADDITIVE]),
            RETIRED_STEP_TWO,
        )
        .await
        .unwrap();

        let fresh_db = open_memory_db().await;
        migrate_with_retired_migrations(
            &fresh_db,
            schema_of(&[STEP_ONE, STEP_THREE_ADDITIVE]),
            RETIRED_STEP_TWO,
        )
        .await
        .unwrap();

        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 't_two'",
        )
        .fetch_one(fresh_db.pool())
        .await
        .unwrap();
        assert_eq!(table_count, 0);
    }

    #[tokio::test]
    async fn newer_database_rejects_missing_local_migration() {
        let db = open_memory_db().await;
        migrate(&db, schema_of(&[STEP_ONE, STEP_THREE_ADDITIVE]))
            .await
            .unwrap();

        let error = migrate(&db, schema_of(&[STEP_ONE, STEP_TWO_ADDITIVE]))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            MigrateError::DatabaseAhead {
                missing_version: 20,
                max_applied_version: 30,
            }
        ));
        assert!(
            error
                .to_string()
                .contains("created by a newer version of Anarlog")
        );

        let recorded: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(db.pool())
                .await
                .unwrap();
        assert_eq!(recorded, vec![10, 30]);

        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 't_two'",
        )
        .fetch_one(db.pool())
        .await
        .unwrap();
        assert_eq!(table_count, 0);
    }

    #[tokio::test]
    async fn breaking_migration_blocks_older_builds() {
        let db = open_memory_db().await;
        migrate(&db, schema_of(&[STEP_ONE, STEP_TWO_BREAKING]))
            .await
            .unwrap();

        let error = migrate(&db, schema_of(&[STEP_ONE])).await.unwrap_err();

        assert!(matches!(
            error,
            MigrateError::SchemaFromNewerApp {
                min_supported_version: 20,
                max_known_version: 10,
            }
        ));
        assert!(
            error
                .to_string()
                .contains("created by a newer version of Anarlog")
        );
    }

    #[tokio::test]
    async fn breaking_floor_allows_builds_that_include_it() {
        let db = open_memory_db().await;
        migrate(
            &db,
            schema_of(&[STEP_ONE, STEP_TWO_BREAKING, STEP_THREE_ADDITIVE]),
        )
        .await
        .unwrap();

        migrate(&db, schema_of(&[STEP_ONE, STEP_TWO_BREAKING]))
            .await
            .unwrap();

        let floor: i64 = sqlx::query_scalar(
            "SELECT min_supported_version FROM _anlg_schema_compat WHERE id = 0",
        )
        .fetch_one(db.pool())
        .await
        .unwrap();
        assert_eq!(floor, 20);
    }

    #[tokio::test]
    async fn unknown_migration_below_newest_known_still_fails() {
        let db = open_memory_db().await;
        migrate(&db, schema_of(&[STEP_ONE, STEP_TWO_ADDITIVE]))
            .await
            .unwrap();

        let error = migrate(&db, schema_of(&[STEP_TWO_ADDITIVE]))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            MigrateError::SqlxMigrate(sqlx::migrate::MigrateError::VersionMissing(10))
        ));
    }

    #[tokio::test]
    async fn migrate_bootstraps_migration_history() {
        let db = Db::open(DbOpenOptions {
            storage: DbStorage::Memory,
            cloudsync_enabled: false,
            journal_mode_wal: true,
            foreign_keys: true,
            max_connections: Some(1),
        })
        .await
        .unwrap();

        migrate(&db, empty_schema()).await.unwrap();

        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert!(tables.contains(&"_sqlx_migrations".to_string()));
    }

    #[tokio::test]
    async fn migration_progress_only_counts_pending_steps() {
        let db = open_memory_db().await;
        migrate(&db, schema_of(&[STEP_ONE])).await.unwrap();

        let mut updates = Vec::new();
        migrate_with_progress(
            &db,
            schema_of(&[STEP_ONE, STEP_TWO_ADDITIVE, STEP_THREE_ADDITIVE]),
            |progress| updates.push(progress),
        )
        .await
        .unwrap();

        assert_eq!(
            updates,
            vec![
                MigrationProgress {
                    completed: 0,
                    total: 2,
                },
                MigrationProgress {
                    completed: 1,
                    total: 2,
                },
                MigrationProgress {
                    completed: 2,
                    total: 2,
                },
            ]
        );
    }
}
