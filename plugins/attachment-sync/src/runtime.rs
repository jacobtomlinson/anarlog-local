use std::path::{Component, Path, PathBuf};

use sqlx::FromRow;
use tauri::Runtime;
use tauri_plugin_settings::SettingsPluginExt;
use uuid::Uuid;

use crate::control::DownloadOperation;
use crate::error::{Error, Result};
use crate::models::{PreparedSharedUpload, SharedAttachmentCacheResult, SharedUploadVersion};

mod cache;
mod download;

use cache::{
    cached_shared_attachment_path, clear_attachment_cache_directory, commit_shared_cache_entry,
    create_shared_upload_cache_root, file_matches_cancellable_async, read_range,
    read_shared_cache_metadata, shared_cache_id, shared_cache_matches_async,
    shared_cache_metadata_path, shared_cache_root, shared_preview_cache_root, shared_scope_path,
    shared_upload_cache_path, shared_upload_cache_root, snapshot_verified_file, valid_sha256,
    validate_range,
};
use download::{
    DownloadObject, download_to_path, require_configured_supabase_url, validate_signed_download_url,
};

const MAX_PLAINTEXT_BYTES: u64 = 545_259_520;
pub(crate) const SHARED_PREVIEW_SCOPE_PREFIX: &str =
    anlg_attachment_sync_core::SHARED_PREVIEW_SCOPE_PREFIX;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
struct SharedUploadAttachment {
    attachment_id: String,
    session_id: String,
    workspace_id: String,
    relative_path: String,
    source_type: String,
    sha256: String,
    size_bytes: i64,
    filename: String,
    content_type: String,
    remote_object_key: String,
}

pub async fn prepare_shared_upload<R: Runtime>(
    app: &tauri::AppHandle<R>,
    state: &tauri_plugin_db::ManagedState,
    operation: &DownloadOperation,
    attachment_id: &str,
    expected: &SharedUploadVersion,
) -> Result<PreparedSharedUpload> {
    operation.ensure_active()?;
    let attachment = load_shared_upload_attachment(state.pool(), attachment_id).await?;
    validate_shared_upload_version(&attachment, expected)?;
    let (_, source_path) = attachment_paths(app, &attachment)?;
    let cache_root = shared_upload_cache_root(app)?;
    create_shared_upload_cache_root(&cache_root).await?;
    let cache_id = Uuid::new_v4().to_string();
    let cache_path = shared_upload_cache_path(&cache_root, &cache_id)?;
    let cancellation = operation.cancellation().clone();
    let source_path_for_snapshot = source_path.clone();
    let cache_path_for_snapshot = cache_path.clone();
    let expected_sha256 = expected.sha256.clone();
    let cache_guard = tokio::task::spawn_blocking(move || {
        snapshot_verified_file(
            &source_path_for_snapshot,
            &cache_path_for_snapshot,
            expected.size_bytes,
            &expected_sha256,
            &cancellation,
        )
    })
    .await
    .map_err(|_| Error::CacheUnavailable)??;
    operation.ensure_active()?;
    let current = load_shared_upload_attachment(state.pool(), attachment_id).await?;
    validate_shared_upload_version(&current, expected)?;
    if current != attachment {
        return Err(Error::InvalidTransferState);
    }
    operation.begin_commit()?;
    cache_guard.disarm();
    Ok(PreparedSharedUpload {
        cache_id,
        sha256: expected.sha256.clone(),
        size_bytes: expected.size_bytes,
    })
}

pub async fn read_shared_upload_range<R: Runtime>(
    app: &tauri::AppHandle<R>,
    state: &tauri_plugin_db::ManagedState,
    attachment_id: &str,
    cache_id: &str,
    expected: &SharedUploadVersion,
    start: u64,
    end: u64,
) -> Result<Vec<u8>> {
    validate_range(start, end)?;
    let attachment = load_shared_upload_attachment(state.pool(), attachment_id).await?;
    validate_shared_upload_version(&attachment, expected)?;
    if end > expected.size_bytes {
        return Err(Error::InvalidRange);
    }
    read_range(
        shared_upload_cache_path(&shared_upload_cache_root(app)?, cache_id)?,
        start,
        end,
        expected.size_bytes,
    )
    .await
}

