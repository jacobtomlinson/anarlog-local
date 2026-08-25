-- Retire the user-facing shared-workspace capability without rewriting
-- migrations that may already have been applied. Existing tables, records,
-- and personal-workspace/session-sharing paths remain available.

BEGIN;

-- E2EE witness access is personal-workspace-only. The API also enforces this
-- boundary, but keep the service-role RPCs safe if they are called elsewhere.
CREATE OR REPLACE FUNCTION private.active_e2ee_freshness_key_id(
  p_actor_user_id uuid,
  p_workspace_id uuid
)
RETURNS text
LANGUAGE sql
STABLE
SECURITY INVOKER
SET search_path = ''
AS $$
  SELECT workspace.e2ee_key_id
  FROM public.workspaces AS workspace
  WHERE workspace.id = p_workspace_id
    AND workspace.id = p_actor_user_id
    AND workspace.owner_user_id = p_actor_user_id
    AND workspace.kind = 'personal'
    AND workspace.deleted_at IS NULL
    AND workspace.e2ee_key_id IS NOT NULL;
$$;

-- Keep the public session-sharing entry point, but retire the workspace scope
-- and workspace target. Legacy rows are still readable by existing clients.
CREATE OR REPLACE FUNCTION private.protected_set_session_share_scope(
  p_share_id uuid,
  p_general_scope text,
  p_general_workspace_id uuid DEFAULT NULL
)
RETURNS TABLE (
  share_id uuid,
  general_scope text,
  general_workspace_id uuid,
  public_slug text,
  access_version bigint
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  v_result record;
  v_share public.session_shares%ROWTYPE;
BEGIN
  IF p_general_scope = 'workspace' OR p_general_workspace_id IS NOT NULL THEN
    RAISE EXCEPTION 'workspace session sharing is no longer available'
      USING ERRCODE = '42501';
  END IF;

  SELECT share.*
  INTO v_share
  FROM public.session_shares AS share
  WHERE share.id = p_share_id;

  IF FOUND THEN
    PERFORM private.assert_allowed_share_scope(
      v_share.workspace_id,
      p_general_scope
    );
  END IF;

  SELECT *
  INTO v_result
  FROM private.set_session_share_scope(
    p_share_id,
    p_general_scope,
    NULL
  );

  IF p_general_scope <> 'restricted' THEN
    PERFORM private.require_hyprnote_pro_entitlement();
  END IF;

  RETURN QUERY
  SELECT
    v_result.share_id,
    v_result.general_scope,
    v_result.general_workspace_id,
    v_result.public_slug,
    v_result.access_version;
END;
$$;

-- The raw implementation is an internal dependency of the protected wrapper;
-- prevent authenticated callers from bypassing the new guard through the
-- private schema.
REVOKE ALL ON FUNCTION private.set_session_share_scope(uuid, text, uuid)
  FROM PUBLIC, anon, authenticated;

-- Stop automatic membership creation for users matching a shared-workspace
-- domain. Keep the function and data for service-side compatibility.
DROP TRIGGER IF EXISTS on_auth_user_domain_capture ON auth.users;

-- Shared workspace lifecycle, membership, invitation, policy, usage, domain,
-- and slug RPCs are retired. Revoking both schemas prevents direct private
-- schema calls from bypassing the public API retirement.
REVOKE ALL ON FUNCTION private.create_workspace(text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.rename_workspace(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.set_workspace_membership_role(uuid, uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.transfer_workspace_ownership(uuid, uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.leave_workspace(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.delete_workspace(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.protected_create_workspace(text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.protected_rename_workspace(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.protected_set_workspace_membership_role(uuid, uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.protected_transfer_workspace_ownership(uuid, uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.protected_create_workspace_invitation(uuid, text)
  FROM PUBLIC, anon, authenticated;

REVOKE ALL ON FUNCTION public.create_workspace(text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.rename_workspace(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.set_workspace_membership_role(uuid, uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.transfer_workspace_ownership(uuid, uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.leave_workspace(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.delete_workspace(uuid)
  FROM PUBLIC, anon, authenticated;

REVOKE ALL ON FUNCTION private.create_workspace_invitation(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.accept_workspace_invitation(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.revoke_workspace_invitation(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.revoke_workspace_membership(uuid, uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.list_workspace_invitations(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.list_workspace_memberships(uuid)
  FROM PUBLIC, anon, authenticated;

REVOKE ALL ON FUNCTION public.create_workspace_invitation(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.accept_workspace_invitation(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.revoke_workspace_invitation(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.revoke_workspace_membership(uuid, uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.list_workspace_invitations(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.list_workspace_memberships(uuid)
  FROM PUBLIC, anon, authenticated;

REVOKE ALL ON FUNCTION private.get_workspace_policy(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.get_workspace_policy(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.set_workspace_policy(
  uuid,
  text[],
  text,
  integer,
  boolean,
  boolean,
  boolean
) FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.get_workspace_usage_overview(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.claim_workspace_domain(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.rotate_workspace_scim_token(uuid, text, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.scim_apply_user(text, text, boolean)
  FROM PUBLIC, anon, authenticated, service_role;
REVOKE ALL ON FUNCTION public.scim_apply_user_id(text, uuid, boolean)
  FROM PUBLIC, anon, authenticated, service_role;

REVOKE ALL ON FUNCTION private.set_workspace_share_slug(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.get_session_share_workspace_slug(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.set_workspace_share_slug(uuid, text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.get_session_share_workspace_slug(uuid)
  FROM PUBLIC, anon, authenticated;

-- Shared-workspace E2EE grant and member-identity RPCs are no longer part of
-- the client protocol. Keep the underlying records for compatibility and
-- service-side cleanup.
REVOKE ALL ON FUNCTION private.publish_e2ee_member_identity(text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.list_workspace_key_recipients(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.set_workspace_e2ee_key(uuid, text, jsonb)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.list_my_workspace_e2ee_grants(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION private.list_all_my_workspace_e2ee_grants()
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.publish_e2ee_member_identity(text)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.list_workspace_key_recipients(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.set_workspace_e2ee_key(uuid, text, jsonb)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.list_my_workspace_e2ee_grants(uuid)
  FROM PUBLIC, anon, authenticated;
REVOKE ALL ON FUNCTION public.list_all_my_workspace_e2ee_grants()
  FROM PUBLIC, anon, authenticated;

COMMIT;
