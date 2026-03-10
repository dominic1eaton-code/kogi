PRAGMA foreign_keys = ON;

INSERT OR IGNORE INTO auth_account (id, email, password_hash)
VALUES
  ('11111111-1111-1111-1111-111111111111', 'owner@kogi.local', 'hash-placeholder');

INSERT OR IGNORE INTO auth_profile (id, account_id, handle, display_name)
VALUES
  ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111111', 'kogi-owner', 'Kogi Owner');

INSERT OR IGNORE INTO portfolio_workspace (id, account_id, workspace_type, name)
VALUES
  ('33333333-3333-3333-3333-333333333333', '11111111-1111-1111-1111-111111111111', 'personal', 'Owner Workspace');

INSERT OR IGNORE INTO portfolio_portfolio (id, workspace_id, owner_id, name, domain, visibility)
VALUES
  ('55555555-5555-5555-5555-555555555555', '33333333-3333-3333-3333-333333333333', '11111111-1111-1111-1111-111111111111', 'Kogi Root Portfolio', 'operations', 'private');

INSERT OR IGNORE INTO auth_identity (id, account_id, display_name, status)
VALUES
  ('66666666-6666-6666-6666-666666666666', '11111111-1111-1111-1111-111111111111', 'Dominic Worker', 'active');

INSERT OR IGNORE INTO auth_identity_persona (identity_id, persona_type)
VALUES
  ('66666666-6666-6666-6666-666666666666', 'investor'),
  ('66666666-6666-6666-6666-666666666666', 'developer'),
  ('66666666-6666-6666-6666-666666666666', 'donor');

INSERT OR IGNORE INTO auth_identity_role (identity_id, role_name)
VALUES
  ('66666666-6666-6666-6666-666666666666', 'owner'),
  ('66666666-6666-6666-6666-666666666666', 'admin');

INSERT OR IGNORE INTO auth_identity_worker_type (identity_id, worker_type)
VALUES
  ('66666666-6666-6666-6666-666666666666', 'freelancer'),
  ('66666666-6666-6666-6666-666666666666', 'consultant'),
  ('66666666-6666-6666-6666-666666666666', 'entrepreneur');

INSERT OR IGNORE INTO auth_identity_profile (id, identity_id, profile_type, name, settings, options, parameters, status)
VALUES
  (
    '77777777-7777-7777-7777-777777777771',
    '66666666-6666-6666-6666-666666666666',
    'personal',
    'Personal Profile',
    '{"theme":"light","timezone":"America/Chicago"}',
    '{"notifications":true}',
    '{"focus_hours":2}',
    'active'
  ),
  (
    '77777777-7777-7777-7777-777777777772',
    '66666666-6666-6666-6666-666666666666',
    'work',
    'Work Profile',
    '{"default_workspace":"work"}',
    '{"time_tracking":true}',
    '{"billing_rate":115}',
    'active'
  ),
  (
    '77777777-7777-7777-7777-777777777773',
    '66666666-6666-6666-6666-666666666666',
    'business',
    'Business Profile',
    '{"fiscal_year_start":"2026-01-01"}',
    '{"automated_distributions":true}',
    '{"approval_threshold":0.66}',
    'active'
  ),
  (
    '77777777-7777-7777-7777-777777777774',
    '66666666-6666-6666-6666-666666666666',
    'community',
    'Community Profile',
    '{"visibility":"public"}',
    '{"allow_dm":true}',
    '{"weekly_events":3}',
    'active'
  );

INSERT OR IGNORE INTO auth_profile_workspace_binding (profile_id, workspace_id)
VALUES
  ('77777777-7777-7777-7777-777777777771', '33333333-3333-3333-3333-333333333333'),
  ('77777777-7777-7777-7777-777777777772', '33333333-3333-3333-3333-333333333333'),
  ('77777777-7777-7777-7777-777777777773', '33333333-3333-3333-3333-333333333333'),
  ('77777777-7777-7777-7777-777777777774', '33333333-3333-3333-3333-333333333333');

