#[derive(Debug, Clone, serde::Serialize, specta::Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreparedSharedUpload {
    pub cache_id: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedAttachmentCacheResult {
    pub cache_id: String,
    pub local_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedUploadVersion {
    pub sha256: String,
    pub size_bytes: u64,
    pub filename: String,
    pub content_type: String,
    pub remote_object_key: String,
}
