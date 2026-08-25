begin;
select plan(8);

select tests.create_supabase_user('share_owner', 'share-owner@example.com');
select tests.authenticate_as_hyprnote_pro('share_owner');

create temporary table session_sharing_test_state (
  share_id uuid
);
grant all on session_sharing_test_state to authenticated, service_role;

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
  'Personal session sharing keeps its client RPCs'
);

select lives_ok(
  $$
    insert into session_sharing_test_state (share_id)
    select share_id
    from public.create_session_share(auth.uid(), 'personal-session')
  $$,
  'A personal workspace can create a restricted session share'
);

select results_eq(
  $$
    select general_scope, general_workspace_id
    from public.get_session_share_management(
      (select share_id from session_sharing_test_state)
    )
  $$,
  $$values ('restricted'::text, null::uuid)$$,
  'Personal session shares start restricted without a workspace target'
);

select throws_ok(
  $$
    select *
    from public.set_session_share_scope(
      (select share_id from session_sharing_test_state),
      'workspace',
      auth.uid()
    )
  $$,
  '42501',
  'workspace session sharing is no longer available',
  'Workspace session scopes are rejected'
);

select lives_ok(
  $$
    select *
    from public.set_session_share_scope(
      (select share_id from session_sharing_test_state),
      'restricted'
    )
  $$,
  'Restricted personal session sharing remains available'
);

select ok(
  not has_table_privilege('authenticated', 'public.session_shares', 'SELECT')
    and not has_table_privilege(
      'authenticated',
      'public.session_share_links',
      'SELECT'
    ),
  'Session-sharing authority remains gateway-only'
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
  'Workspace-specific share hosts are retired'
);

select ok(
  not has_function_privilege(
    'authenticated',
    'public.create_workspace_invitation(uuid,text)',
    'EXECUTE'
  ),
  'Workspace invitation flows are not part of session sharing'
);

select * from finish();
rollback;