pub async fn validate_shared_upload<R: Runtime>(
    app: &tauri::AppHandle<R>,
    state: &tauri_plugin_db::ManagedState,
    operation: &DownloadOperation,
    attachment_id: &str,
    cache_id: &str,
    expected: &SharedUploadVersion,
) -> Result<bool> {
    operation.ensure_active()?;
    let attachment = match load_shared_upload_attachment(state.pool(), attachment_id).await {
        Ok(attachment) => attachment,
        Err(Error::LocalAttachmentUnavailable) => return Ok(false),
        Err(error) => return Err(error),
    };
    if validate_shared_upload_version(&attachment, expected).is_err() {
        return Ok(false);
    }
    let (_, source_path) = attachment_paths(app, &attachment)?;
    let cache_path = shared_upload_cache_path(&shared_upload_cache_root(app)?, cache_id)?;
    let cancellation = operation.cancellation().clone();
    let source_matches = file_matches_cancellable_async(
        source_path,
        expected.size_bytes,
        expected.sha256.clone(),
        cancellation.clone(),
    );
    let cache_matches = file_matches_cancellable_async(
        cache_path,
        expected.size_bytes,
        expected.sha256.clone(),
        cancellation,
    );
    let (source_matches, cache_matches) = tokio::join!(source_matches, cache_matches);
    Ok(source_matches? && cache_matches?)
}

pub async fn cleanup_shared_upload<R: Runtime>(
    app: &tauri::AppHandle<R>,
    cache_id: &str,
) -> Result<bool> {
    let path = shared_upload_cache_path(&shared_upload_cache_root(app)?, cache_id)?;
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub async fn download_shared_attachment<R: Runtime>(
    app: &tauri::AppHandle<R>,
    operation: &DownloadOperation,
    scope_id: &str,
    attachment_id: &str,
    signed_url: &str,
    expected_sha256: &str,
    expected_size_bytes: u64,
) -> Result<SharedAttachmentCacheResult> {
    operation.ensure_active()?;
    validate_opaque_id(scope_id)?;
    validate_opaque_id(attachment_id)?;
    if !valid_sha256(expected_sha256) || expected_size_bytes > MAX_PLAINTEXT_BYTES {
        return Err(Error::InvalidMetadata);
    }
    let url = validate_signed_download_url(
        signed_url,
        require_configured_supabase_url(crate::configured_supabase_url())?,
        DownloadObject::Shared(attachment_id),
    )?;
    let scope_path = shared_scope_path(app, scope_id)?;
    tokio::fs::create_dir_all(&scope_path).await?;
    let cache_id = shared_cache_id(scope_id, attachment_id);
    let local_path = cached_shared_attachment_path(&scope_path, &cache_id);
    if shared_cache_matches_async(
        scope_path.clone(),
        cache_id.clone(),
        expected_size_bytes,
        expected_sha256.to_string(),
    )
    .await?
    {
        return Ok(SharedAttachmentCacheResult {
            cache_id,
            local_path: local_path.to_string_lossy().into_owned(),
            size_bytes: expected_size_bytes,
            sha256: expected_sha256.to_string(),
        });
    }
    let temp = tempfile::NamedTempFile::new_in(&scope_path)?;
    let temp_path = temp.path().to_path_buf();
    download_to_path(
        url,
        &temp_path,
        expected_size_bytes,
        expected_sha256,
        false,
        operation.cancellation(),
    )
    .await?;
    temp.as_file().sync_all()?;
    operation.begin_commit()?;
    temp.persist(&local_path)
        .map_err(|error| Error::Io(error.error))?;
    commit_shared_cache_entry(
        &scope_path,
        &cache_id,
        &local_path,
        expected_size_bytes,
        expected_sha256,
    )?;
    Ok(SharedAttachmentCacheResult {
        cache_id,
        local_path: local_path.to_string_lossy().into_owned(),
        size_bytes: expected_size_bytes,
        sha256: expected_sha256.to_string(),
    })
}

pub async fn existing_shared_attachment_path<R: Runtime>(
    app: &tauri::AppHandle<R>,
    scope_id: &str,
    attachment_id: &str,
) -> Result<Option<String>> {
    validate_opaque_id(scope_id)?;
    validate_opaque_id(attachment_id)?;
    let scope_path = shared_scope_path(app, scope_id)?;
    let cache_id = shared_cache_id(scope_id, attachment_id);
    let Some((size, sha256)) = read_shared_cache_metadata(&scope_path, &cache_id)? else {
        return Ok(None);
    };
    if !shared_cache_matches_async(scope_path.clone(), cache_id.clone(), size, sha256).await? {
        return Ok(None);
    }
    Ok(Some(
        cached_shared_attachment_path(&scope_path, &cache_id)
            .to_string_lossy()
            .into_owned(),
    ))
}

pub async fn remove_shared_attachment<R: Runtime>(
    app: &tauri::AppHandle<R>,
    scope_id: &str,
    attachment_id: &str,
) -> Result<bool> {
    validate_opaque_id(scope_id)?;
    validate_opaque_id(attachment_id)?;
    let scope_path = shared_scope_path(app, scope_id)?;
    let cache_id = shared_cache_id(scope_id, attachment_id);
    let removed =
        match tokio::fs::remove_file(cached_shared_attachment_path(&scope_path, &cache_id)).await {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
    let _ = tokio::fs::remove_file(shared_cache_metadata_path(&scope_path, &cache_id)).await;
    Ok(removed)
}

pub async fn clear_shared_attachment_scope<R: Runtime>(
    app: &tauri::AppHandle<R>,
    scope_id: &str,
) -> Result<u64> {
    validate_opaque_id(scope_id)?;
    let path = shared_scope_path(app, scope_id)?;
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => Ok(1),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error.into()),
    }
}

