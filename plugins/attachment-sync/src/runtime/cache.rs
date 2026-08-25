use std::path::{Path, PathBuf};

use anlg_attachment_sync_core::{Error, Result, TransferPaths, clear_attachment_cache_directory};
use tauri::{Manager, Runtime};

pub(super) use anlg_attachment_sync_core::{
    cached_shared_attachment_path, commit_shared_cache_entry, create_shared_upload_cache_root,
    file_matches_cancellable_async, read_range, read_shared_cache_metadata, shared_cache_id,
    shared_cache_matches_async, shared_cache_metadata_path, shared_upload_cache_path,
    snapshot_verified_file, valid_sha256, validate_range,
};

pub(super) fn shared_upload_cache_root<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf> {
    Ok(cache_paths(app)?.shared_upload_cache_root())
}

pub(super) fn shared_scope_path<R: Runtime>(
    app: &tauri::AppHandle<R>,
    scope_id: &str,
) -> Result<PathBuf> {
    cache_paths(app)?.shared_scope_path(scope_id)
}

pub(super) fn shared_cache_root<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf> {
    Ok(cache_paths(app)?.shared_cache_root())
}

pub(super) fn shared_preview_cache_root<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf> {
    Ok(cache_paths(app)?.shared_preview_cache_root())
}

fn cache_paths<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<TransferPaths> {
    let cache_base = app
        .path()
        .app_cache_dir()
        .map_err(|_| Error::CacheUnavailable)?;
    Ok(TransferPaths::new(cache_base))
}
