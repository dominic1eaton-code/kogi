-- =============================================================================
--  Kogi Platform — seed.sql  (Full Demonstration Dataset)
--
--  Scenario: Three users in a Kogi ecosystem
--   • owner    (Dominic)  — platform owner / admin / full-stack developer
--   • user1    (Amara)    — real estate investor / playbook creator
--   • user2    (Kwame)    — aspiring real estate investor / buyer
--   • user3    (Zuri)     — real estate mastermind host
--
--  Covers every schema introduced in schema.sql.
-- =============================================================================

-- =============================================================================
--  AUTH.ACCOUNT
-- =============================================================================

INSERT INTO auth.account (id, email, password_hash, status) VALUES
  ('11111111-1111-1111-1111-111111111111', 'owner@kogi.local',  'hash-owner',  'active'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'amara@kogi.local',  'hash-amara',  'active'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'kwame@kogi.local',  'hash-kwame',  'active'),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'zuri@kogi.local',   'hash-zuri',   'active')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  AUTH.PROFILE  (primary public handle)
-- =============================================================================

INSERT INTO auth.profile (id, account_id, handle, display_name, persona, timezone) VALUES
  ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111111', 'kogi-owner', 'Dominic Walker',   'platform_admin',       'America/Chicago'),
  ('a2222222-2222-2222-2222-222222222222', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'amara-re',   'Amara Osei',       'investor',             'America/New_York'),
  ('b2222222-2222-2222-2222-222222222222', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'kwame-dev',  'Kwame Mensah',     'entrepreneur',         'America/Los_Angeles'),
  ('c2222222-2222-2222-2222-222222222222', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'zuri-master','Zuri Diallo',      'community_organizer',  'America/New_York')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  AUTH.IDENTITY
-- =============================================================================

INSERT INTO auth.identity (id, account_id, display_name, bio, status) VALUES
  ('66666666-6666-6666-6666-666666666666', '11111111-1111-1111-1111-111111111111', 'Dominic Walker',  'Platform founder & full-stack developer', 'active'),
  ('a6666666-6666-6666-6666-666666666666', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Amara Osei',      'Seasoned real estate investor & educator', 'active'),
  ('b6666666-6666-6666-6666-666666666666', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Kwame Mensah',    'Aspiring RE investor & software engineer',  'active'),
  ('c6666666-6666-6666-6666-666666666666', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'Zuri Diallo',     'Mastermind host & community builder',       'active')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  AUTH.IDENTITY_PERSONA
-- =============================================================================

INSERT INTO auth.identity_persona (identity_id, persona_type) VALUES
  ('66666666-6666-6666-6666-666666666666', 'investor'),
  ('66666666-6666-6666-6666-666666666666', 'developer'),
  ('66666666-6666-6666-6666-666666666666', 'donor'),
  ('66666666-6666-6666-6666-666666666666', 'entrepreneur'),
  ('a6666666-6666-6666-6666-666666666666', 'investor'),
  ('a6666666-6666-6666-6666-666666666666', 'educator'),
  ('a6666666-6666-6666-6666-666666666666', 'creator'),
  ('b6666666-6666-6666-6666-666666666666', 'developer'),
  ('b6666666-6666-6666-6666-666666666666', 'investor'),
  ('b6666666-6666-6666-6666-666666666666', 'entrepreneur'),
  ('c6666666-6666-6666-6666-666666666666', 'community_organizer'),
  ('c6666666-6666-6666-6666-666666666666', 'educator'),
  ('c6666666-6666-6666-6666-666666666666', 'investor')
ON CONFLICT (identity_id, persona_type) DO NOTHING;

-- =============================================================================
--  AUTH.IDENTITY_ROLE
-- =============================================================================

INSERT INTO auth.identity_role (identity_id, role_name) VALUES
  ('66666666-6666-6666-6666-666666666666', 'owner'),
  ('66666666-6666-6666-6666-666666666666', 'admin'),
  ('a6666666-6666-6666-6666-666666666666', 'publisher'),
  ('a6666666-6666-6666-6666-666666666666', 'creator'),
  ('b6666666-6666-6666-6666-666666666666', 'subscriber'),
  ('c6666666-6666-6666-6666-666666666666', 'community_lead')
ON CONFLICT (identity_id, role_name) DO NOTHING;

-- =============================================================================
--  AUTH.IDENTITY_WORKER_TYPE
-- =============================================================================

INSERT INTO auth.identity_worker_type (identity_id, worker_type) VALUES
  ('66666666-6666-6666-6666-666666666666', 'freelancer'),
  ('66666666-6666-6666-6666-666666666666', 'consultant'),
  ('66666666-6666-6666-6666-666666666666', 'entrepreneur'),
  ('a6666666-6666-6666-6666-666666666666', 'consultant'),
  ('a6666666-6666-6666-6666-666666666666', 'entrepreneur'),
  ('b6666666-6666-6666-6666-666666666666', 'freelancer'),
  ('b6666666-6666-6666-6666-666666666666', 'entrepreneur'),
  ('c6666666-6666-6666-6666-666666666666', 'entrepreneur'),
  ('c6666666-6666-6666-6666-666666666666', 'consultant')
ON CONFLICT (identity_id, worker_type) DO NOTHING;

-- =============================================================================
--  AUTH.RBAC_ROLE
-- =============================================================================

INSERT INTO auth.rbac_role (name, description, permissions) VALUES
  ('platform_admin',    'Full platform administration',              ARRAY['*']),
  ('owner',             'Portfolio & content owner',                 ARRAY['portfolio:*','component:*','market:sell','exchange:trade']),
  ('editor',            'Can edit components and create content',    ARRAY['component:edit','component:create','market:list']),
  ('subscriber',        'Can subscribe and consume content',         ARRAY['component:read','market:buy','community:post']),
  ('viewer',            'Read-only access',                          ARRAY['component:read','community:read']),
  ('community_lead',    'Can manage community spaces',               ARRAY['community:*','component:read']),
  ('publisher',         'Can publish and sell content',              ARRAY['market:sell','market:list','component:publish'])
ON CONFLICT (name) DO NOTHING;

-- =============================================================================
--  AUTH.ACCOUNT_ROLE
-- =============================================================================

INSERT INTO auth.account_role (account_id, role_name, scope) VALUES
  ('11111111-1111-1111-1111-111111111111', 'platform_admin', 'platform'),
  ('11111111-1111-1111-1111-111111111111', 'owner',          'platform'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'publisher',      'platform'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'owner',          'platform'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'subscriber',     'platform'),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'community_lead', 'platform'),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'owner',          'platform')
ON CONFLICT (account_id, role_name, scope, scope_ref) DO NOTHING;

-- =============================================================================
--  AUTH.IDENTITY_PROFILE (typed profiles per identity)
-- =============================================================================

INSERT INTO auth.identity_profile (id, identity_id, profile_type, name, handle, bio, settings, options, parameters, visibility, status) VALUES
  -- Dominic
  ('77777777-7777-7777-7777-777777777771', '66666666-6666-6666-6666-666666666666', 'personal',   'Personal Profile',  'dominic-personal',  'Day-to-day personal',   '{"theme":"dark","timezone":"America/Chicago"}'::jsonb, '{"notifications":true}'::jsonb,   '{"focus_hours":2}'::jsonb,            'private', 'active'),
  ('77777777-7777-7777-7777-777777777772', '66666666-6666-6666-6666-666666666666', 'work',       'Work Profile',      'dominic-work',      'Dev & consulting work',  '{"default_workspace":"work"}'::jsonb,               '{"time_tracking":true}'::jsonb,    '{"billing_rate":115}'::jsonb,         'private', 'active'),
  ('77777777-7777-7777-7777-777777777773', '66666666-6666-6666-6666-666666666666', 'business',   'Business Profile',  'dominic-biz',       'Kogi platform ventures', '{"fiscal_year_start":"2026-01-01"}'::jsonb,         '{"automated_distributions":true}'::jsonb,'{"approval_threshold":0.66}'::jsonb, 'private', 'active'),
  ('77777777-7777-7777-7777-777777777774', '66666666-6666-6666-6666-666666666666', 'community',  'Community Profile', 'dominic-community', 'Community builder',      '{"visibility":"public"}'::jsonb,                    '{"allow_dm":true}'::jsonb,         '{"weekly_events":3}'::jsonb,          'public',  'active'),
  -- Amara
  ('a7777777-7777-7777-7777-777777777771', 'a6666666-6666-6666-6666-666666666666', 'personal',   'Personal',          'amara-personal',    'Private life portfolio', '{"theme":"dark"}'::jsonb,                           '{"notifications":true}'::jsonb,    '{}'::jsonb,                           'private', 'active'),
  ('a7777777-7777-7777-7777-777777777772', 'a6666666-6666-6666-6666-666666666666', 'work',       'Investor Profile',  'amara-investor',    'Real estate portfolio',  '{"default_workspace":"re_investment"}'::jsonb,      '{"show_returns":true}'::jsonb,     '{"billing_rate":250}'::jsonb,         'public',  'active'),
  -- Kwame
  ('b7777777-7777-7777-7777-777777777771', 'b6666666-6666-6666-6666-666666666666', 'personal',   'Personal',          'kwame-personal',    'My personal portfolio',  '{"theme":"dark"}'::jsonb,                           '{"notifications":true}'::jsonb,    '{}'::jsonb,                           'private', 'active'),
  ('b7777777-7777-7777-7777-777777777772', 'b6666666-6666-6666-6666-666666666666', 'work',       'Dev & Invest',      'kwame-work',        'Dev & RE side work',     '{"default_workspace":"re_learning"}'::jsonb,        '{"time_tracking":true}'::jsonb,    '{"billing_rate":95}'::jsonb,          'public',  'active'),
  -- Zuri
  ('c7777777-7777-7777-7777-777777777771', 'c6666666-6666-6666-6666-666666666666', 'personal',   'Personal',          'zuri-personal',     'My life',                '{"theme":"dark"}'::jsonb,                           '{"notifications":true}'::jsonb,    '{}'::jsonb,                           'private', 'active'),
  ('c7777777-7777-7777-7777-777777777772', 'c6666666-6666-6666-6666-666666666666', 'community',  'Community Lead',    'zuri-community',    'Mastermind organizer',   '{"visibility":"public"}'::jsonb,                    '{"allow_dm":true}'::jsonb,         '{"events_per_month":4}'::jsonb,       'public',  'active')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  AUTH.SKILL
-- =============================================================================

INSERT INTO auth.skill (account_id, name, category, proficiency, verified, endorsements) VALUES
  ('11111111-1111-1111-1111-111111111111', 'Rust Systems Programming',    'engineering',       'expert',        true,  12),
  ('11111111-1111-1111-1111-111111111111', 'PostgreSQL',                  'database',          'expert',        true,  8),
  ('11111111-1111-1111-1111-111111111111', 'Platform Architecture',       'engineering',       'expert',        true,  15),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Real Estate Analysis',        'real_estate',       'expert',        true,  23),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Portfolio Management',        'finance',           'expert',        true,  18),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Deal Structuring',            'real_estate',       'expert',        true,  11),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Software Engineering',        'engineering',       'expert',        false, 5),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Real Estate Investing',       'real_estate',       'beginner',      false, 0),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Community Building',          'community',         'expert',        true,  31),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Real Estate Investing',       'real_estate',       'intermediate',  false, 7)
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AUTH.CONNECTION_REGISTRY
-- =============================================================================

INSERT INTO auth.connection_registry (profile_id, connection_type, provider, account_ref, metadata, status) VALUES
  ('77777777-7777-7777-7777-777777777772', 'software',   'github',     'dominic-dev',       '{"scope":"work"}'::jsonb,        'active'),
  ('77777777-7777-7777-7777-777777777773', 'finance',    'stripe',     'acct_dominic_001',  '{"scope":"business"}'::jsonb,    'active'),
  ('77777777-7777-7777-7777-777777777774', 'community',  'slack',      'kogi-community',    '{"scope":"community"}'::jsonb,   'active'),
  ('a7777777-7777-7777-7777-777777777772', 'finance',    'stripe',     'acct_amara_001',    '{"scope":"business"}'::jsonb,    'active'),
  ('a7777777-7777-7777-7777-777777777772', 'marketplace','zillow',     'amara-re-profile',  '{"scope":"real_estate"}'::jsonb, 'active'),
  ('b7777777-7777-7777-7777-777777777771', 'software',   'github',     'kwame-dev',         '{"scope":"personal"}'::jsonb,    'active'),
  ('c7777777-7777-7777-7777-777777777772', 'community',  'facebook',   'zuri.mastermind',   '{"scope":"community"}'::jsonb,   'active'),
  ('c7777777-7777-7777-7777-777777777772', 'community',  'zoom',       'zuri-host',         '{"scope":"community"}'::jsonb,   'active')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AUTH.CONTACT_DIRECTORY