INSERT OR IGNORE INTO auth_profile_portfolio_binding (profile_id, portfolio_id)
VALUES
  ('77777777-7777-7777-7777-777777777771', '55555555-5555-5555-5555-555555555555'),
  ('77777777-7777-7777-7777-777777777772', '55555555-5555-5555-5555-555555555555'),
  ('77777777-7777-7777-7777-777777777773', '55555555-5555-5555-5555-555555555555'),
  ('77777777-7777-7777-7777-777777777774', '55555555-5555-5555-5555-555555555555');

INSERT OR IGNORE INTO auth_connection_registry (profile_id, connection_type, provider, account_ref, metadata, status)
VALUES
  ('77777777-7777-7777-7777-777777777772', 'software', 'github', 'domin-dev', '{"scope":"work"}', 'active'),
  ('77777777-7777-7777-7777-777777777773', 'finance', 'stripe', 'acct_001', '{"scope":"business"}', 'active'),
  ('77777777-7777-7777-7777-777777777774', 'community', 'slack', 'kogi-community', '{"scope":"community"}', 'active');

INSERT OR IGNORE INTO auth_profile_tool (profile_id, tool_name, tool_category, metadata)
VALUES
  ('77777777-7777-7777-7777-777777777772', 'Sprint Planner', 'planning', '{}'),
  ('77777777-7777-7777-7777-777777777773', 'Capital Dashboard', 'finance', '{}'),
  ('77777777-7777-7777-7777-777777777774', 'Community Room', 'social', '{}');

INSERT OR IGNORE INTO auth_profile_program (profile_id, program_name, status, metadata)
VALUES
  ('77777777-7777-7777-7777-777777777772', 'Consulting Practice', 'active', '{}'),
  ('77777777-7777-7777-7777-777777777773', 'Revenue Operations', 'active', '{}'),
  ('77777777-7777-7777-7777-777777777774', 'Mutual Aid Network', 'active', '{}');

INSERT OR IGNORE INTO auth_profile_project (profile_id, project_ref, source_module, status, metadata)
VALUES
  ('77777777-7777-7777-7777-777777777772', 'kogi-mvp', 'wbs', 'active', '{}'),
  ('77777777-7777-7777-7777-777777777773', 'fundraise-seed', 'fund', 'active', '{}'),
  ('77777777-7777-7777-7777-777777777774', 'co-op-launch', 'community', 'active', '{}');

INSERT OR IGNORE INTO auth_contact_directory (profile_id, contact_name, email, phone, organization, relationship_type, status, metadata)
VALUES
  ('77777777-7777-7777-7777-777777777772', 'Client Alpha', 'client.alpha@example.com', '+1-555-0101', 'Alpha Org', 'client', 'active', '{}'),
  ('77777777-7777-7777-7777-777777777774', 'Community Lead', 'lead@community.org', '+1-555-0110', 'Worker Co-op', 'partner', 'active', '{}');

INSERT OR IGNORE INTO auth_asset_vault (profile_id, asset_type, name, value_amount, currency, metadata)
VALUES
  ('77777777-7777-7777-7777-777777777773', 'capital', 'Seed Reserve', 25000.0000, 'USD', '{}'),
  ('77777777-7777-7777-7777-777777777772', 'software', 'IDE License', 299.0000, 'USD', '{}');

INSERT OR IGNORE INTO exchange_wallet (id, account_id, wallet_type, currency, available, reserved)
VALUES
  ('44444444-4444-4444-4444-444444444444', '11111111-1111-1111-1111-111111111111', 'operations', 'USD', 12500.0000, 1000.0000);

INSERT OR IGNORE INTO kern_module_registry (module_id, module_kind, language)
VALUES
  ('kogi.office', 'office', 'rust'),
  ('kogi.bank', 'bank', 'go'),
  ('kogi.exchange', 'exchange', 'go'),
  ('kogi.community', 'community', 'go');
