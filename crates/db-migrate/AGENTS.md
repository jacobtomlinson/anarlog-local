# `db-migrate`

## Role

App-database migration execution. Narrow port of `sqlx` migration behavior with one intentional divergence: explicit per-step scope (`Plain` vs `Retired`) so removed historical migrations can preserve their ids and checksums without recreating retired tables.

## Owns

- Migration orchestration and `_sqlx_migrations` bookkeeping.
- `MigrationStep` → `sqlx::migrate::Migration` translation.
- Validation of step ids, duplicate versions, and retired-step eligibility.
- Execution semantics for `Plain` vs `Retired`.

## Does Not Own

- Pool creation or database opening.
- Database extension loading or network setup.
- App table definitions, row types, query APIs, migration SQL contents.
- Inference of whether a step requires special runtime behavior.

## Invariants

- `Retired` steps preserve historical migration bookkeeping without executing their historical SQL on fresh databases or existing databases that have already recorded the step.
- Preserves `sqlx` semantics: ordered apply, checksum validation, dirty-version detection, idempotent re-run.
- Migration scope is explicit in the manifest; never inferred from SQL text.