pub async fn clear_shared_attachment_preview_scopes<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<bool> {
    let path = shared_preview_cache_root(app)?;
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn clear_shared_attachment_cache_root<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<()> {
    clear_attachment_cache_directory(&shared_cache_root(app)?)
}

pub(crate) fn clear_shared_upload_cache_root<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<()> {
    clear_attachment_cache_directory(&shared_upload_cache_root(app)?)
}

pub(crate) fn clear_shared_attachment_preview_cache_root<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<()> {
    clear_attachment_cache_directory(&shared_preview_cache_root(app)?)
}

async fn load_shared_upload_attachment(
    pool: &sqlx::SqlitePool,
    attachment_id: &str,
) -> Result<SharedUploadAttachment> {
    validate_opaque_id(attachment_id)?;
    sqlx::query_as::<_, SharedUploadAttachment>(
        "SELECT attachment.id AS attachment_id, attachment.session_id,
                attachment.workspace_id, attachment.relative_path, attachment.source_type,
                attachment.sha256, attachment.size_bytes, attachment.filename,
                attachment.content_type,
                -- This historical column is the shared-note object key, not private Sync state.
                attachment.cloud_object_key AS remote_object_key
         FROM session_attachments AS attachment
         JOIN attachment_local_state AS local
           ON local.attachment_id = attachment.id AND local.availability = 'present'
         WHERE attachment.id = ? AND attachment.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(Error::LocalAttachmentUnavailable)
}

fn validate_shared_upload_version(
    attachment: &SharedUploadAttachment,
    expected: &SharedUploadVersion,
) -> Result<()> {
    if !valid_sha256(&expected.sha256)
        || expected.size_bytes == 0
        || expected.size_bytes > MAX_PLAINTEXT_BYTES
        || attachment.sha256 != expected.sha256
        || u64::try_from(attachment.size_bytes).ok() != Some(expected.size_bytes)
        || attachment.filename != expected.filename
        || attachment.content_type != expected.content_type
        || attachment.remote_object_key != expected.remote_object_key
    {
        return Err(Error::InvalidTransferState);
    }
    Ok(())
}

fn attachment_paths<R: Runtime>(
    app: &tauri::AppHandle<R>,
    attachment: &SharedUploadAttachment,
) -> Result<(PathBuf, PathBuf)> {
    let components = Path::new(&attachment.relative_path)
        .components()
        .collect::<Vec<_>>();
    let valid = if attachment.source_type == "session_audio" {
        matches!(components.as_slice(), [Component::Normal(name)] if matches!(name.to_str(), Some("audio.mp3" | "audio.wav" | "audio.ogg")))
    } else {
        matches!(components.as_slice(), [Component::Normal(directory), Component::Normal(filename)] if directory == &std::ffi::OsStr::new("attachments") && !filename.is_empty())
    };
    if !valid || attachment.attachment_id.is_empty() || attachment.session_id.is_empty() {
        return Err(Error::InvalidMetadata);
    }
    let vault_base = app
        .settings()
        .vault_base()
        .map_err(|_| Error::Vault)?
        .into_std_path_buf();
    let session_candidate = anlg_fs_sync_core::FsSyncCore::new(vault_base.clone())
        .resolve_session_dir(&attachment.session_id)
        .map_err(|_| Error::LocalAttachmentUnavailable)?;
    let session_dir = anlg_fs_sync_core::resolve_path_inside_base(&vault_base, &session_candidate)
        .map_err(|_| Error::LocalAttachmentUnavailable)?;
    let path = anlg_fs_sync_core::resolve_path_inside_base(
        &session_dir,
        Path::new(&attachment.relative_path),
    )
    .map_err(|_| Error::LocalAttachmentUnavailable)?;
    Ok((session_dir, path))
}

fn validate_opaque_id(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(Error::InvalidMetadata);
    }
    Ok(())
}
