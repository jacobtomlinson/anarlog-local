use attachment_sync_core::{
    DownloadObject, require_configured_supabase_url, validate_signed_download_url,
};

#[test]
fn signed_urls_are_origin_and_path_bound() {
    let attachment_id = "00000000-0000-4000-8000-000000000003";
    let shared = format!(
        "https://project.supabase.co/storage/v1/object/sign/shared-note-attachments/00000000-0000-4000-8000-000000000001/00000000-0000-4000-8000-000000000002/{attachment_id}.sna1?token=secret"
    );
    assert!(
        validate_signed_download_url(
            &shared,
            "https://project.supabase.co",
            DownloadObject::Shared(attachment_id),
        )
        .is_ok()
    );
    assert!(
        validate_signed_download_url(
            &shared.replace("shared-note-attachments", "other-bucket"),
            "https://project.supabase.co",
            DownloadObject::Shared(attachment_id),
        )
        .is_err()
    );
    assert!(
        validate_signed_download_url(
            &shared.replace(attachment_id, "00000000-0000-4000-8000-000000000004"),
            "https://project.supabase.co",
            DownloadObject::Shared(attachment_id),
        )
        .is_err()
    );
}

#[test]
fn missing_or_empty_configured_origins_fail_closed() {
    assert!(require_configured_supabase_url(None).is_err());
    assert!(require_configured_supabase_url(Some("")).is_err());
    assert!(require_configured_supabase_url(Some("   ")).is_err());
    assert_eq!(
        require_configured_supabase_url(Some("https://project.supabase.co")).unwrap(),
        "https://project.supabase.co"
    );
}
