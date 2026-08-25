mod cache;
mod control;
mod download;
mod error;
mod layout;

pub use cache::{
    MAX_RANGE_BYTES, cached_shared_attachment_path, clear_attachment_cache_directory,
    commit_shared_cache_entry, create_shared_upload_cache_root, file_matches_cancellable_async,
    hash_identifier, hex_digest, read_range, read_shared_cache_metadata, shared_cache_id,
    shared_cache_matches, shared_cache_matches_async, shared_cache_metadata_path,
    shared_upload_cache_path, snapshot_verified_file, valid_cache_id, valid_sha256, validate_range,
    write_shared_cache_metadata,
};
pub use control::{DownloadControl, DownloadOperation, SharedScopeClear, SharedScopePrefixClear};
pub use download::{
    DownloadObject, download_to_path, require_configured_supabase_url, validate_signed_download_url,
};
pub use error::{Error, Result};
pub use layout::{SHARED_PREVIEW_SCOPE_PREFIX, TransferPaths};
pub const MAX_PLAINTEXT_BYTES: u64 = 545_259_520;
