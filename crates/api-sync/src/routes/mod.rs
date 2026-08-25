use axum::Router;

use crate::state::AppState;

mod session_shares;
mod shared_attachments;

#[cfg(test)]
use session_shares::MAX_SNAPSHOT_RESPONSE_BYTES;
pub(crate) use session_shares::{SharedNoteAttachment, validate_shared_attachments};

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = session_shares::openapi();
    openapi.merge(shared_attachments::openapi());
    openapi
}

pub fn session_share_router(state: AppState) -> Router {
    session_shares::router()
        .merge(shared_attachments::router())
        .with_state(state)
}

pub fn web_edit_router(state: AppState) -> Router {
    session_shares::web_edit_router().with_state(state)
}