-- =============================================================================

INSERT INTO auth.contact_directory (profile_id, contact_name, email, phone, organization, relationship_type, tags, notes) VALUES
  ('77777777-7777-7777-7777-777777777772', 'Client Alpha',      'alpha@client.com',    '+1-555-0101', 'Alpha Org',        'client',    ARRAY['consulting','tech'], 'Long-term consulting client'),
  ('77777777-7777-7777-7777-777777777774', 'Community Lead',    'lead@community.org',  '+1-555-0110', 'Worker Co-op',     'partner',   ARRAY['cooperative'],      'Co-op founding partner'),
  ('a7777777-7777-7777-7777-777777777772', 'RE Broker Mike',    'mike@re.com',         '+1-555-0120', 'Metro Realty',     'advisor',   ARRAY['real_estate'],      'Primary listing broker'),
  ('a7777777-7777-7777-7777-777777777772', 'Lender First Bank', 'loans@firstbank.com', '+1-555-0130', 'First National',   'lender',    ARRAY['finance','loans'],  'Hard money + conventional'),
  ('c7777777-7777-7777-7777-777777777772', 'Mastermind Zuri',   'zuri@mastermind.io',  '+1-555-0140', 'RE Mastermind LLC','partner',   ARRAY['mastermind','re'],  'Own mastermind organization')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AUTH.ASSET_VAULT
-- =============================================================================

INSERT INTO auth.asset_vault (profile_id, asset_type, name, description, value_amount, currency) VALUES
  ('77777777-7777-7777-7777-777777777773', 'capital',    'Seed Reserve',       'Platform seed capital',    25000.00,  'USD'),
  ('77777777-7777-7777-7777-777777777772', 'software',   'IDE License',        'JetBrains All Products',   299.00,    'USD'),
  ('a7777777-7777-7777-7777-777777777772', 'real_estate','Property – Oak Ave', 'Rental property duplex',   425000.00, 'USD'),
  ('a7777777-7777-7777-7777-777777777772', 'real_estate','Property – Elm St',  'Flip in progress',         310000.00, 'USD'),
  ('a7777777-7777-7777-7777-777777777772', 'digital',    'RE Playbook IP',     'Intellectual property',     15000.00, 'USD'),
  ('b7777777-7777-7777-7777-777777777771', 'digital',    'Software License',   'Dev tools subscription',   120.00,    'USD')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AUTH.PROFILE_TOOL / PROGRAM / PROJECT
-- =============================================================================

INSERT INTO auth.profile_tool (profile_id, tool_name, tool_category, tool_url) VALUES
  ('77777777-7777-7777-7777-777777777772', 'Sprint Planner',    'planning',    'https://linear.app'),
  ('77777777-7777-7777-7777-777777777773', 'Capital Dashboard', 'finance',     'https://stripe.com'),
  ('77777777-7777-7777-7777-777777777774', 'Community Room',    'social',      'https://slack.com'),
  ('a7777777-7777-7777-7777-777777777772', 'Zillow Analytics',  'real_estate', 'https://zillow.com'),
  ('a7777777-7777-7777-7777-777777777772', 'DealCheck',         'real_estate', 'https://dealcheck.io'),
  ('b7777777-7777-7777-7777-777777777772', 'GitHub',            'developer',   'https://github.com')
ON CONFLICT DO NOTHING;

INSERT INTO auth.profile_program (profile_id, program_name, status) VALUES
  ('77777777-7777-7777-7777-777777777772', 'Kogi MVP Build',        'active'),
  ('77777777-7777-7777-7777-777777777773', 'Revenue Operations',    'active'),
  ('77777777-7777-7777-7777-777777777774', 'Mutual Aid Network',    'active'),
  ('a7777777-7777-7777-7777-777777777772', 'RE Investment Program', 'active'),
  ('b7777777-7777-7777-7777-777777777772', 'RE Learning Program',   'active'),
  ('c7777777-7777-7777-7777-777777777772', 'Mastermind Program',    'active')
ON CONFLICT DO NOTHING;

INSERT INTO auth.profile_project (profile_id, project_ref, source_module, status) VALUES
  ('77777777-7777-7777-7777-777777777772', 'kogi-mvp',             'wbs',       'active'),
  ('77777777-7777-7777-7777-777777777773', 'fundraise-seed',       'fund',      'active'),
  ('77777777-7777-7777-7777-777777777774', 'co-op-launch',         'community', 'active'),
  ('a7777777-7777-7777-7777-777777777772', 're-portfolio-build',   'portfolio', 'active'),
  ('a7777777-7777-7777-7777-777777777772', 're-playbook-v1',       'studio',    'completed'),
  ('b7777777-7777-7777-7777-777777777772', 're-learning-journey',  'portfolio', 'active'),
  ('c7777777-7777-7777-7777-777777777772', 're-mastermind-launch', 'community', 'active')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AUTH.MFA_CONFIG
-- =============================================================================

INSERT INTO auth.mfa_config (account_id, mfa_type, secret_hash, enabled) VALUES
  ('11111111-1111-1111-1111-111111111111', 'totp', 'hash-totp-owner', true),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'totp', 'hash-totp-amara', true)
ON CONFLICT (account_id) DO NOTHING;

-- =============================================================================
--  AUTH.API_KEY
-- =============================================================================

INSERT INTO auth.api_key (account_id, key_hash, label, scopes) VALUES
  ('11111111-1111-1111-1111-111111111111', 'kogi-api-owner-hash', 'Platform Admin Key', ARRAY['*']),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'kogi-api-amara-hash', 'Amara Dev Key',      ARRAY['portfolio:read','market:sell'])
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AUTH.FEDERATED_IDENTITY
-- =============================================================================

INSERT INTO auth.federated_identity (account_id, provider, subject_id, email) VALUES
  ('11111111-1111-1111-1111-111111111111', 'github', 'gh-dominic-001', 'owner@kogi.local'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'google', 'google-amara-001','amara@kogi.local')
ON CONFLICT (provider, subject_id) DO NOTHING;

-- =============================================================================
--  KERN.MODULE_REGISTRY
-- =============================================================================

INSERT INTO kern.module_registry (module_id, module_kind, language, version, status) VALUES
  ('kogi.office',        'office',        'rust',  '1.0.0', 'active'),
  ('kogi.bank',          'bank',          'rust',  '1.0.0', 'active'),
  ('kogi.exchange',      'exchange',      'go',    '1.0.0', 'active'),
  ('kogi.marketplace',   'marketplace',   'go',    '1.0.0', 'active'),
  ('kogi.community',     'community',     'go',    '1.0.0', 'active'),
  ('kogi.studio',        'studio',        'rust',  '1.0.0', 'active'),
  ('kogi.profile',       'profile',       'rust',  '1.0.0', 'active'),
  ('kogi.organizations', 'organizations', 'rust',  '1.0.0', 'active'),
  ('kogi.developer',     'developer',     'rust',  '1.0.0', 'active'),
  ('kogi.database',      'database',      'rust',  '1.0.0', 'active')
ON CONFLICT (module_id) DO NOTHING;

-- =============================================================================
--  KERN.NODE_REGISTRY
-- =============================================================================

INSERT INTO kern.node_registry (node_id, node_type, host, port, region, role, status) VALUES
  ('kogi-host-001', 'host',    'localhost', 8080, 'us-central-1', 'leader',   'active'),
  ('kogi-gw-001',   'gateway', 'localhost', 8090, 'us-central-1', 'follower', 'active'),
  ('kogi-eng-001',  'engine',  'localhost', 9090, 'us-central-1', 'follower', 'active')
ON CONFLICT (node_id) DO NOTHING;

-- =============================================================================
--  KERN.RBAC_POLICY
-- =============================================================================

INSERT INTO kern.rbac_policy (policy_name, resource_type, action, conditions, effect, priority) VALUES
  ('allow_owner_all',        'component', '*',       '{"role":"owner"}'::jsonb,       'allow', 1),
  ('allow_editor_edit',      'component', 'edit',    '{"role":"editor"}'::jsonb,      'allow', 10),
  ('allow_subscriber_read',  'component', 'read',    '{"role":"subscriber"}'::jsonb,  'allow', 20),
  ('deny_anon_write',        'component', 'write',   '{"role":"anonymous"}'::jsonb,   'deny',  1),
  ('allow_admin_all',        '*',         '*',       '{"role":"admin"}'::jsonb,       'allow', 0),
  ('allow_publisher_sell',   'listing',   'create',  '{"role":"publisher"}'::jsonb,   'allow', 15),
  ('allow_buyer_purchase',   'listing',   'buy',     '{"authenticated":true}'::jsonb, 'allow', 20)
ON CONFLICT (policy_name) DO NOTHING;

-- =============================================================================
--  KERN.SCHEDULER_JOB
-- =============================================================================

INSERT INTO kern.scheduler_job (owner_id, topic, cron_expr, payload_template, next_run_at) VALUES
  ('11111111-1111-1111-1111-111111111111', 'analytics.daily_rollup',    '0 2 * * *',   '{"scope":"platform"}'::jsonb,   now() + interval '1 day'),
  ('11111111-1111-1111-1111-111111111111', 'portfolio.snapshot.auto',   '0 0 * * 0',   '{"scope":"all"}'::jsonb,        now() + interval '7 days'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'market.subscription.renew', '0 9 1 * *',   '{"scope":"subscriptions"}'::jsonb,now() + interval '30 days')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  PORTFOLIO.WORKSPACE
-- =============================================================================

INSERT INTO portfolio.workspace (id, account_id, workspace_type, name, description) VALUES
  ('33333333-3333-3333-3333-333333333333', '11111111-1111-1111-1111-111111111111', 'personal',       'Dominic Main Workspace',       'All of Dominic''s work'),
  ('a3333333-3333-3333-3333-333333333333', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'business',       'Amara RE Investment Hub',      'Amara''s real estate operations'),
  ('b3333333-3333-3333-3333-333333333333', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'personal',       'Kwame Learning Workspace',     'Kwame''s learning & investment workspace'),
  ('c3333333-3333-3333-3333-333333333333', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'community',      'Zuri Mastermind Workspace',    'Zuri''s mastermind & community workspace')
ON CONFLICT (id) DO NOTHING;

-- Profile → Workspace bindings
INSERT INTO auth.profile_workspace_binding (profile_id, workspace_id) VALUES
  ('77777777-7777-7777-7777-777777777771', '33333333-3333-3333-3333-333333333333'),
  ('77777777-7777-7777-7777-777777777772', '33333333-3333-3333-3333-333333333333'),
  ('77777777-7777-7777-7777-777777777773', '33333333-3333-3333-3333-333333333333'),
  ('77777777-7777-7777-7777-777777777774', '33333333-3333-3333-3333-333333333333'),
  ('a7777777-7777-7777-7777-777777777771', 'a3333333-3333-3333-3333-333333333333'),
  ('a7777777-7777-7777-7777-777777777772', 'a3333333-3333-3333-3333-333333333333'),
  ('b7777777-7777-7777-7777-777777777771', 'b3333333-3333-3333-3333-333333333333'),
  ('b7777777-7777-7777-7777-777777777772', 'b3333333-3333-3333-3333-333333333333'),
  ('c7777777-7777-7777-7777-777777777771', 'c3333333-3333-3333-3333-333333333333'),
  ('c7777777-7777-7777-7777-777777777772', 'c3333333-3333-3333-3333-333333333333')
