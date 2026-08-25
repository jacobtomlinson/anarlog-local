#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationScope {
    Plain,
    /// A shipped Sync-only migration whose checksum and history remain valid,
    /// but whose schema change is intentionally not applied by local-only builds.
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationStep {
    pub id: &'static str,
    pub scope: MigrationScope,
    pub sql: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetiredMigration {
    pub version: i64,
    pub checksum: &'static [u8],
}

#[derive(Clone, Copy)]
pub struct DbSchema {
    pub steps: &'static [MigrationStep],
}
