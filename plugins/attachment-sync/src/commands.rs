use crate::models::{PreparedSharedUpload, SharedAttachmentCacheResult, SharedUploadVersion};

#[tauri::command]
#[specta::specta]
pub(crate) async fn begin_shared_upload_operation(
    control: tauri::State<'_, crate::control::DownloadControl>,
    operation_id: String,
) -> Result<(), String> {
    control
        .begin(&operation_id, None)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn cancel_shared_upload_operation(
    control: tauri::State<'_, crate::control::DownloadControl>,
    operation_id: String,
) -> Result<bool, String> {
    control
        .cancel(&operation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn prepare_shared_upload<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    control: tauri::State<'_, crate::control::DownloadControl>,
    state: tauri::State<'_, tauri_plugin_db::ManagedState>,
    operation_id: String,
    attachment_id: String,
    expected: SharedUploadVersion,
) -> Result<PreparedSharedUpload, String> {
    let operation = control
        .start(&operation_id, None)
        .map_err(|error| error.to_string())?;
    crate::runtime::prepare_shared_upload(
        &app,
        state.inner(),
        &operation,
        &attachment_id,
        &expected,
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn read_shared_upload_range<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, tauri_plugin_db::ManagedState>,
    attachment_id: String,
    cache_id: String,
    expected: SharedUploadVersion,
    start: u64,
    end: u64,
) -> Result<Vec<u8>, String> {
    crate::runtime::read_shared_upload_range(
        &app,
        state.inner(),
        &attachment_id,
        &cache_id,
        &expected,
        start,
        end,
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn validate_shared_upload<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    control: tauri::State<'_, crate::control::DownloadControl>,
    state: tauri::State<'_, tauri_plugin_db::ManagedState>,
    operation_id: String,
    attachment_id: String,
    cache_id: String,
    expected: SharedUploadVersion,
) -> Result<bool, String> {
    let operation = control
        .start(&operation_id, None)
        .map_err(|error| error.to_string())?;
    crate::runtime::validate_shared_upload(
        &app,
        state.inner(),
        &operation,
        &attachment_id,
        &cache_id,
        &expected,
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn cleanup_shared_upload<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    cache_id: String,
) -> Result<bool, String> {
    crate::runtime::cleanup_shared_upload(&app, &cache_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_shared_attachment<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    operation_id: String,
    scope_id: String,
    attachment_id: String,
    signed_url: String,
    expected_sha256: String,
    expected_size_bytes: u64,
) -> Result<SharedAttachmentCacheResult, String> {
    let control = app.state::<crate::control::DownloadControl>();
    let operation = control
        .start(&operation_id, Some(&scope_id))
        .map_err(|error| error.to_string())?;
    crate::runtime::download_shared_attachment(
        &app,
        &operation,
        &scope_id,
        &attachment_id,
        &signed_url,
        &expected_sha256,
        expected_size_bytes,
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn shared_attachment_path<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    scope_id: String,
    attachment_id: String,
) -> Result<Option<String>, String> {
    crate::runtime::existing_shared_attachment_path(&app, &scope_id, &attachment_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn remove_shared_attachment<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    scope_id: String,
    attachment_id: String,
) -> Result<bool, String> {
    crate::runtime::remove_shared_attachment(&app, &scope_id, &attachment_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn clear_shared_attachment_scope<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    scope_id: String,
) -> Result<u64, String> {
    crate::runtime::clear_shared_attachment_scope(&app, &scope_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn clear_shared_attachment_preview_scopes<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<bool, String> {
    crate::runtime::clear_shared_attachment_preview_scopes(&app)
        .await
        .map_err(|error| error.to_string())
}