ON CONFLICT (profile_id, workspace_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.PORTFOLIO
-- =============================================================================

INSERT INTO portfolio.portfolio (id, workspace_id, owner_id, name, description, domain, mission, visibility, status, tags) VALUES
  -- Dominic
  ('55555555-5555-5555-5555-555555555555', '33333333-3333-3333-3333-333333333333', '11111111-1111-1111-1111-111111111111',
   'Kogi Root Portfolio',          'Platform operations & dev portfolio',           'operations',   'Build the Kogi platform',              'private',  'active', ARRAY['kogi','platform','dev']),
  -- Amara
  ('a5555555-5555-5555-5555-555555555555', 'a3333333-3333-3333-3333-333333333333', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   'Amara Real Estate Portfolio',  'All RE investments, assets & resources',        'real_estate',  'Build wealth through real estate',     'public',   'active', ARRAY['real_estate','investing','wealth']),
  ('a5555555-5555-5555-5555-555555555556', 'a3333333-3333-3333-3333-333333333333', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   'Educator Portfolio',           'Playbooks, guides & courses created by Amara',  'education',    'Democratize RE knowledge',             'public',   'active', ARRAY['education','playbook','creator']),
  -- Kwame
  ('b5555555-5555-5555-5555-555555555555', 'b3333333-3333-3333-3333-333333333333', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   'Kwame RE Learning Portfolio',  'My RE journey: learning → assets',              'real_estate',  'Invest in my first 5 properties',      'private',  'active', ARRAY['real_estate','learning','investing']),
  ('b5555555-5555-5555-5555-555555555556', 'b3333333-3333-3333-3333-333333333333', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   'RE Project Management SaaS',   'Platform for managing RE asset portfolios',     'software',     'Ship a RE PM SaaS for other investors', 'private',  'active', ARRAY['saas','real_estate','software']),
  -- Zuri
  ('c5555555-5555-5555-5555-555555555555', 'c3333333-3333-3333-3333-333333333333', 'cccccccc-cccc-cccc-cccc-cccccccccccc',
   'Mastermind Portfolio',         'Community-driven masterminds & events',         'community',    'Uplift others through shared knowledge','public',   'active', ARRAY['mastermind','community','education','real_estate'])
ON CONFLICT (id) DO NOTHING;

-- Profile → Portfolio bindings
INSERT INTO auth.profile_portfolio_binding (profile_id, portfolio_id) VALUES
  ('77777777-7777-7777-7777-777777777771', '55555555-5555-5555-5555-555555555555'),
  ('77777777-7777-7777-7777-777777777772', '55555555-5555-5555-5555-555555555555'),
  ('77777777-7777-7777-7777-777777777773', '55555555-5555-5555-5555-555555555555'),
  ('77777777-7777-7777-7777-777777777774', '55555555-5555-5555-5555-555555555555'),
  ('a7777777-7777-7777-7777-777777777771', 'a5555555-5555-5555-5555-555555555555'),
  ('a7777777-7777-7777-7777-777777777772', 'a5555555-5555-5555-5555-555555555555'),
  ('a7777777-7777-7777-7777-777777777772', 'a5555555-5555-5555-5555-555555555556'),
  ('b7777777-7777-7777-7777-777777777771', 'b5555555-5555-5555-5555-555555555555'),
  ('b7777777-7777-7777-7777-777777777772', 'b5555555-5555-5555-5555-555555555555'),
  ('b7777777-7777-7777-7777-777777777772', 'b5555555-5555-5555-5555-555555555556'),
  ('c7777777-7777-7777-7777-777777777771', 'c5555555-5555-5555-5555-555555555555'),
  ('c7777777-7777-7777-7777-777777777772', 'c5555555-5555-5555-5555-555555555555')
ON CONFLICT (profile_id, portfolio_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.COMPONENT  (Items & Containers in the portfolio system)
-- =============================================================================

INSERT INTO portfolio.component (id, portfolio_id, component_kind, category, book_kind, name, description, status, state, visibility, version, tags, hashtags, topics, payload) VALUES
  -- ── Amara's RE Portfolio Items ────────────────────────────────────────────
  ('a0000001-0000-0000-0000-000000000001', 'a5555555-5555-5555-5555-555555555555', 'item', 'program',  NULL,
   'RE Investment Program', 'Master program organizing all RE investment projects', 'active', 'running', 'public', '1.0.0',
   ARRAY['real_estate','investment'], ARRAY['#realestate','#investing'], ARRAY['real_estate','wealth_building'],
   '{"objective":"Build a 10-property portfolio by 2028","budget":500000,"currency":"USD"}'::jsonb),

  ('a0000001-0000-0000-0000-000000000002', 'a5555555-5555-5555-5555-555555555555', 'item', 'project', NULL,
   'Oak Ave Duplex Acquisition', 'Acquired rental duplex – full renovation & tenant placement', 'completed', 'sealed', 'private', '1.0.0',
   ARRAY['real_estate','rental','duplex'], ARRAY['#rental','#cashflow'], ARRAY['real_estate'],
   '{"project_type":"Investment","start_date":"2024-01-15","end_date":"2024-08-30","methodology":"agile"}'::jsonb),

  ('a0000001-0000-0000-0000-000000000003', 'a5555555-5555-5555-5555-555555555555', 'item', 'project', NULL,
   'Elm St Flip', 'House flip in progress – purchase→rehab→sell', 'active', 'running', 'private', '1.0.0',
   ARRAY['real_estate','flip'], ARRAY['#flip','#realestate'], ARRAY['real_estate'],
   '{"project_type":"Investment","methodology":"waterfall","start_date":"2025-11-01"}'::jsonb),

  ('a0000001-0000-0000-0000-000000000004', 'a5555555-5555-5555-5555-555555555555', 'item', 'asset', NULL,
   'Oak Ave Duplex Asset', 'Property asset — Oak Ave, duplex, currently rented', 'active', 'running', 'private', '1.0.0',
   ARRAY['real_estate','asset','rental'], ARRAY['#asset'], ARRAY['real_estate'],
   '{"asset_type":"Physical","valuation":425000,"currency":"USD","acquired_at":"2024-08-30"}'::jsonb),

  ('a0000001-0000-0000-0000-000000000005', 'a5555555-5555-5555-5555-555555555555', 'item', 'resource', NULL,
   'Hard Money Lender Access', 'Network of hard money lenders for fast acquisitions', 'active', 'configured', 'private', '1.0.0',
   ARRAY['real_estate','finance','lender'], ARRAY['#financing'], ARRAY['real_estate'],
   '{"resource_type":"Service","skills":["deal_financing","due_diligence"],"availability":1.0}'::jsonb),

  -- ── Amara's Educator Portfolio Items (Playbook etc.) ─────────────────────
  ('a0000001-0000-0000-0000-000000000010', 'a5555555-5555-5555-5555-555555555556', 'container', 'book', 'playbook',
   'Real Estate Investment Playbook v1', 'Step-by-step RE investing guide from deal sourcing to exit', 'active', 'running', 'public', '1.0.0',
   ARRAY['real_estate','playbook','investing','education'], ARRAY['#realestate','#playbook','#investing'], ARRAY['real_estate','education'],
   '{"description":"Complete RE investing guide","plays":["Deal Sourcing","Due Diligence","Financing","Rehab","Exit"]}'::jsonb),

  ('a0000001-0000-0000-0000-000000000011', 'a5555555-5555-5555-5555-555555555556', 'container', 'book', 'playbook',
   'Real Estate Investment Playbook v2', 'Updated playbook with mastermind insights & advanced strategies', 'active', 'running', 'public', '2.0.0',
   ARRAY['real_estate','playbook','investing','education','advanced'], ARRAY['#realestate','#playbook'], ARRAY['real_estate','education'],
   '{"description":"Advanced RE investing guide with mastermind insights","plays":["Deal Sourcing","Advanced Due Diligence","Creative Financing","Value-Add Strategy","Portfolio Exit Planning"]}'::jsonb),

  -- ── Kwame's RE Learning Items ─────────────────────────────────────────────
  ('b0000001-0000-0000-0000-000000000001', 'b5555555-5555-5555-5555-555555555555', 'item', 'project', NULL,
   'RE Learning Journey', 'My structured path to first investment property', 'active', 'running', 'private', '1.0.0',
   ARRAY['real_estate','learning'], ARRAY['#realestate','#learning'], ARRAY['real_estate'],
   '{"project_type":"Research","methodology":"agile","start_date":"2025-10-01"}'::jsonb),

  ('b0000001-0000-0000-0000-000000000002', 'b5555555-5555-5555-5555-555555555555', 'item', 'artifact', NULL,
   'Purchased: RE Playbook v1', 'Amara''s playbook purchased by Kwame — unlocked content', 'active', 'configured', 'private', '1.0.0',
   ARRAY['real_estate','playbook','purchased'], ARRAY['#playbook'], ARRAY['real_estate'],
   '{"artifact_type":"playbook","source_component_id":"a0000001-0000-0000-0000-000000000010"}'::jsonb),

  ('b0000001-0000-0000-0000-000000000003', 'b5555555-5555-5555-5555-555555555555', 'item', 'artifact', NULL,
   'Purchased: RE Playbook v2', 'Updated playbook — discount applied via subscriber loyalty', 'active', 'configured', 'private', '2.0.0',
   ARRAY['real_estate','playbook','purchased'], ARRAY['#playbook'], ARRAY['real_estate'],
   '{"artifact_type":"playbook","source_component_id":"a0000001-0000-0000-0000-000000000011","discount_applied":0.20}'::jsonb),

  ('b0000001-0000-0000-0000-000000000010', 'b5555555-5555-5555-5555-555555555556', 'item', 'project', NULL,
   'RE PM SaaS Build', 'Building a RE asset portfolio management SaaS platform', 'active', 'running', 'private', '0.1.0',
   ARRAY['saas','real_estate','software','startup'], ARRAY['#saas','#buildinpublic'], ARRAY['software','real_estate'],
   '{"project_type":"Software","methodology":"agile","start_date":"2026-01-01"}'::jsonb),

  -- ── Zuri's Mastermind Items ──────────────────────────────────────────────
  ('c0000001-0000-0000-0000-000000000001', 'c5555555-5555-5555-5555-555555555555', 'item', 'project', NULL,
   'RE Mastermind Q1-2026', 'Paid real estate investment mastermind cohort – Q1 2026', 'active', 'running', 'public', '1.0.0',
   ARRAY['real_estate','mastermind','community','education'], ARRAY['#mastermind','#realestate'], ARRAY['real_estate','community'],
   '{"project_type":"Organizational","methodology":"custom","start_date":"2026-01-15","end_date":"2026-04-15"}'::jsonb),

  ('c0000001-0000-0000-0000-000000000002', 'c5555555-5555-5555-5555-555555555555', 'container', 'book', 'guidebook',
   'Mastermind Participant Guide', 'Onboarding & participation guide for mastermind members', 'active', 'running', 'protected', '1.0.0',
   ARRAY['mastermind','guide','education'], ARRAY['#mastermind'], ARRAY['education','community'],
   '{"sections":["Welcome","Norms","Weekly Structure","Resources","Accountability"],"version":"1.0.0","audience":["investors","entrepreneurs"]}'::jsonb),

  -- ── Dominic's Kogi Platform Items ─────────────────────────────────────────
  ('d0000001-0000-0000-0000-000000000001', '55555555-5555-5555-5555-555555555555', 'item', 'project', NULL,
   'Kogi MVP Development', 'Full MVP build of the Kogi Independent Worker OS', 'active', 'running', 'private', '0.8.0',
   ARRAY['kogi','platform','development','mvp'], ARRAY['#kogi','#buildinpublic'], ARRAY['software','platform'],
   '{"project_type":"Software","methodology":"agile","start_date":"2025-01-01"}'::jsonb),

  ('d0000001-0000-0000-0000-000000000002', '55555555-5555-5555-5555-555555555555', 'item', 'program', NULL,
   'Kogi Platform Program', 'Umbrella program for all Kogi development projects', 'active', 'running', 'private', '1.0.0',
   ARRAY['kogi','platform'], ARRAY['#kogi'], ARRAY['software'],
   '{"objective":"Launch Kogi v1 by Q4 2026","budget":150000,"currency":"USD"}'::jsonb)

ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.COMPONENT_OWNER
-- =============================================================================

INSERT INTO portfolio.component_owner (component_id, user_id, is_primary) VALUES
  ('a0000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
  ('a0000001-0000-0000-0000-000000000002', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
  ('a0000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
  ('a0000001-0000-0000-0000-000000000004', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
  ('a0000001-0000-0000-0000-000000000010', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
  ('a0000001-0000-0000-0000-000000000011', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
  ('b0000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', true),
  ('b0000001-0000-0000-0000-000000000002', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', true),
  ('b0000001-0000-0000-0000-000000000003', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', true),
  ('b0000001-0000-0000-0000-000000000010', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', true),
  ('c0000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', true),
  ('c0000001-0000-0000-0000-000000000002', 'cccccccc-cccc-cccc-cccc-cccccccccccc', true),
  ('d0000001-0000-0000-0000-000000000001', '11111111-1111-1111-1111-111111111111', true),
  ('d0000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', true)
ON CONFLICT (component_id, user_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.COMPONENT_USER  (subscriptions, followers, contributors)
-- =============================================================================

INSERT INTO portfolio.component_user (component_id, user_id, permission_tier, role_bucket) VALUES
  -- Kwame subscribed to Amara's playbooks
  ('a0000001-0000-0000-0000-000000000010', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'subscriber', 'subscriber'),
  ('a0000001-0000-0000-0000-000000000011', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'subscriber', 'subscriber'),
  -- Kwame watches Amara's RE program
  ('a0000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'subscriber', 'watcher'),
  -- Amara joined/participated in Zuri's mastermind
  ('c0000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'contributor', 'follower'),
  -- Kwame follows Zuri's mastermind
  ('c0000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'subscriber', 'subscriber'),
  -- Dominic watches platform development
  ('d0000001-0000-0000-0000-000000000001', '11111111-1111-1111-1111-111111111111', 'owner', 'owner')
ON CONFLICT (component_id, user_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.COMPONENT_ANALYTICS
-- =============================================================================

INSERT INTO portfolio.component_analytics (component_id, clicks, impressions, likes, saves, followers, subscribers, watchers, engagement_rate, benchmark_score) VALUES
  ('a0000001-0000-0000-0000-000000000010', 3420, 8900, 215, 182, 320, 145, 88, 0.0182, 72.4),
  ('a0000001-0000-0000-0000-000000000011', 1200, 2900, 98,  74,  180, 112, 43, 0.0220, 68.1),
  ('a0000001-0000-0000-0000-000000000001', 540,  1200, 33,  28,  95,  45,  22, 0.0140, 58.0),
  ('c0000001-0000-0000-0000-000000000001', 890,  2200, 67,  55,  140, 88,  30, 0.0150, 63.5),
  ('d0000001-0000-0000-0000-000000000001', 120,  300,  8,   5,   14,  6,   3,  0.0100, 41.0)
ON CONFLICT (component_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.GRAPH_EDGE
-- =============================================================================

INSERT INTO portfolio.graph_edge (id, from_id, to_id, kind, label) VALUES
  -- Program contains projects
  ('edge-000001', 'a0000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000002', 'contains', 'program→project'),
  ('edge-000002', 'a0000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000003', 'contains', 'program→project'),
  -- Project → Asset hierarchy
  ('edge-000003', 'a0000001-0000-0000-0000-000000000002', 'a0000001-0000-0000-0000-000000000004', 'hierarchy', 'acquisition→asset'),
  -- Project depends on resource
  ('edge-000004', 'a0000001-0000-0000-0000-000000000003', 'a0000001-0000-0000-0000-000000000005', 'dependency', 'flip needs financing'),
  -- Kwame's project links to purchased playbook artifact
  ('edge-000005', 'b0000001-0000-0000-0000-000000000001', 'b0000001-0000-0000-0000-000000000002', 'contains', 'project→artifact'),
  ('edge-000006', 'b0000001-0000-0000-0000-000000000001', 'b0000001-0000-0000-0000-000000000003', 'contains', 'project→artifact'),
  -- Kwame's SaaS project links to RE learning
  ('edge-000007', 'b0000001-0000-0000-0000-000000000010', 'b0000001-0000-0000-0000-000000000001', 'link', 'saas inspired by learning'),
  -- Mastermind links to Amara's playbook (source/federation)
  ('edge-000008', 'c0000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000010', 'federation', 'mastermind references playbook')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.COMPONENT_ACTION_LOG
-- =============================================================================

INSERT INTO portfolio.component_action_log (component_id, actor_id, action_kind, action_params, description) VALUES
  ('a0000001-0000-0000-0000-000000000010', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Post',       '{"visibility":"Public"}'::jsonb,    'Published playbook v1 to marketplace'),
  ('a0000001-0000-0000-0000-000000000010', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Subscribe',  '{}'::jsonb,                         'Kwame subscribed to playbook'),
  ('a0000001-0000-0000-0000-000000000010', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Like',       '{}'::jsonb,                         'Kwame liked the playbook'),
  ('a0000001-0000-0000-0000-000000000011', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Post',       '{"visibility":"Public"}'::jsonb,    'Published playbook v2 after mastermind'),
  ('c0000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Join',       '{}'::jsonb,                         'Amara joined mastermind'),
  ('c0000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Invest',     '{"amount":500}'::jsonb,             'Amara invested in mastermind offering')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  PORTFOLIO.MILESTONE / RISK / CHARTER / METRIC
-- =============================================================================

INSERT INTO portfolio.milestone (component_id, name, description, due_date, status) VALUES
  ('a0000001-0000-0000-0000-000000000003', 'Purchase Closed',    'Close on the property',    '2025-12-15 00:00:00+00', 'completed'),
  ('a0000001-0000-0000-0000-000000000003', 'Rehab Complete',     'Finish all renovations',   '2026-03-01 00:00:00+00', 'active'),
  ('a0000001-0000-0000-0000-000000000003', 'Listed for Sale',    'List on MLS',              '2026-04-01 00:00:00+00', 'draft'),
  ('b0000001-0000-0000-0000-000000000010', 'MVP Launched',       'Ship RE PM SaaS MVP',      '2026-09-01 00:00:00+00', 'draft'),
  ('d0000001-0000-0000-0000-000000000001', 'Kogi Alpha Launch',  'Internal alpha release',   '2026-06-01 00:00:00+00', 'active'),
  ('d0000001-0000-0000-0000-000000000001', 'Kogi Beta Launch',   'Public beta release',      '2026-09-01 00:00:00+00', 'draft')
ON CONFLICT DO NOTHING;

INSERT INTO portfolio.risk (component_id, description, severity, probability, mitigation, status) VALUES
  ('a0000001-0000-0000-0000-000000000003', 'Rehab cost overrun',          'high',   0.40, '10% contingency budget',              'open'),
  ('a0000001-0000-0000-0000-000000000003', 'Market softens before sale',  'medium', 0.25, 'Hold as rental if needed',            'open'),
  ('d0000001-0000-0000-0000-000000000001', 'Key hire delays ship date',   'high',   0.35, 'Prioritize contractor network',       'open'),
  ('b0000001-0000-0000-0000-000000000010', 'No early user validation',    'medium', 0.50, 'Launch landing page + waitlist early','open')
ON CONFLICT DO NOTHING;

INSERT INTO portfolio.charter (component_id, executive_summary, objectives, scope, success_criteria, version) VALUES
  ('a0000001-0000-0000-0000-000000000001',
   'Build a portfolio of cashflowing real estate assets generating $10k/month net income by 2028.',
   ARRAY['Acquire 10 properties', 'Achieve 8% avg cap rate', 'Generate $10k/mo net NOI'],
   'US residential and small multifamily real estate',
   ARRAY['10 properties acquired', '$10k/mo net income', 'Portfolio LTV < 70%'],
   '1.0.0'),
  ('d0000001-0000-0000-0000-000000000002',
   'Build and launch the Kogi Independent Worker OS — v1 — enabling 10k+ independent workers.',
   ARRAY['Complete MVP by Q4 2026', 'Onboard 10k beta users', 'Achieve $50k MRR'],
   'Web + desktop + mobile platform with all 10 core modules',
   ARRAY['All modules live', 'Performance SLAs met', '$50k MRR'],
   '1.0.0')
ON CONFLICT (component_id) DO NOTHING;

INSERT INTO portfolio.metric (component_id, name, metric_type, value, unit) VALUES
  ('a0000001-0000-0000-0000-000000000001', 'Portfolio Cap Rate',    'kpi',     7.2,    '%'),
  ('a0000001-0000-0000-0000-000000000001', 'Monthly NOI',           'gauge',   3200,   'USD'),
  ('a0000001-0000-0000-0000-000000000001', 'Properties Owned',      'counter', 2,      'count'),
  ('a0000001-0000-0000-0000-000000000010', 'Playbook Sales',        'counter', 87,     'count'),
  ('a0000001-0000-0000-0000-000000000010', 'Avg Rating',            'gauge',   4.8,    'stars'),
  ('d0000001-0000-0000-0000-000000000002', 'Modules Completed',     'counter', 7,      'count'),
  ('d0000001-0000-0000-0000-000000000002', 'Beta Users Waitlisted', 'gauge',   342,    'users')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  PORTFOLIO.VERSION_ENTRY
-- =============================================================================

INSERT INTO portfolio.version_entry (component_id, version, author_id, message, tags) VALUES
  ('a0000001-0000-0000-0000-000000000010', '1.0.0', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Initial publication of RE Playbook', ARRAY['release']),
  ('a0000001-0000-0000-0000-000000000011', '2.0.0', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Updated with mastermind insights and advanced strategies', ARRAY['release','major']),
  ('d0000001-0000-0000-0000-000000000001', '0.8.0', '11111111-1111-1111-1111-111111111111', 'Pre-alpha: all modules scaffolded', ARRAY['milestone'])
ON CONFLICT DO NOTHING;

-- =============================================================================
--  PORTFOLIO.SCHEDULE / SCHEDULE_ENTRY
-- =============================================================================

INSERT INTO portfolio.schedule (id, portfolio_id, name, description, owner_id) VALUES
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'a5555555-5555-5555-5555-555555555555', 'RE Acquisition Pipeline 2026', 'Scheduled property targets', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
  ('a0eebc95-0000-0000-0000-13135d5153f1', 'c5555555-5555-5555-5555-555555555555', 'Mastermind Sessions Q1-2026',  'Weekly mastermind calls',   'cccccccc-cccc-cccc-cccc-cccccccccccc')
ON CONFLICT (id) DO NOTHING;

INSERT INTO portfolio.schedule_entry (schedule_id, component_id, scheduled_at, duration_minutes, notes, position) VALUES
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'a0000001-0000-0000-0000-000000000003', '2026-04-01 00:00:00+00', NULL, 'Target sell by April',         1),
  ('a0eebc95-0000-0000-0000-13135d5153f1', 'c0000001-0000-0000-0000-000000000001', '2026-01-22 18:00:00+00', 90,   'Week 1: Deal sourcing',         1),
  ('a0eebc95-0000-0000-0000-13135d5153f1', 'c0000001-0000-0000-0000-000000000001', '2026-01-29 18:00:00+00', 90,   'Week 2: Due diligence deep dive',2)
ON CONFLICT DO NOTHING;

-- =============================================================================
--  PORTFOLIO.GROUP / COLLECTION / LIST
-- =============================================================================

INSERT INTO portfolio.group (id, portfolio_id, name, description, group_type, owner_id) VALUES
  ('ca000001-0000-0000-0000-000000000001', 'a5555555-5555-5555-5555-555555555555', 'Active Projects', 'All in-flight RE projects',           'projects', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
  ('cb000001-0000-0000-0000-000000000001', 'b5555555-5555-5555-5555-555555555555', 'RE Resources',    'Playbooks & guides Kwame is using',   'resources','bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT (id) DO NOTHING;

INSERT INTO portfolio.group_member (group_id, component_id) VALUES
  ('ca000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000002'),
  ('ca000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000003'),
  ('cb000001-0000-0000-0000-000000000001', 'b0000001-0000-0000-0000-000000000002'),
  ('cb000001-0000-0000-0000-000000000001', 'b0000001-0000-0000-0000-000000000003')
ON CONFLICT (group_id, component_id) DO NOTHING;

INSERT INTO portfolio.collection (id, portfolio_id, name, description, tags, owner_id) VALUES
  ('ca000001-0000-0000-0000-000000000001', 'a5555555-5555-5555-5555-555555555556', 'Published Playbooks', 'All public playbooks Amara has published', ARRAY['playbook','education'], 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa')
ON CONFLICT (id) DO NOTHING;

INSERT INTO portfolio.collection_item (collection_id, component_id) VALUES
  ('ca000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000010'),
  ('ca000001-0000-0000-0000-000000000001', 'a0000001-0000-0000-0000-000000000011')
ON CONFLICT (collection_id, component_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.SNAPSHOT / CHECKPOINT
-- =============================================================================

INSERT INTO portfolio.snapshot (portfolio_id, snapshot_id, label, component_count, edge_count, event_count, created_by) VALUES
  ('a5555555-5555-5555-5555-555555555555', 'snap-000001', 'Initial RE Portfolio Snapshot', 5,  4, 6,  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
  ('b5555555-5555-5555-5555-555555555555', 'snap-000002', 'Kwame Q1-2026 State',           3,  2, 3,  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
  ('55555555-5555-5555-5555-555555555555', 'snap-000003', 'Platform Alpha State',          2,  0, 1,  '11111111-1111-1111-1111-111111111111')
ON CONFLICT (snapshot_id) DO NOTHING;

INSERT INTO portfolio.checkpoint (portfolio_id, checkpoint_id, label, snapshot_id, note, created_by) VALUES
  ('a5555555-5555-5555-5555-555555555555', 'ckpt-000001', 'Q4 2025 Review', 'snap-000001', 'Year-end portfolio review checkpoint', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
  ('55555555-5555-5555-5555-555555555555', 'ckpt-000002', 'Pre-Alpha Gate',  'snap-000003', 'Gate review before alpha user testing', '11111111-1111-1111-1111-111111111111')
ON CONFLICT (checkpoint_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.FEDERATION
-- =============================================================================

INSERT INTO portfolio.federation (id, name, description, created_by) VALUES
  ('fed00001-0000-0000-0000-000000000001', 'RE Investors Federation', 'Cross-portfolio federation for RE investors on Kogi', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa')
ON CONFLICT (id) DO NOTHING;

INSERT INTO portfolio.federation_member (federation_id, portfolio_id, peer_node_id) VALUES
  ('fed00001-0000-0000-0000-000000000001', 'a5555555-5555-5555-5555-555555555555', 'kogi-host-001'),
  ('fed00001-0000-0000-0000-000000000001', 'b5555555-5555-5555-5555-555555555555', 'kogi-host-001'),
  ('fed00001-0000-0000-0000-000000000001', 'c5555555-5555-5555-5555-555555555555', 'kogi-host-001')
ON CONFLICT (federation_id, portfolio_id) DO NOTHING;

-- =============================================================================
--  PORTFOLIO.PORTFOLIO_COLLABORATOR
-- =============================================================================

INSERT INTO portfolio.portfolio_collaborator (portfolio_id, account_id, role, permissions) VALUES
  ('a5555555-5555-5555-5555-555555555556', '11111111-1111-1111-1111-111111111111', 'admin', '["*"]'::jsonb)
ON CONFLICT (portfolio_id, account_id) DO NOTHING;

-- =============================================================================
--  WBS
-- =============================================================================

INSERT INTO wbs.wbs (id, portfolio_id, component_id, title, methodology, status) VALUES
  ('d0000001-0000-0000-0000-000000000001', '55555555-5555-5555-5555-555555555555', 'd0000001-0000-0000-0000-000000000001', 'Kogi MVP Agile Board',    'agile', 'active'),
  ('d0000001-0000-0000-0000-000000000002', 'b5555555-5555-5555-5555-555555555556', 'b0000001-0000-0000-0000-000000000010', 'RE PM SaaS Sprint Board', 'scrum', 'active')
ON CONFLICT (id) DO NOTHING;

INSERT INTO wbs.sprint (id, wbs_id, name, goal, status, starts_at, ends_at) VALUES
  ('50000001-0000-0000-0000-000000000001', 'd0000001-0000-0000-0000-000000000001', 'Sprint 12 — DB & Schema',   'Complete schema and seed data',      'active',  '2026-03-01 00:00:00+00', '2026-03-14 00:00:00+00'),
  ('50000001-0000-0000-0000-000000000002', 'd0000001-0000-0000-0000-000000000001', 'Sprint 13 — API Layer',     'Build REST + gRPC endpoints',        'planned', '2026-03-15 00:00:00+00', '2026-03-28 00:00:00+00'),
  ('50000001-0000-0000-0000-000000000003', 'd0000001-0000-0000-0000-000000000002', 'Sprint 1 — MVP Discovery',  'User research + feature mapping',    'active',  '2026-03-01 00:00:00+00', '2026-03-14 00:00:00+00')
ON CONFLICT (id) DO NOTHING;

INSERT INTO wbs.story (id, wbs_id, story_type, title, priority, status, estimate, sprint_id, assignee_id) VALUES
  (gen_random_uuid(), 'd0000001-0000-0000-0000-000000000001', 'feature',   'Full schema.sql build',        'critical', 'in_progress', 8,   '50000001-0000-0000-0000-000000000001', '11111111-1111-1111-1111-111111111111'),
  (gen_random_uuid(), 'd0000001-0000-0000-0000-000000000001', 'feature',   'Full seed.sql build',          'critical', 'in_progress', 5,   '50000001-0000-0000-0000-000000000001', '11111111-1111-1111-1111-111111111111'),
  (gen_random_uuid(), 'd0000001-0000-0000-0000-000000000001', 'feature',   'Portfolio gRPC service',       'high',     'backlog',     13,  '50000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111'),
  (gen_random_uuid(), 'd0000001-0000-0000-0000-000000000001', 'feature',   'Provider affiliate system',    'high',     'backlog',     8,   '50000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111'),
  (gen_random_uuid(), 'd0000001-0000-0000-0000-000000000002', 'feature',   'User interviews (10)',         'critical', 'in_progress', 5,   '50000001-0000-0000-0000-000000000003', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
  (gen_random_uuid(), 'd0000001-0000-0000-0000-000000000002', 'feature',   'Landing page + waitlist',     'high',     'todo',        3,   '50000001-0000-0000-0000-000000000003', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  MARKET.LISTING
-- =============================================================================

INSERT INTO market.listing (id, seller_id, component_id, listing_type, title, description, price, currency, tags, status, visibility) VALUES
  -- Amara's playbook v1
  ('b1000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000010',
   'playbook', 'Real Estate Investment Playbook v1', 'Complete step-by-step RE investing guide for beginners', 79.00, 'USD',
   ARRAY['real_estate','playbook','investing'], 'active', 'public'),
  -- Amara's playbook v2
  ('b1000001-0000-0000-0000-000000000002', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000011',
   'playbook', 'Real Estate Investment Playbook v2', 'Advanced RE guide with mastermind insights — includes creative financing', 129.00, 'USD',
   ARRAY['real_estate','playbook','advanced','investing'], 'active', 'public'),
  -- Zuri's mastermind
  ('b1000001-0000-0000-0000-000000000003', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'c0000001-0000-0000-0000-000000000001',
   'service', 'RE Mastermind Q1-2026 — 12-Week Program', '12-week accountability + education mastermind for RE investors', 500.00, 'USD',
   ARRAY['real_estate','mastermind','community','education'], 'active', 'public'),
  -- Kwame's RE PM SaaS (coming soon)
  ('b1000001-0000-0000-0000-000000000004', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b0000001-0000-0000-0000-000000000010',
   'service', 'RE Portfolio Manager SaaS — Early Access', 'Get early access to the RE portfolio management platform', 49.00, 'USD',
   ARRAY['saas','real_estate','software'], 'draft', 'public')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  MARKET.MARKET_ORDER  (purchases)
-- =============================================================================

INSERT INTO market.market_order (id, listing_id, buyer_id, seller_id, amount, currency, status) VALUES
  -- Kwame buys Amara's playbook v1
  ('b0000001-0000-0000-0000-000000000001', 'b1000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 79.00,  'USD', 'completed'),
  -- Kwame buys playbook v2 at 20% loyal-subscriber discount
  ('b0000001-0000-0000-0000-000000000002', 'b1000001-0000-0000-0000-000000000002', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 103.20, 'USD', 'completed'),
  -- Amara buys into Zuri's mastermind
  ('b0000001-0000-0000-0000-000000000003', 'b1000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 500.00, 'USD', 'completed'),
  -- Kwame buys mastermind spot
  ('b0000001-0000-0000-0000-000000000004', 'b1000001-0000-0000-0000-000000000003', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 500.00, 'USD', 'completed')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  MARKET.REVIEW
-- =============================================================================

INSERT INTO market.review (order_id, reviewer_id, rating, comment) VALUES
  ('b0000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 5, 'Incredibly detailed and actionable. This playbook gave me the exact framework I needed to start.'),
  ('b0000001-0000-0000-0000-000000000002', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 5, 'Even better than v1 — the creative financing section alone is worth the price.'),
  ('b0000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 5, 'Zuri runs a tight ship. Best mastermind I''ve been part of — updated my playbook because of this.')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  MARKET.SUBSCRIPTION
-- =============================================================================

INSERT INTO market.subscription (subscriber_id, publisher_id, component_id, tier, price, currency, status, renews_at) VALUES
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000001', 'paid', 9.99, 'USD', 'active', now() + interval '30 days'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000010', 'paid', 9.99, 'USD', 'active', now() + interval '30 days')
ON CONFLICT (subscriber_id, publisher_id, component_id) DO NOTHING;

-- =============================================================================
--  EXCHANGE.WALLET
-- =============================================================================

INSERT INTO exchange.wallet (id, account_id, wallet_type, currency, available, reserved) VALUES
  ('44444444-4444-4444-4444-444444444441', '11111111-1111-1111-1111-111111111111', 'operations',   'USD', 12500.00, 1000.00),
  ('44444444-4444-4444-4444-444444444442', '11111111-1111-1111-1111-111111111111', 'personal',     'USD', 3200.00,  0.00),
  ('a4444444-4444-4444-4444-444444444441', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'operations',   'USD', 24800.00, 2000.00),
  ('a4444444-4444-4444-4444-444444444442', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'marketplace',  'USD', 6890.00,  0.00),
  ('b4444444-4444-4444-4444-444444444441', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'personal',     'USD', 4200.00,  682.20),
  ('b4444444-4444-4444-4444-444444444442', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'marketplace',  'USD', 0.00,     0.00),
  ('c4444444-4444-4444-4444-444444444441', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'operations',   'USD', 8500.00,  0.00),
  ('c4444444-4444-4444-4444-444444444442', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'marketplace',  'USD', 1000.00,  0.00)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  EXCHANGE.LEDGER_ENTRY
-- =============================================================================

INSERT INTO exchange.ledger_entry (wallet_id, entry_kind, amount, currency, reference_type, reference_id, balance_after) VALUES
  -- Kwame pays for playbook v1
  ('b4444444-4444-4444-4444-444444444441', 'debit',  79.00,  'USD', 'market_order', 'b0000001-0000-0000-0000-000000000001', 4121.00),
  -- Amara receives payment
  ('a4444444-4444-4444-4444-444444444442', 'credit', 79.00,  'USD', 'market_order', 'b0000001-0000-0000-0000-000000000001', 79.00),
  -- Kwame pays for playbook v2 (discounted)
  ('b4444444-4444-4444-4444-444444444441', 'debit',  103.20, 'USD', 'market_order', 'b0000001-0000-0000-0000-000000000002', 4017.80),
  ('a4444444-4444-4444-4444-444444444442', 'credit', 103.20, 'USD', 'market_order', 'b0000001-0000-0000-0000-000000000002', 182.20),
  -- Amara pays for mastermind
  ('a4444444-4444-4444-4444-444444444441', 'debit',  500.00, 'USD', 'market_order', 'b0000001-0000-0000-0000-000000000003', 24300.00),
  ('c4444444-4444-4444-4444-444444444441', 'credit', 500.00, 'USD', 'market_order', 'b0000001-0000-0000-0000-000000000003', 9000.00),
  -- Kwame pays for mastermind
  ('b4444444-4444-4444-4444-444444444441', 'debit',  500.00, 'USD', 'market_order', 'b0000001-0000-0000-0000-000000000004', 3517.80),
  ('c4444444-4444-4444-4444-444444444441', 'credit', 500.00, 'USD', 'market_order', 'b0000001-0000-0000-0000-000000000004', 9500.00)
ON CONFLICT DO NOTHING;

-- =============================================================================
--  EXCHANGE.ESCROW
-- =============================================================================

INSERT INTO exchange.escrow (order_id, payer_wallet_id, payee_wallet_id, amount, currency, status) VALUES
  ('b0000001-0000-0000-0000-000000000001', 'b4444444-4444-4444-4444-444444444441', 'a4444444-4444-4444-4444-444444444442', 79.00,  'USD', 'released'),
  ('b0000001-0000-0000-0000-000000000002', 'b4444444-4444-4444-4444-444444444441', 'a4444444-4444-4444-4444-444444444442', 103.20, 'USD', 'released'),
  ('b0000001-0000-0000-0000-000000000003', 'a4444444-4444-4444-4444-444444444441', 'c4444444-4444-4444-4444-444444444441', 500.00, 'USD', 'released'),
  ('b0000001-0000-0000-0000-000000000004', 'b4444444-4444-4444-4444-444444444441', 'c4444444-4444-4444-4444-444444444441', 500.00, 'USD', 'released')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  EXCHANGE.BID / OFFER / DEAL / INVOICE / PAYMENT
-- =============================================================================

INSERT INTO exchange.offer (id, offerer_id, recipient_id, component_id, offer_type, amount, currency, terms, message, status) VALUES
  ('e0000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   'a0000001-0000-0000-0000-000000000004', 'buy', 430000.00, 'USD',
   '{"inspection_days":10,"financing":"conventional","close_days":30}'::jsonb,
   'Interested in purchasing Oak Ave property', 'open')
ON CONFLICT (id) DO NOTHING;

INSERT INTO exchange.invoice (id, issuer_id, recipient_id, line_items, subtotal, tax, total, currency, status) VALUES
  ('e1000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   '[{"description":"RE Playbook v1","qty":1,"unit_price":79.00}]'::jsonb,
   79.00, 0.00, 79.00, 'USD', 'paid'),
  ('e1000001-0000-0000-0000-000000000002', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   '[{"description":"RE Playbook v2 (loyal subscriber 20% off)","qty":1,"unit_price":103.20}]'::jsonb,
   103.20, 0.00, 103.20, 'USD', 'paid')
ON CONFLICT (id) DO NOTHING;

INSERT INTO exchange.payment (invoice_id, payer_id, payee_id, wallet_id, amount, currency, payment_method, status, paid_at) VALUES
  ('e1000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'b4444444-4444-4444-4444-444444444441', 79.00,  'USD', 'internal', 'completed', now() - interval '45 days'),
  ('e1000001-0000-0000-0000-000000000002', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'b4444444-4444-4444-4444-444444444441', 103.20, 'USD', 'internal', 'completed', now() - interval '5 days')
ON CONFLICT DO NOTHING;

INSERT INTO exchange.tax_record (account_id, tax_year, tax_type, jurisdiction, gross_income, deductions, tax_owed, tax_paid, status) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 2025, 'income',   'US', 185000.00, 42000.00, 37000.00, 37000.00, 'filed'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 2025, 'self_employment', 'US', 12000.00, 3000.00, 1271.00, 1271.00, 'filed'),
  ('11111111-1111-1111-1111-111111111111', 2025, 'income',   'US', 95000.00,  18000.00, 17000.00, 17000.00, 'filed')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  FUND.CAMPAIGN / CONTRIBUTION / CAPITAL_POOL
-- =============================================================================

INSERT INTO fund.campaign (id, owner_id, portfolio_id, title, description, campaign_type, goal_amount, raised_amount, currency, status, tags) VALUES
  ('fc000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'c5555555-5555-5555-5555-555555555555',
   'RE Mastermind Q2-2026 Cohort',
   'Fundraise to run the next 12-week RE mastermind cohort',
   'donation', 6000.00, 1000.00, 'USD', 'active', ARRAY['mastermind','real_estate','community']),
  ('fc000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555',
   'Kogi Platform Seed Round',
   'Seed fundraise to build and launch Kogi v1',
   'equity_crowdfund', 250000.00, 25000.00, 'USD', 'active', ARRAY['kogi','platform','seed'])
ON CONFLICT (id) DO NOTHING;

INSERT INTO fund.contribution (campaign_id, contributor_id, amount, currency, contribution_type, equity_pct, status) VALUES
  ('fc000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 500.00,   'USD', 'donation',   0,    'confirmed'),
  ('fc000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 500.00,   'USD', 'donation',   0,    'confirmed'),
  ('fc000001-0000-0000-0000-000000000002', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 10000.00, 'USD', 'investment', 2.0,  'confirmed'),
  ('fc000001-0000-0000-0000-000000000002', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 15000.00, 'USD', 'investment', 3.0,  'confirmed')
ON CONFLICT DO NOTHING;

INSERT INTO fund.capital_pool (id, owner_id, portfolio_id, pool_type, name, balance, currency, governance_model) VALUES
  ('f0000001-0000-0000-0000-000000000001', '11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555', 'operating',    'Kogi Ops Pool',       12500.00, 'USD', 'owner'),
  ('f0000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555', 'investment',   'Kogi Investor Pool',  25000.00, 'USD', 'vote'),
  ('f0000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a5555555-5555-5555-5555-555555555555', 'investment',   'Amara RE Deal Pool',  50000.00, 'USD', 'owner'),
  ('f0000001-0000-0000-0000-000000000004', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'c5555555-5555-5555-5555-555555555555', 'mutual_aid',   'Mastermind Aid Fund', 2000.00,  'USD', 'vote')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
--  STUDIO
-- =============================================================================

INSERT INTO studio.idea (id, owner_id, portfolio_id, title, summary, stage, tags) VALUES
  ('51000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b5555555-5555-5555-5555-555555555556',
   'RE PM SaaS Concept', 'Portfolio management platform for independent RE investors', 'mvp', ARRAY['saas','real_estate']),
  ('51000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555',
   'Kogi Mobile App', 'Native mobile experience for Kogi workers', 'concept', ARRAY['mobile','kogi']),
  ('51000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a5555555-5555-5555-5555-555555555556',
   'RE Masterclass Video Series', 'Video course companion to the playbook', 'idea', ARRAY['education','video','real_estate'])
ON CONFLICT (id) DO NOTHING;

INSERT INTO studio.design (owner_id, portfolio_id, title, design_type, tool, version, status) VALUES
  ('11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555', 'Kogi Dashboard V3 Mockup',    'mockup',       'Figma',         '3.0.0', 'active'),
  ('11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555', 'Kogi Platform Architecture',  'architecture', 'draw.io',       '1.2.0', 'active'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b5555555-5555-5555-5555-555555555556', 'RE PM SaaS Wireframes',       'mockup',       'Excalidraw',    '0.3.0', 'draft')
ON CONFLICT DO NOTHING;

INSERT INTO studio.note (owner_id, component_id, title, content, tags) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000003', 'Elm St Flip Notes', 'ARV estimate $390k. Contractor quote $42k rehab. Budget cushion $8k.', ARRAY['flip','notes','re']),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b0000001-0000-0000-0000-000000000001', 'RE Learning Notes', 'Focus on BRRRR strategy. Target duplex in Midwest market. Budget $50k for first deal.', ARRAY['learning','re','notes']),
  ('11111111-1111-1111-1111-111111111111', 'd0000001-0000-0000-0000-000000000001', 'Sprint 12 Notes',  'Prioritize schema + seed this sprint. gRPC endpoints next. Review provider system design.', ARRAY['kogi','sprint','dev'])
ON CONFLICT DO NOTHING;

INSERT INTO studio.tool_link (owner_id, portfolio_id, name, url, tool_type, category) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a5555555-5555-5555-5555-555555555555', 'DealCheck',       'https://dealcheck.io',           'external', 'real_estate'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a5555555-5555-5555-5555-555555555555', 'Zillow',          'https://zillow.com',             'external', 'real_estate'),
  ('11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555', 'Linear.app',      'https://linear.app',             'external', 'planning'),
  ('11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555', 'Claude.ai',       'https://claude.ai',              'ai',       'ai_assistant'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b5555555-5555-5555-5555-555555555556', 'GitHub',          'https://github.com/kwame-repm',  'external', 'developer')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  COMMUNITY.SPACE / SPACE_MEMBER
-- =============================================================================

INSERT INTO community.space (id, owner_id, name, description, space_type, visibility, tags) VALUES
  ('c5000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'RE Mastermind Q1-2026',      'Private mastermind cohort room',           'room',    'private',  ARRAY['real_estate','mastermind']),
  ('c5000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', 'Kogi Dev Community',         'Public dev discussion for Kogi contributors','channel', 'public',   ARRAY['kogi','dev','community']),
  ('c5000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'RE Investors Lounge',        'Public space for RE investors on Kogi',     'feed',    'public',   ARRAY['real_estate','investing']),
  ('c5000001-0000-0000-0000-000000000004', '11111111-1111-1111-1111-111111111111', 'Kogi Platform Announcements','Official Kogi announcements channel',        'channel', 'public',   ARRAY['kogi','announcements'])
ON CONFLICT (id) DO NOTHING;

INSERT INTO community.space_member (space_id, account_id, role) VALUES
  ('c5000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'owner'),
  ('c5000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'member'),
  ('c5000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'member'),
  ('c5000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', 'owner'),
  ('c5000001-0000-0000-0000-000000000002', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'member'),
  ('c5000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'owner'),
  ('c5000001-0000-0000-0000-000000000003', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'member'),
  ('c5000001-0000-0000-0000-000000000003', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'member'),
  ('c5000001-0000-0000-0000-000000000004', '11111111-1111-1111-1111-111111111111', 'owner'),
  ('c5000001-0000-0000-0000-000000000004', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'member'),
  ('c5000001-0000-0000-0000-000000000004', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'member'),
  ('c5000001-0000-0000-0000-000000000004', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'member')
ON CONFLICT (space_id, account_id) DO NOTHING;

-- =============================================================================
--  COMMUNITY.POST / MESSAGE / REACTION / NOTIFICATION / FEED
-- =============================================================================

INSERT INTO community.post (space_id, author_id, body, post_type, hashtags) VALUES
  ('c5000001-0000-0000-0000-000000000003', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   'Just closed on my second rental property 🏠 Cash-on-cash return is sitting at 9.2%. Happy to share my due diligence process if anyone is curious!',
   'post', ARRAY['#realestate','#investing','#cashflow']),
  ('c5000001-0000-0000-0000-000000000003', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   'Just finished Amara''s RE Playbook v2 — the creative financing section changed my perspective entirely. Highly recommend for anyone starting out.',
   'post', ARRAY['#realestate','#playbook','#learning']),
  ('c5000001-0000-0000-0000-000000000004', '11111111-1111-1111-1111-111111111111',
   'Kogi Alpha is coming Q2 2026 🚀 We''re looking for beta users. Drop a comment if interested!',
   'announcement', ARRAY['#kogi','#announcement','#beta']),
  ('c5000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc',
   'Week 1 mastermind recap: 12 investors, $2.3M combined deal pipeline discussed. Amazing insights shared this week!',
   'post', ARRAY['#mastermind','#realestate'])
ON CONFLICT DO NOTHING;

INSERT INTO community.message (space_id, sender_id, body, message_type) VALUES
  ('c5000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'Welcome to the RE Mastermind Q1-2026! Please introduce yourself and your current portfolio.',   'chat'),
  ('c5000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Hey everyone — Amara here. 2 rental properties, working on a flip. Excited to be here!',      'chat'),
  ('c5000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Kwame here! Software engineer by day, aspiring RE investor. On my first deal hunt!',           'chat'),
  ('c5000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', 'Sprint 12 kicked off — schema & seed data is our primary deliverable this sprint.',             'chat'),
  ('c5000001-0000-0000-0000-000000000002', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Looking forward to contributing to the portfolio module once the schema is locked in.',         'chat')
ON CONFLICT DO NOTHING;

INSERT INTO community.notification (account_id, notification_type, title, body, source_type, source_id) VALUES
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'new_content',     'Amara published Playbook v2!',       'A new version of RE Investment Playbook is available — you qualify for loyal subscriber discount.', 'component', 'a0000001-0000-0000-0000-000000000011'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'mastermind_msg',  'New message in mastermind',           'Zuri posted the Week 1 recap.',  'space', 'c5000001-0000-0000-0000-000000000001'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'purchase',        'Kwame purchased your playbook v2',   'New sale: $103.20',              'market_order', 'b0000001-0000-0000-0000-000000000002'),
  ('11111111-1111-1111-1111-111111111111', 'platform_event',  'Sprint 12 board ready',              'Sprint 12 is now active — schema + seed deliverables assigned.', 'wbs', 'd0000001-0000-0000-0000-000000000001')
ON CONFLICT DO NOTHING;

INSERT INTO community.feed_item (account_id, event_type, source_id, source_type, payload) VALUES
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'portfolio_update', 'a0000001-0000-0000-0000-000000000011', 'component',  '{"action":"published","name":"RE Playbook v2"}'::jsonb),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'post',             '1',                                    'community_post', '{"author":"Amara","preview":"Just closed on my second rental..."}'::jsonb),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'purchase',         'b0000001-0000-0000-0000-000000000002', 'market_order','{"buyer":"Kwame","amount":103.20}'::jsonb),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'marketplace',      'b1000001-0000-0000-0000-000000000003', 'listing',     '{"action":"purchase","buyer":"Kwame","buyer2":"Amara"}'::jsonb)
ON CONFLICT DO NOTHING;

-- =============================================================================
--  WORK.TASK / TIME_ENTRY / GIG / CONTRACT / OKR
-- =============================================================================

INSERT INTO work.task (owner_id, assignee_id, component_id, title, task_type, priority, status, estimate_hours, due_at, tags) VALUES
  ('11111111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'd0000001-0000-0000-0000-000000000001',
   'Build full schema.sql',          'feature', 'critical', 'in_progress', 8.0,  now() + interval '2 days',  ARRAY['kogi','db','schema']),
  ('11111111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'd0000001-0000-0000-0000-000000000001',
   'Build full seed.sql',            'feature', 'critical', 'in_progress', 5.0,  now() + interval '2 days',  ARRAY['kogi','db','seed']),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000003',
   'Finalize rehab contractor bids', 'task',    'high',     'todo',         3.0,  now() + interval '7 days',  ARRAY['flip','re','contractor']),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000011',
   'Publish playbook v2 newsletter', 'task',    'normal',   'completed',    1.0,  now() - interval '5 days',  ARRAY['marketing','playbook']),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b0000001-0000-0000-0000-000000000001',
   'Complete 10 investor interviews', 'research', 'high',   'in_progress',  10.0, now() + interval '14 days', ARRAY['research','saas','re'])
ON CONFLICT DO NOTHING;

INSERT INTO work.time_entry (account_id, component_id, description, hours, hourly_rate, currency, billable, started_at, ended_at) VALUES
  ('11111111-1111-1111-1111-111111111111', 'd0000001-0000-0000-0000-000000000001', 'Schema design & build',   6.5, 115.00, 'USD', false, now() - interval '1 day', now() - interval '18 hours'),
  ('11111111-1111-1111-1111-111111111111', 'd0000001-0000-0000-0000-000000000001', 'Seed data construction',  4.0, 115.00, 'USD', false, now() - interval '6 hours', now() - interval '2 hours'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000003', 'Contractor walkthroughs', 3.0, 250.00, 'USD', false, now() - interval '3 days', now() - interval '2.5 days')
ON CONFLICT DO NOTHING;

INSERT INTO work.gig (owner_id, listing_id, title, description, gig_type, price, currency, skills, status) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'b1000001-0000-0000-0000-000000000001',
   'RE Playbook Review Session', '1-hour walkthrough of playbook with Q&A', 'fixed', 150.00, 'USD',
   ARRAY['real_estate','coaching','investing'], 'open'),
  ('11111111-1111-1111-1111-111111111111', NULL,
   'Kogi Platform Consulting', 'Platform architecture & API integration consulting', 'hourly', 115.00, 'USD',
   ARRAY['rust','platform_architecture','postgresql','api'], 'open')
ON CONFLICT DO NOTHING;

INSERT INTO work.okr (owner_id, component_id, title, objective, target_metric, current_value, target_value, period_start, period_end) VALUES
  ('11111111-1111-1111-1111-111111111111', 'd0000001-0000-0000-0000-000000000002',
   'Launch Kogi Alpha', 'Release Kogi Alpha to 100 internal testers', 'beta_users', 0, 100, '2026-01-01', '2026-06-30'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000001',
   '10 RE Deals by 2028', 'Acquire 10 cashflowing properties', 'properties_owned', 2, 10, '2024-01-01', '2028-12-31'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'a0000001-0000-0000-0000-000000000010',
   '500 Playbook Sales', 'Sell 500 copies of RE Playbook', 'sales_count', 87, 500, '2025-01-01', '2026-12-31')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AO.AUTONOMOUS_ORG / ORG_MEMBER / PROPOSAL / VOTE
-- =============================================================================

INSERT INTO ao.autonomous_org (id, owner_id, portfolio_id, name, org_type, charter, governance_model, status, tags) VALUES
  ('a0000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'c5555555-5555-5555-5555-555555555555',
   'RE Mastermind Collective', 'collective',
   '{"purpose":"Collective RE education and accountability","bylaws":"Consensus voting on major decisions","dues":500}'::jsonb,
   'flat', 'active', ARRAY['real_estate','collective','education']),
  ('a0000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', '55555555-5555-5555-5555-555555555555',
   'Kogi Platform Co-op', 'cooperative',
   '{"purpose":"Platform governance and revenue sharing","bylaws":"One member one vote","profit_sharing":0.20}'::jsonb,
   'holacracy', 'active', ARRAY['kogi','cooperative','platform'])
ON CONFLICT (id) DO NOTHING;

INSERT INTO ao.org_member (org_id, account_id, role, equity_pct, voting_weight) VALUES
  ('a0000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'owner',    0,    1.0),
  ('a0000001-0000-0000-0000-000000000001', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'member',   0,    1.0),
  ('a0000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'member',   0,    1.0),
  ('a0000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111', 'owner',    40.0, 2.0),
  ('a0000001-0000-0000-0000-000000000002', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'member',   2.0,  1.0),
  ('a0000001-0000-0000-0000-000000000002', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'member',   3.0,  1.0)
ON CONFLICT (org_id, account_id) DO NOTHING;

INSERT INTO ao.proposal (org_id, proposer_id, title, body, proposal_type, status, quorum_pct, pass_threshold, voting_ends_at, submitted_at) VALUES
  ('a0000001-0000-0000-0000-000000000001', 'cccccccc-cccc-cccc-cccc-cccccccccccc',
   'Run Q2-2026 Mastermind Cohort',
   'Proposal to run a second 12-week cohort in Q2 2026, expanding to 20 participants.',
   'general', 'active', 67, 51, now() + interval '14 days', now()),
  ('a0000001-0000-0000-0000-000000000002', '11111111-1111-1111-1111-111111111111',
   'Allocate $20k to Engineering Sprint Fund',
   'Reserve $20k from Investor Pool for engineering contractors in Q2 2026.',
   'budget', 'active', 67, 67, now() + interval '7 days', now())
ON CONFLICT DO NOTHING;

-- =============================================================================
--  AI.AGENT_SESSION / ACTION_LOG / RECOMMENDATION / PERSONA_PROFILE / SEARCH
-- =============================================================================

INSERT INTO ai.agent_session (id, account_id, model_name, started_at) VALUES
  ('a5000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'sambara-oba-v1', now() - interval '2 days'),
  ('a5000001-0000-0000-0000-000000000002', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'sambara-oba-v1', now() - interval '5 days')
ON CONFLICT (id) DO NOTHING;

INSERT INTO ai.agent_action_log (session_id, action_type, intent, input_payload, output_payload, requires_human_review) VALUES
  ('a5000001-0000-0000-0000-000000000001', 'recommend',  'find_playbooks',
   '{"query":"real estate investing beginner","user_tags":["beginner","real_estate"]}'::jsonb,
   '{"recommended":["a0000001-0000-0000-0000-000000000010"],"reason":"Top rated playbook matching user profile"}'::jsonb, false),
  ('a5000001-0000-0000-0000-000000000001', 'recommend',  'find_masterminds',
   '{"query":"real estate mastermind community"}'::jsonb,
   '{"recommended":["c0000001-0000-0000-0000-000000000001"],"reason":"Active mastermind with high engagement"}'::jsonb, false),
  ('a5000001-0000-0000-0000-000000000002', 'search',     'find_subscribers',
   '{"query":"who subscribed to my playbooks"}'::jsonb,
   '{"subscribers":["bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"],"count":1}'::jsonb, false)
ON CONFLICT DO NOTHING;

INSERT INTO ai.recommendation (account_id, session_id, rec_type, recommended_id, recommended_type, score, reason, accepted) VALUES
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'a5000001-0000-0000-0000-000000000001', 'component',  'a0000001-0000-0000-0000-000000000010', 'playbook', 0.94, 'Top-rated RE playbook matching beginner profile', true),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'a5000001-0000-0000-0000-000000000001', 'component',  'c0000001-0000-0000-0000-000000000001', 'mastermind', 0.87, 'Highly engaged mastermind matching RE interests', true),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'a5000001-0000-0000-0000-000000000001', 'user',       'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'account',   0.91, 'Content creator aligned with your RE learning goals', true)
ON CONFLICT DO NOTHING;

INSERT INTO ai.persona_profile (account_id, interests, behavior_tags, preferences, sentiment_score) VALUES
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   ARRAY['real_estate','software','investing','saas'],
   ARRAY['learner','builder','early_adopter'],
   '{"preferred_content_type":"playbook","notification_frequency":"daily","discovery_mode":"recommendation"}'::jsonb,
   0.78),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   ARRAY['real_estate','education','investing','wealth_building'],
   ARRAY['creator','educator','community_leader'],
   '{"preferred_content_type":"analytics","notification_frequency":"weekly","discovery_mode":"search"}'::jsonb,
   0.85),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc',
   ARRAY['community','real_estate','education','mastermind'],
   ARRAY['community_builder','organizer','educator'],
   '{"preferred_content_type":"community","notification_frequency":"instant","discovery_mode":"network"}'::jsonb,
   0.91)
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO ai.search_index (source_type, source_id, tags) VALUES
  ('component',  'a0000001-0000-0000-0000-000000000010', ARRAY['real_estate','playbook','investing','education']),
  ('component',  'a0000001-0000-0000-0000-000000000011', ARRAY['real_estate','playbook','advanced','investing']),
  ('component',  'c0000001-0000-0000-0000-000000000001', ARRAY['mastermind','real_estate','community','education']),
  ('listing',    'b1000001-0000-0000-0000-000000000001', ARRAY['real_estate','playbook','marketplace']),
  ('listing',    'b1000001-0000-0000-0000-000000000002', ARRAY['real_estate','playbook','advanced','marketplace']),
  ('listing',    'b1000001-0000-0000-0000-000000000003', ARRAY['mastermind','real_estate','community','service'])
ON CONFLICT (source_type, source_id) DO NOTHING;

-- =============================================================================
--  PROVIDER.REGISTRY / RESOURCES / ACCOUNT_CONNECTION
-- =============================================================================

INSERT INTO provider.registry (id, name, description, provider_type, category, website_url, api_base_url, auth_type, status) VALUES
  ('de000001-0000-0000-0000-000000000001', 'Stripe',     'Payment processing & financial infrastructure',  'payment',     'finance',       'https://stripe.com',    'https://api.stripe.com',    'api_key', 'active'),
  ('de000001-0000-0000-0000-000000000002', 'GitHub',     'Code hosting & developer collaboration',         'integration', 'developer',     'https://github.com',    'https://api.github.com',    'oauth2',  'active'),
  ('de000001-0000-0000-0000-000000000003', 'Zillow',     'Real estate data & listings',                    'integration', 'real_estate',   'https://zillow.com',    'https://api.zillow.com',    'api_key', 'active'),
  ('de000001-0000-0000-0000-000000000004', 'Slack',      'Team communication & notifications',             'integration', 'community',     'https://slack.com',     'https://slack.com/api',     'oauth2',  'active'),
  ('de000001-0000-0000-0000-000000000005', 'Upwork',     'Freelance talent marketplace',                   'affiliate',   'marketplace',   'https://upwork.com',    'https://api.upwork.com',    'oauth2',  'active'),
  ('de000001-0000-0000-0000-000000000006', 'DealCheck',  'Real estate deal analysis tool',                 'affiliate',   'real_estate',   'https://dealcheck.io',  NULL,                        'api_key', 'active'),
  ('de000001-0000-0000-0000-000000000007', 'Fiverr',     'Freelance services marketplace',                 'affiliate',   'marketplace',   'https://fiverr.com',    'https://api.fiverr.com',    'oauth2',  'active'),
  ('de000001-0000-0000-0000-000000000008', 'Robinhood',  'Investment & trading platform',                  'affiliate',   'finance',       'https://robinhood.com', NULL,                        'oauth2',  'active'),
  ('de000001-0000-0000-0000-000000000009', 'Patreon',    'Creator membership & fundraising platform',      'affiliate',   'marketplace',   'https://patreon.com',   'https://api.patreon.com',   'oauth2',  'active'),
  ('de000001-0000-0000-0000-000000000010', 'Google',     'OAuth identity provider',                        'oauth',       'general',       'https://google.com',    'https://oauth2.googleapis.com','oauth2','active')
ON CONFLICT (name) DO NOTHING;

INSERT INTO provider.provider_resource (provider_id, resource_name, resource_type, description, endpoint, version) VALUES
  ('de000001-0000-0000-0000-000000000001', 'Payment Intent',      'endpoint', 'Create & manage payment intents',  '/v1/payment_intents', '2024-04-10'),
  ('de000001-0000-0000-0000-000000000001', 'Webhook Events',      'webhook',  'Stripe webhook event stream',      '/v1/webhook_endpoints','2024-04-10'),
  ('de000001-0000-0000-0000-000000000002', 'Repository API',      'endpoint', 'Repository CRUD operations',       '/repos',              '2022-11-28'),
  ('de000001-0000-0000-0000-000000000003', 'Property Search',     'endpoint', 'Zillow property search API',       '/v2/propertySearch',  '2.0'),
  ('de000001-0000-0000-0000-000000000004', 'Slack Webhooks',      'webhook',  'Post messages to Slack channels',  '/services/webhooks',  '1.0')
ON CONFLICT DO NOTHING;

INSERT INTO provider.account_connection (account_id, provider_id, external_ref, scopes, status) VALUES
  ('11111111-1111-1111-1111-111111111111', 'de000001-0000-0000-0000-000000000001', 'acct_dominic_001',  ARRAY['read_write','webhooks'], 'active'),
  ('11111111-1111-1111-1111-111111111111', 'de000001-0000-0000-0000-000000000002', 'dominic-dev',       ARRAY['repo','user'],           'active'),
  ('11111111-1111-1111-1111-111111111111', 'de000001-0000-0000-0000-000000000004', 'kogi-community',    ARRAY['chat:write','channels'],  'active'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'de000001-0000-0000-0000-000000000001', 'acct_amara_001',    ARRAY['read_write'],            'active'),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'de000001-0000-0000-0000-000000000003', 'amara-re',          ARRAY['property_search'],       'active'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'de000001-0000-0000-0000-000000000002', 'kwame-dev',         ARRAY['repo','user'],           'active'),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'de000001-0000-0000-0000-000000000004', 'zuri-host',         ARRAY['chat:write','channels'],  'active')
ON CONFLICT (account_id, provider_id) DO NOTHING;

-- =============================================================================
--  PROVIDER.AFFILIATE / AFFILIATE_LINK / AFFILIATE_COMMISSION
-- =============================================================================

INSERT INTO provider.affiliate (id, provider_id, account_id, affiliate_code, commission_rate, commission_type, status) VALUES
  -- Amara is a DealCheck affiliate
  ('da000001-0000-0000-0000-000000000001', 'de000001-0000-0000-0000-000000000006', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   'AMARA-DEALCHECK', 0.20, 'percentage', 'active'),
  -- Amara is a Patreon affiliate
  ('da000001-0000-0000-0000-000000000002', 'de000001-0000-0000-0000-000000000009', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
   'AMARA-PATREON', 0.08, 'percentage', 'active'),
  -- Zuri is an Upwork affiliate
  ('da000001-0000-0000-0000-000000000003', 'de000001-0000-0000-0000-000000000005', 'cccccccc-cccc-cccc-cccc-cccccccccccc',
   'ZURI-UPWORK', 0.05, 'percentage', 'active'),
  -- Dominic is a Fiverr affiliate
  ('da000001-0000-0000-0000-000000000004', 'de000001-0000-0000-0000-000000000007', '11111111-1111-1111-1111-111111111111',
   'KOGI-FIVERR', 0.10, 'percentage', 'active')
ON CONFLICT (affiliate_code) DO NOTHING;

INSERT INTO provider.affiliate_link (id, affiliate_id, provider_id, listing_id, link_code, url, discount_type, discount_value, discount_code, max_uses, status) VALUES
  -- Amara's DealCheck link inside playbook v1
  ('a1000001-0000-0000-0000-000000000001', 'da000001-0000-0000-0000-000000000001', 'de000001-0000-0000-0000-000000000006',
   NULL, 'AMARA-DC-PB1', 'https://dealcheck.io?ref=AMARA-DC-PB1', 'percentage', 20.0, 'AMARA20', NULL, 'active'),
  -- Amara's playbook v2 includes updated DealCheck link
  ('a1000001-0000-0000-0000-000000000002', 'da000001-0000-0000-0000-000000000001', 'de000001-0000-0000-0000-000000000006',
   NULL, 'AMARA-DC-PB2', 'https://dealcheck.io?ref=AMARA-DC-PB2', 'percentage', 25.0, 'AMARA25', NULL, 'active'),
  -- Zuri's mastermind Upwork link for finding workers
  ('a1000001-0000-0000-0000-000000000003', 'da000001-0000-0000-0000-000000000003', 'de000001-0000-0000-0000-000000000005',
   NULL, 'ZURI-UW-MM',   'https://upwork.com?ref=ZURI-UW-MM',    'flat',       15.0, NULL,       500, 'active'),
  -- Dominic's Fiverr link inside Kogi developer module
  ('a1000001-0000-0000-0000-000000000004', 'da000001-0000-0000-0000-000000000004', 'de000001-0000-0000-0000-000000000007',
   NULL, 'KOGI-FVR',     'https://fiverr.com?ref=KOGI-FVR',      'percentage', 10.0, 'KOGI10',   1000,'active')
ON CONFLICT (link_code) DO NOTHING;

-- Kwame used Amara's DealCheck affiliate link (only once — enforced by unique constraint)
INSERT INTO provider.affiliate_link_usage (affiliate_link_id, account_id, used_at) VALUES
  ('a1000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', now() - interval '40 days')
ON CONFLICT (affiliate_link_id, account_id) DO NOTHING;

-- Commission earned from Kwame's DealCheck signup via Amara's link
INSERT INTO provider.affiliate_commission (affiliate_id, affiliate_link_id, referred_account_id, sale_amount, commission_amount, currency, status, paid_at) VALUES
  ('da000001-0000-0000-0000-000000000001', 'a1000001-0000-0000-0000-000000000001', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
   99.00, 19.80, 'USD', 'paid', now() - interval '38 days')
ON CONFLICT DO NOTHING;

-- =============================================================================
--  KERN.EVENT_LOG  (Platform events)
-- =============================================================================

INSERT INTO kern.event_log (topic, source_node, payload) VALUES
  ('portfolio.component.created', 'kogi-host-001', '{"component_id":"a0000001-0000-0000-0000-000000000010","type":"playbook","actor":"amara"}'::jsonb),
  ('market.order.completed',      'kogi-host-001', '{"order_id":"b0000001-0000-0000-0000-000000000001","buyer":"kwame","seller":"amara","amount":79.00}'::jsonb),
  ('market.order.completed',      'kogi-host-001', '{"order_id":"b0000001-0000-0000-0000-000000000002","buyer":"kwame","seller":"amara","amount":103.20}'::jsonb),
  ('community.post.created',      'kogi-gw-001',   '{"space_id":"c5000001-0000-0000-0000-000000000003","author":"amara","hashtags":["#realestate"]}'::jsonb),
  ('fund.contribution.confirmed', 'kogi-host-001', '{"campaign_id":"fc000001-0000-0000-0000-000000000002","contributor":"amara","amount":10000}'::jsonb),
  ('ai.recommendation.accepted',  'kogi-eng-001',  '{"account_id":"bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb","recommended":"a0000001-0000-0000-0000-000000000010"}'::jsonb)
ON CONFLICT DO NOTHING;

-- =============================================================================
--  KERN.HEARTBEAT_LOG
-- =============================================================================

INSERT INTO kern.heartbeat_log (node_id, status, cpu_pct, memory_pct, payload) VALUES
  ('kogi-host-001', 'healthy',  12.3, 34.5, '{"uptime_secs":86400,"active_sessions":3}'::jsonb),
  ('kogi-gw-001',   'healthy',  5.1,  18.2, '{"messages_per_sec":142}'::jsonb),
  ('kogi-eng-001',  'healthy',  22.7, 55.0, '{"jobs_queued":4,"jobs_running":2}'::jsonb)
ON CONFLICT DO NOTHING;

-- =============================================================================
--  PORTFOLIO.EVENT_LOG  (Portfolio domain events)
-- =============================================================================

INSERT INTO portfolio.event_log (portfolio_id, event_id, sequence, actor_id, event_kind, payload) VALUES
  ('a5555555-5555-5555-5555-555555555555', 'evt-000001', 1, 'amara', 'ComponentCreated',      '{"component_id":"a0000001-0000-0000-0000-000000000010","name":"RE Playbook v1"}'::jsonb),
  ('a5555555-5555-5555-5555-555555555555', 'evt-000002', 2, 'amara', 'ComponentUpdated',      '{"component_id":"a0000001-0000-0000-0000-000000000010","field":"status","value":"active"}'::jsonb),
  ('a5555555-5555-5555-5555-555555555555', 'evt-000003', 3, 'amara', 'ComponentCreated',      '{"component_id":"a0000001-0000-0000-0000-000000000011","name":"RE Playbook v2"}'::jsonb),
  ('b5555555-5555-5555-5555-555555555555', 'evt-000004', 1, 'kwame', 'ComponentCreated',      '{"component_id":"b0000001-0000-0000-0000-000000000002","name":"Purchased Playbook v1"}'::jsonb),
  ('b5555555-5555-5555-5555-555555555555', 'evt-000005', 2, 'kwame', 'MemberAdded',           '{"user_id":"bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb","component_id":"a0000001-0000-0000-0000-000000000010","tier":"subscriber"}'::jsonb),
  ('55555555-5555-5555-5555-555555555555', 'evt-000006', 1, 'dominic','ComponentCreated',     '{"component_id":"d0000001-0000-0000-0000-000000000001","name":"Kogi MVP Development"}'::jsonb),
  ('55555555-5555-5555-5555-555555555555', 'evt-000007', 2, 'dominic','SnapshotSaved',        '{"snapshot_id":"snap-000003"}'::jsonb)
ON CONFLICT (event_id) DO NOTHING;

-- =============================================================================
--  KERN.RESOURCE_ALLOCATION
-- =============================================================================

INSERT INTO kern.resource_allocation (component_id, resource_kind, total, allocated, consumed, denomination) VALUES
  ('d0000001-0000-0000-0000-000000000001', 'Budget',       150000, 80000,  62000,  'USD'),
  ('d0000001-0000-0000-0000-000000000001', 'PersonHours',  2400,   1800,   1240,   'person-hours'),
  ('a0000001-0000-0000-0000-000000000003', 'Budget',       50000,  42000,  38000,  'USD'),
  ('c0000001-0000-0000-0000-000000000001', 'PersonHours',  480,    300,    120,    'person-hours')
ON CONFLICT DO NOTHING;
