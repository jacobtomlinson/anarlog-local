const COMMANDS: &[&str] = &[
    "begin_shared_upload_operation",
    "cancel_shared_upload_operation",
    "prepare_shared_upload",
    "read_shared_upload_range",
    "validate_shared_upload",
    "cleanup_shared_upload",
    "download_shared_attachment",
    "shared_attachment_path",
    "remove_shared_attachment",
    "clear_shared_attachment_scope",
    "clear_shared_attachment_preview_scopes",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
