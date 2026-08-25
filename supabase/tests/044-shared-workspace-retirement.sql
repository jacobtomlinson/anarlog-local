begin;
select plan(13);

select tests.create_supabase_user('retired_workspace_owner', 'retired-workspace-owner@example.com');

create temporary table retirement_test_state (
  workspace_id uuid,
  share_id uuid
);
grant all on retirement_test_state to authenticated, service_role;

select ok(
  has_function_privilege(
    'authenticated',
    'public.create_session_share(uuid,text)',
    'EXECUTE'
  )
    and has_function_privilege(
      'authenticated',
      'public.set_session_share_scope(uuid,text,uuid)',
      'EXECUTE'
    ),
  'Personal session-sharing capability remains available'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.create_workspace(text)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.rename_workspace(uuid,text)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.set_workspace_membership_role(uuid,uuid,text)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.transfer_workspace_ownership(uuid,uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.leave_workspace(uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.delete_workspace(uuid)',
      'EXECUTE'
    ),
  'Shared-workspace lifecycle RPCs are retired'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.create_workspace_invitation(uuid,text)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.accept_workspace_invitation(uuid,text)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.revoke_workspace_invitation(uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.revoke_workspace_membership(uuid,uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.list_workspace_invitations(uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.list_workspace_memberships(uuid)',
      'EXECUTE'
    ),
  'Membership and invitation RPCs are retired'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.get_workspace_policy(uuid)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.set_workspace_policy(uuid,text[],text,integer,boolean,boolean,boolean)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.get_workspace_usage_overview(uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.claim_workspace_domain(uuid,text)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.rotate_workspace_scim_token(uuid,text,text)',
      'EXECUTE'
    ),
  'Workspace policy, usage, domain, and SCIM setup RPCs are retired'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.scim_apply_user(text,text,boolean)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'service_role',
      'public.scim_apply_user(text,text,boolean)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.scim_apply_user_id(text,uuid,boolean)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'service_role',
      'public.scim_apply_user_id(text,uuid,boolean)',
      'EXECUTE'
    ),
  'SCIM membership mutators cannot be called by clients or old providers'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.publish_e2ee_member_identity(text)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.list_workspace_key_recipients(uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.set_workspace_e2ee_key(uuid,text,jsonb)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.list_my_workspace_e2ee_grants(uuid)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'authenticated',
      'public.list_all_my_workspace_e2ee_grants()',
      'EXECUTE'
    ),
  'Shared-workspace E2EE grant RPCs are retired'
);

select tests.authenticate_as_service_role();

insert into retirement_test_state (workspace_id)
values (gen_random_uuid());

insert into public.workspaces (id, owner_user_id, kind, name)
select workspace_id, tests.get_supabase_uid('retired_workspace_owner'), 'shared', 'Legacy'
from retirement_test_state;

insert into public.workspace_memberships (workspace_id, user_id, role)
select workspace_id, tests.get_supabase_uid('retired_workspace_owner'), 'owner'
from retirement_test_state;

select results_eq(
  $$
    select count(*)
    from public.workspaces
    where id = (select workspace_id from retirement_test_state)
      and kind = 'shared'
  $$,
  array[1::bigint],
  'Legacy shared-workspace records remain stored'
);

select tests.clear_authentication();
select tests.authenticate_as_hyprnote_pro('retired_workspace_owner');

select results_eq(
  $$
    select count(*)
    from public.workspaces
    where id = auth.uid()
      and kind = 'personal'
  $$,
  array[1::bigint],
  'Personal workspace records remain readable'
);

select lives_ok(
  $$
    insert into retirement_test_state (share_id)
    select share_id
    from public.create_session_share(auth.uid(), 'retained-personal-session')
  $$,
  'Personal session shares can still be created'
);

select tests.clear_authentication();
select tests.authenticate_as_service_role();

select lives_ok(
  $$
    select *
    from public.publish_session_share_snapshot(
      (select share_id from retirement_test_state where share_id is not null),
      tests.get_supabase_uid('retired_workspace_owner'),
      'Legacy session read',
      '{"type":"doc","content":[{"type":"paragraph"}]}'::jsonb
    )
  $$,
  'Trusted service code can retain a personal session snapshot'
);

select tests.clear_authentication();
select tests.authenticate_as_hyprnote_pro('retired_workspace_owner');

select results_eq(
  $$
    select title, body_json
    from public.read_my_session_share_snapshot(
      (select share_id from retirement_test_state where share_id is not null)
    )
  $$,
  $$values (
    'Legacy session read'::text,
    '{"type":"doc","content":[{"type":"paragraph"}]}'::jsonb
  )$$,
  'Personal session snapshot reads remain compatible'
);

select throws_ok(
  $$
    select *
    from public.set_session_share_scope(
      (select share_id from retirement_test_state where share_id is not null),
      'workspace',
      (select workspace_id from retirement_test_state)
    )
  $$,
  '42501',
  'workspace session sharing is no longer available',
  'Shared-workspace session scopes are rejected'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.get_session_share_workspace_slug(uuid)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.set_workspace_share_slug(uuid,text)',
      'EXECUTE'
    ),
  'Shared-workspace share hosts are retired while records remain'
);

select * from finish();
rollback;
