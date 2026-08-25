use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SyncError>;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Anarlog Pro is required for shared note publication")]
    ProPlanRequired,

    #[error("Shared note publication is not permitted")]
    SnapshotPublicationForbidden,

    #[error("Shared note changed")]
    SnapshotChanged,

    #[error("Shared note service is unavailable")]
    SnapshotServiceUnavailable,

    #[error("Shared note is unavailable")]
    SharedNoteNotFound,

    #[error("Shared note invitation email is unavailable")]
    InvitationEmailUnavailable,

    #[error("Shared note recap email is unavailable")]
    RecapEmailUnavailable,

    #[error("Shared attachment access is not permitted")]
    SharedAttachmentForbidden,

    #[error("Shared attachment is unavailable")]
    SharedAttachmentNotFound,

    #[error("Shared attachment changed")]
    SharedAttachmentConflict,

    #[error("Shared attachment quota is exhausted")]
    SharedAttachmentQuotaExceeded,

    #[error("Shared attachment service is unavailable")]
    SharedAttachmentServiceUnavailable,

    #[error("Shared attachment verification is busy")]
    SharedAttachmentVerificationBusy,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for SyncError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::ProPlanRequired => (
                StatusCode::FORBIDDEN,
                "subscription_required",
                "Anarlog Pro is required for shared note publication".to_string(),
            ),
            Self::SnapshotPublicationForbidden => (
                StatusCode::FORBIDDEN,
                "shared_note_publication_forbidden",
                "Shared note publication is not permitted".to_string(),
            ),
            Self::SnapshotChanged => (
                StatusCode::CONFLICT,
                "snapshot_conflict",
                "Shared note changed; update Anarlog and reload before publishing again"
                    .to_string(),
            ),
            Self::SnapshotServiceUnavailable => (
                StatusCode::BAD_GATEWAY,
                "shared_note_service_unavailable",
                "Shared note service is unavailable".to_string(),
            ),
            Self::SharedNoteNotFound => (
                StatusCode::NOT_FOUND,
                "shared_note_not_found",
                "Shared note is unavailable".to_string(),
            ),
            Self::InvitationEmailUnavailable => (
                StatusCode::BAD_GATEWAY,
                "shared_note_invitation_email_unavailable",
                "Shared note invitation email is unavailable".to_string(),
            ),
            Self::RecapEmailUnavailable => (
                StatusCode::BAD_GATEWAY,
                "shared_note_recap_email_unavailable",
                "Shared note recap email is unavailable".to_string(),
            ),
            Self::SharedAttachmentForbidden => (
                StatusCode::FORBIDDEN,
                "shared_attachment_forbidden",
                "Shared attachment access is not permitted".to_string(),
            ),
            Self::SharedAttachmentNotFound => (
                StatusCode::NOT_FOUND,
                "shared_attachment_not_found",
                "Shared attachment is unavailable".to_string(),
            ),
            Self::SharedAttachmentConflict => (
                StatusCode::CONFLICT,
                "shared_attachment_conflict",
                "Shared attachment changed".to_string(),
            ),
            Self::SharedAttachmentQuotaExceeded => (
                StatusCode::INSUFFICIENT_STORAGE,
                "shared_attachment_quota_exceeded",
                "Shared attachment quota is exhausted".to_string(),
            ),
            Self::SharedAttachmentServiceUnavailable => (
                StatusCode::BAD_GATEWAY,
                "shared_attachment_service_unavailable",
                "Shared attachment service is unavailable".to_string(),
            ),
            Self::SharedAttachmentVerificationBusy => (
                StatusCode::SERVICE_UNAVAILABLE,
                "shared_attachment_verification_busy",
                "Shared attachment verification is busy".to_string(),
            ),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", message),
        };
        anlg_api_error::error_response(status, code, &message)
    }
}
