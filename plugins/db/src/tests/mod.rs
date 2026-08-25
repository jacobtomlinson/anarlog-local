mod bindings;
mod cloudsync_credentials;
mod cloudsync_lifecycle;
mod queries;
pub(crate) mod support;
mod transactions;

#[test]
fn cloudsync_projection_rejects_shared_workspace_kind() {
    let projection = serde_json::json!({
        "accountUserId": "user-a",
        "personalWorkspaceId": "user-a",
        "workspaces": [{
            "id": "workspace-shared",
            "ownerUserId": "user-b",
            "kind": "shared",
            "name": "Shared",
            "membershipId": "membership-shared",
            "role": "member",
            "membershipCreatedAt": "2026-07-16T00:01:00Z",
            "membershipUpdatedAt": "2026-07-16T00:02:00Z",
            "createdAt": "2026-07-16T00:00:00Z",
            "updatedAt": "2026-07-16T00:00:00Z"
        }]
    });

    assert!(serde_json::from_value::<crate::CloudsyncWorkspaceProjection>(projection).is_err());
}
