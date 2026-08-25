use crate::{Envelope, Error, Result, WorkspaceKey};

/// Every workspace key generation a local replica can still read.
///
/// Rotation mints a new key and leaves earlier ones in place, so history stays
/// readable while writes move to the newest generation.
#[derive(Clone)]
pub struct WorkspaceKeyring {
    active: WorkspaceKey,
    retired: Vec<WorkspaceKey>,
}

impl WorkspaceKeyring {
    pub fn new(active: WorkspaceKey) -> Self {
        Self {
            active,
            retired: Vec::new(),
        }
    }

    /// Adds an older generation. Re-adding the active key or a duplicate is a
    /// no-op so callers can replay local key history without bookkeeping.
    pub fn insert_retired(&mut self, key: WorkspaceKey) {
        if self.get(key.key_id()).is_some() {
            return;
        }
        self.retired.push(key);
    }

    pub fn active(&self) -> &WorkspaceKey {
        &self.active
    }

    pub fn generations(&self) -> impl Iterator<Item = &WorkspaceKey> {
        std::iter::once(&self.active).chain(self.retired.iter())
    }

    pub fn get(&self, key_id: &str) -> Option<&WorkspaceKey> {
        if self.active.key_id() == key_id {
            return Some(&self.active);
        }
        self.retired.iter().find(|key| key.key_id() == key_id)
    }

    pub fn open_field(
        &self,
        workspace_id: &str,
        record_id: &str,
        payload: &str,
    ) -> Result<crate::OpenedField> {
        let envelope: Envelope =
            serde_json::from_str(payload).map_err(|_| Error::InvalidPayload)?;
        self.get(&envelope.key_id)
            .ok_or(Error::UnknownKey)?
            .open_field(workspace_id, record_id, payload)
    }
}

impl From<WorkspaceKey> for WorkspaceKeyring {
    fn from(key: WorkspaceKey) -> Self {
        Self::new(key)
    }
}

impl std::ops::Deref for WorkspaceKeyring {
    type Target = WorkspaceKey;

    fn deref(&self) -> &Self::Target {
        self.active()
    }
}
