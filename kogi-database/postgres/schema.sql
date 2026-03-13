-- =============================================================================
--  Kogi Platform — PostgreSQL 16+  schema.sql  (Reconciled Full Build)
--  Covers: auth · kern · portfolio · wbs · market · exchange · fund · studio
--           community · work · ao · ai · provider
--
--  Design sources: portfolio_system.rs (v2.0), design.md, notes.md,
--                  notes2.md, sdd.md, README.md
-- =============================================================================

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS btree_gin;

-- ── Schemas ──────────────────────────────────────────────────────────────────
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS kern;
CREATE SCHEMA IF NOT EXISTS portfolio;
CREATE SCHEMA IF NOT EXISTS wbs;
CREATE SCHEMA IF NOT EXISTS market;
CREATE SCHEMA IF NOT EXISTS exchange;
CREATE SCHEMA IF NOT EXISTS studio;
CREATE SCHEMA IF NOT EXISTS community;
CREATE SCHEMA IF NOT EXISTS work;
CREATE SCHEMA IF NOT EXISTS fund;
CREATE SCHEMA IF NOT EXISTS ao;
CREATE SCHEMA IF NOT EXISTS ai;
CREATE SCHEMA IF NOT EXISTS provider;

-- =============================================================================
--  AUTH
-- =============================================================================

CREATE TABLE IF NOT EXISTS auth.account (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT        NOT NULL UNIQUE,
    password_hash   TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.api_key (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    key_hash        TEXT        NOT NULL UNIQUE,
    label           TEXT        NOT NULL,
    scopes          TEXT[]      NOT NULL DEFAULT '{}',
    expires_at      TIMESTAMPTZ,
    last_used_at    TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.session (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES auth.account(id),
    access_token_hash   TEXT        NOT NULL,
    refresh_token_hash  TEXT,
    expires_at          TIMESTAMPTZ NOT NULL,
    ip_address          INET,
    device_fingerprint  TEXT,
    user_agent          TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.mfa_config (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id) UNIQUE,
    mfa_type        TEXT        NOT NULL DEFAULT 'totp',
    secret_hash     TEXT        NOT NULL,
    recovery_codes  TEXT[]      NOT NULL DEFAULT '{}',
    enabled         BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.federated_identity (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    provider        TEXT        NOT NULL,   -- 'google','github','okta','saml'
    subject_id      TEXT        NOT NULL,
    email           TEXT,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, subject_id)
);

CREATE TABLE IF NOT EXISTS auth.rbac_role (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT        NOT NULL UNIQUE,
    description     TEXT,
    permissions     TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.account_role (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    role_name       TEXT        NOT NULL,
    scope           TEXT        NOT NULL DEFAULT 'platform',  -- platform | module | resource
    scope_ref       UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, role_name, scope, scope_ref)
);

CREATE TABLE IF NOT EXISTS auth.identity (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    display_name    TEXT        NOT NULL,
    avatar_url      TEXT,
    bio             TEXT,
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.identity_persona (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id     UUID        NOT NULL REFERENCES auth.identity(id),
    persona_type    TEXT        NOT NULL,  -- investor | developer | donor | creator | consultant ...
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_id, persona_type)
);

CREATE TABLE IF NOT EXISTS auth.identity_role (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id     UUID        NOT NULL REFERENCES auth.identity(id),
    role_name       TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_id, role_name)
);

CREATE TABLE IF NOT EXISTS auth.identity_worker_type (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id     UUID        NOT NULL REFERENCES auth.identity(id),
    worker_type     TEXT        NOT NULL,  -- freelancer | consultant | entrepreneur | employee | contractor
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_id, worker_type)
);

-- identity_profile: typed profiles (personal / work / business / community / custom)
CREATE TABLE IF NOT EXISTS auth.identity_profile (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id     UUID        NOT NULL REFERENCES auth.identity(id),
    profile_type    TEXT        NOT NULL,
    name            TEXT        NOT NULL,
    handle          TEXT        UNIQUE,
    avatar_url      TEXT,
    bio             TEXT,
    settings        JSONB       NOT NULL DEFAULT '{}',
    options         JSONB       NOT NULL DEFAULT '{}',
    parameters      JSONB       NOT NULL DEFAULT '{}',
    visibility      TEXT        NOT NULL DEFAULT 'private',
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    handle          TEXT        NOT NULL UNIQUE,
    display_name    TEXT        NOT NULL,
    persona         TEXT        NOT NULL DEFAULT 'independent_worker',
    timezone        TEXT        NOT NULL DEFAULT 'UTC',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.connection_registry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    connection_type TEXT        NOT NULL,   -- software | finance | community | marketplace ...
    provider        TEXT        NOT NULL,   -- github | stripe | slack | etc
    account_ref     TEXT        NOT NULL,
    access_token    TEXT,                   -- encrypted
    refresh_token   TEXT,                   -- encrypted
    metadata        JSONB       NOT NULL DEFAULT '{}',
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.contact_directory (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id          UUID        NOT NULL REFERENCES auth.identity_profile(id),
    contact_name        TEXT        NOT NULL,
    email               TEXT,
    phone               TEXT,
    organization        TEXT,
    relationship_type   TEXT        NOT NULL DEFAULT 'professional',
    tags                TEXT[]      NOT NULL DEFAULT '{}',
    social_links        JSONB       NOT NULL DEFAULT '{}',
    notes               TEXT,
    status              TEXT        NOT NULL DEFAULT 'active',
    metadata            JSONB       NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.asset_vault (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    asset_type      TEXT        NOT NULL,
    name            TEXT        NOT NULL,
    description     TEXT,
    value_amount    NUMERIC(20,4),
    currency        CHAR(3)     DEFAULT 'USD',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_tool (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    tool_name       TEXT        NOT NULL,
    tool_category   TEXT        NOT NULL,
    tool_url        TEXT,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_program (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    program_name    TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'active',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_project (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    project_ref     TEXT        NOT NULL,
    source_module   TEXT        NOT NULL DEFAULT 'wbs',
    status          TEXT        NOT NULL DEFAULT 'active',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.skill (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    name            TEXT        NOT NULL,
    category        TEXT        NOT NULL,
    proficiency     TEXT        NOT NULL DEFAULT 'intermediate',  -- beginner|intermediate|expert
    verified        BOOLEAN     NOT NULL DEFAULT false,
    endorsements    INT         NOT NULL DEFAULT 0,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  PORTFOLIO WORKSPACE BINDINGS
-- =============================================================================

CREATE TABLE IF NOT EXISTS portfolio.workspace (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    workspace_type  TEXT        NOT NULL DEFAULT 'personal',
    name            TEXT        NOT NULL,
    description     TEXT,
    settings        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_workspace_binding (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    workspace_id    UUID        NOT NULL REFERENCES portfolio.workspace(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (profile_id, workspace_id)
);

-- =============================================================================
--  KERN
-- =============================================================================

CREATE TABLE IF NOT EXISTS kern.module_registry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id       TEXT        NOT NULL UNIQUE,
    module_kind     TEXT        NOT NULL,
    language        TEXT        NOT NULL,
    version         TEXT        NOT NULL DEFAULT '0.1.0',
    endpoint        TEXT,
    status          TEXT        NOT NULL DEFAULT 'active',
    registered_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.node_registry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id         TEXT        NOT NULL UNIQUE,
    node_type       TEXT        NOT NULL DEFAULT 'host',   -- host | service | engine | gateway
    host            TEXT        NOT NULL,
    port            INT         NOT NULL,
    region          TEXT,
    role            TEXT        NOT NULL DEFAULT 'follower',  -- leader | follower
    status          TEXT        NOT NULL DEFAULT 'active',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.heartbeat_log (
    id              BIGSERIAL   PRIMARY KEY,
    node_id         TEXT        NOT NULL,
    status          TEXT        NOT NULL,   -- healthy | degraded | failing
    cpu_pct         NUMERIC(5,2),
    memory_pct      NUMERIC(5,2),
    payload         JSONB       NOT NULL DEFAULT '{}',
    emitted_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.event_log (
    id              BIGSERIAL   PRIMARY KEY,
    topic           TEXT        NOT NULL,
    source_node     TEXT,
    payload         JSONB       NOT NULL,
    emitted_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.scheduler_job (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID,
    topic           TEXT        NOT NULL,
    cron_expr       TEXT        NOT NULL,
    payload_template JSONB      NOT NULL,
    next_run_at     TIMESTAMPTZ NOT NULL,
    last_run_at     TIMESTAMPTZ,
    enabled         BOOLEAN     NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.rbac_policy (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_name     TEXT        NOT NULL UNIQUE,
    resource_type   TEXT        NOT NULL,
    action          TEXT        NOT NULL,
    conditions      JSONB       NOT NULL DEFAULT '{}',
    effect          TEXT        NOT NULL DEFAULT 'allow',   -- allow | deny
    priority        INT         NOT NULL DEFAULT 100,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.resource_allocation (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL,
    resource_kind   TEXT        NOT NULL,   -- Budget | PersonHours | StoryPoints | ComputeUnits | StorageGiB
    total           NUMERIC(20,4) NOT NULL DEFAULT 0,
    allocated       NUMERIC(20,4) NOT NULL DEFAULT 0,
    consumed        NUMERIC(20,4) NOT NULL DEFAULT 0,
    denomination    TEXT        NOT NULL DEFAULT 'USD',
    period          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  PORTFOLIO SYSTEM
--  Mirrors portfolio_system.rs §11-§25 (ComponentMetadata / ComponentData /
--  Item / Container / GraphEdge / EventLog / CrdtLog / Governance / Snapshot)
-- =============================================================================

-- Core portfolio table
CREATE TABLE IF NOT EXISTS portfolio.portfolio (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID        NOT NULL REFERENCES portfolio.workspace(id),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    domain          TEXT        NOT NULL DEFAULT 'general',
    mission         TEXT,
    focus_areas     TEXT[]      NOT NULL DEFAULT '{}',
    visibility      TEXT        NOT NULL DEFAULT 'private',
    status          TEXT        NOT NULL DEFAULT 'active',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_portfolio_binding (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id      UUID        NOT NULL REFERENCES auth.identity_profile(id),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (profile_id, portfolio_id)
);

-- Universal component table (§12 ComponentData / §16 Component)
-- Holds both Items and Containers; component_kind distinguishes them.
CREATE TABLE IF NOT EXISTS portfolio.component (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id        UUID        REFERENCES portfolio.portfolio(id),
    component_kind      TEXT        NOT NULL,  -- item | container
    -- ItemCategory:  portfolio | program | project | resource | artifact | asset | sub_portfolio
    -- ContainerCategory: binder | book | record | folder | registry | archive
    category            TEXT        NOT NULL,
    -- Book sub-kind only when category='book'
    book_kind           TEXT,                  -- notebook | contactbook | playbook | schedulebook | planbook | guidebook | itembook
    name                TEXT        NOT NULL,
    description         TEXT,
    status              TEXT        NOT NULL DEFAULT 'draft',
    state               TEXT        NOT NULL DEFAULT 'initializing',
    visibility          TEXT        NOT NULL DEFAULT 'private',
    -- metadata fields (§11 ComponentMetadata)
    version             TEXT        NOT NULL DEFAULT '0.1.0',
    last_actor          TEXT,
    vector_clock        JSONB       NOT NULL DEFAULT '{}',
    properties          JSONB       NOT NULL DEFAULT '{}',
    budget              NUMERIC(20,4),
    budget_spent        NUMERIC(20,4) NOT NULL DEFAULT 0,
    resource_units      NUMERIC(20,4) NOT NULL DEFAULT 0,
    -- tags / hashtags / topics
    tags                TEXT[]      NOT NULL DEFAULT '{}',
    hashtags            TEXT[]      NOT NULL DEFAULT '{}',
    topics              TEXT[]      NOT NULL DEFAULT '{}',
    -- extension
    addons              UUID[]      NOT NULL DEFAULT '{}',
    plugin_configs      JSONB       NOT NULL DEFAULT '{}',
    -- payload (item-type specific data as JSONB, mirrors ItemType/ContainerType payloads)
    payload             JSONB       NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Component owners (many per component)
CREATE TABLE IF NOT EXISTS portfolio.component_owner (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    user_id         UUID        NOT NULL REFERENCES auth.account(id),
    is_primary      BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (component_id, user_id)
);

-- Policy IDs attached to components
CREATE TABLE IF NOT EXISTS portfolio.component_policy (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    policy_id       UUID        NOT NULL REFERENCES kern.rbac_policy(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (component_id, policy_id)
);

-- User roles per component (§10 ComponentUsers — permission_map)
CREATE TABLE IF NOT EXISTS portfolio.component_user (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    user_id         UUID        NOT NULL REFERENCES auth.account(id),
    -- Viewer | Subscriber | Contributor | Editor | Manager | Owner | Admin
    permission_tier TEXT        NOT NULL DEFAULT 'viewer',
    -- role buckets: owner | editor | watcher | subscriber | follower | investor | donor
    role_bucket     TEXT        NOT NULL DEFAULT 'follower',
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (component_id, user_id)
);

-- Graph edges between components (§17 GraphEdge)
-- Hierarchy | Dependency | Link | Contains | Federation
CREATE TABLE IF NOT EXISTS portfolio.graph_edge (
    id              TEXT        PRIMARY KEY,
    from_id         UUID        NOT NULL REFERENCES portfolio.component(id),
    to_id           UUID        NOT NULL REFERENCES portfolio.component(id),
    kind            TEXT        NOT NULL,  -- hierarchy | dependency | link | contains | federation
    label           TEXT,
    properties      JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Component analytics (§9 ComponentAnalytics)
CREATE TABLE IF NOT EXISTS portfolio.component_analytics (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id            UUID        NOT NULL REFERENCES portfolio.component(id) UNIQUE,
    -- traffic
    clicks                  BIGINT      NOT NULL DEFAULT 0,
    click_through_rate      NUMERIC(8,6) NOT NULL DEFAULT 0,
    view_time_seconds       BIGINT      NOT NULL DEFAULT 0,
    impressions             BIGINT      NOT NULL DEFAULT 0,
    -- engagement
    likes                   BIGINT      NOT NULL DEFAULT 0,
    comments                BIGINT      NOT NULL DEFAULT 0,
    shares                  BIGINT      NOT NULL DEFAULT 0,
    saves                   BIGINT      NOT NULL DEFAULT 0,
    bookmarks               BIGINT      NOT NULL DEFAULT 0,
    reactions               JSONB       NOT NULL DEFAULT '{}',
    engagement_rate         NUMERIC(8,6) NOT NULL DEFAULT 0,
    spread                  BIGINT      NOT NULL DEFAULT 0,
    reposts                 BIGINT      NOT NULL DEFAULT 0,
    referrals               BIGINT      NOT NULL DEFAULT 0,
    -- audience
    followers               BIGINT      NOT NULL DEFAULT 0,
    subscribers             BIGINT      NOT NULL DEFAULT 0,
    watchers                BIGINT      NOT NULL DEFAULT 0,
    follower_growth_rate    NUMERIC(8,6) NOT NULL DEFAULT 0,
    subscriber_growth_rate  NUMERIC(8,6) NOT NULL DEFAULT 0,
    watcher_growth_rate     NUMERIC(8,6) NOT NULL DEFAULT 0,
    -- benchmarking
    benchmark_score         NUMERIC(8,4) NOT NULL DEFAULT 0,
    peer_percentile         NUMERIC(5,2),
    comparison_kpis         JSONB       NOT NULL DEFAULT '{}',
    -- period
    period_start            TIMESTAMPTZ,
    period_end              TIMESTAMPTZ,
    last_updated            TIMESTAMPTZ
);

-- Action history on a component (§8 ActionKind / ActivityLog)
CREATE TABLE IF NOT EXISTS portfolio.component_action_log (
    id              BIGSERIAL   PRIMARY KEY,
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    actor_id        UUID        NOT NULL REFERENCES auth.account(id),
    action_kind     TEXT        NOT NULL,
    action_params   JSONB       NOT NULL DEFAULT '{}',
    description     TEXT,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Portfolio-level event log — append-only (§18 EventLog / PortfolioEvent)
CREATE TABLE IF NOT EXISTS portfolio.event_log (
    id              BIGSERIAL   PRIMARY KEY,
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    event_id        TEXT        NOT NULL UNIQUE,
    sequence        BIGINT      NOT NULL,
    actor_id        TEXT        NOT NULL,
    event_kind      TEXT        NOT NULL,
    payload         JSONB       NOT NULL DEFAULT '{}',
    emitted_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- CRDT operation log (§20 CrdtLog)
CREATE TABLE IF NOT EXISTS portfolio.crdt_log (
    id              BIGSERIAL   PRIMARY KEY,
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    operation_kind  TEXT        NOT NULL,  -- set_field | add_to_set | remove_from_set | add_edge | remove_edge
    component_id    TEXT,
    field_name      TEXT,
    element         TEXT,
    tag             TEXT,
    edge_id         TEXT,
    value           TEXT,
    actor           TEXT        NOT NULL,
    clock_snapshot  JSONB       NOT NULL DEFAULT '{}',
    ts              BIGINT      NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Portfolio snapshot (§19 PortfolioSnapshot)
CREATE TABLE IF NOT EXISTS portfolio.snapshot (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    snapshot_id     TEXT        NOT NULL UNIQUE,
    label           TEXT,
    component_count INT         NOT NULL DEFAULT 0,
    edge_count      INT         NOT NULL DEFAULT 0,
    event_count     INT         NOT NULL DEFAULT 0,
    state_blob      JSONB       NOT NULL DEFAULT '{}',
    created_by      UUID        REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Portfolio checkpoint (§19 PortfolioCheckpoint)
CREATE TABLE IF NOT EXISTS portfolio.checkpoint (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    checkpoint_id   TEXT        NOT NULL UNIQUE,
    label           TEXT        NOT NULL,
    snapshot_id     TEXT        NOT NULL,
    note            TEXT,
    created_by      UUID        REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Governance: approval requests (§21 ApprovalRequest)
CREATE TABLE IF NOT EXISTS portfolio.approval_request (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    requested_by    UUID        NOT NULL REFERENCES auth.account(id),
    reason          TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'pending',  -- pending | approved | rejected | expired
    resolver_id     UUID        REFERENCES auth.account(id),
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at     TIMESTAMPTZ
);

-- Structural primitives: group / collection / list / schedule / directory

CREATE TABLE IF NOT EXISTS portfolio.group (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    group_type      TEXT        NOT NULL DEFAULT 'default',
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.group_member (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id        UUID        NOT NULL REFERENCES portfolio.group(id),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (group_id, component_id)
);

CREATE TABLE IF NOT EXISTS portfolio.collection (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.collection_item (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id   UUID        NOT NULL REFERENCES portfolio.collection(id),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (collection_id, component_id)
);

CREATE TABLE IF NOT EXISTS portfolio.list (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.list_item (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id         UUID        NOT NULL REFERENCES portfolio.list(id),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    position        INT         NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (list_id, component_id)
);

CREATE TABLE IF NOT EXISTS portfolio.schedule (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.schedule_entry (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id         UUID        NOT NULL REFERENCES portfolio.schedule(id),
    component_id        UUID        NOT NULL REFERENCES portfolio.component(id),
    scheduled_at        TIMESTAMPTZ NOT NULL,
    duration_minutes    INT,
    recurrence          TEXT,
    notes               TEXT,
    position            INT         NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.directory (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    parent_id       UUID        REFERENCES portfolio.directory(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    path            TEXT        NOT NULL,
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.directory_entry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    directory_id    UUID        NOT NULL REFERENCES portfolio.directory(id),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    entry_name      TEXT        NOT NULL,
    entry_type      TEXT        NOT NULL,
    path            TEXT        NOT NULL,
    position        INT         NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (directory_id, component_id)
);

-- Collaborators on a portfolio
CREATE TABLE IF NOT EXISTS portfolio.portfolio_collaborator (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    role            TEXT        NOT NULL,
    permissions     JSONB       NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (portfolio_id, account_id)
);

-- Generic portfolio_item (legacy compatibility layer)
CREATE TABLE IF NOT EXISTS portfolio.portfolio_item (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    item_type       TEXT        NOT NULL,
    title           TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'active',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.resource (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id         UUID        NOT NULL REFERENCES portfolio.portfolio_item(id),
    resource_type   TEXT        NOT NULL,
    value_amount    NUMERIC(16,2),
    currency        CHAR(3)     DEFAULT 'USD',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Milestones (§7 Milestone)
CREATE TABLE IF NOT EXISTS portfolio.milestone (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    due_date        TIMESTAMPTZ NOT NULL,
    completed_at    TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'active',
    linked_items    UUID[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Risk register (§7 Risk)
CREATE TABLE IF NOT EXISTS portfolio.risk (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    description     TEXT        NOT NULL,
    severity        TEXT        NOT NULL DEFAULT 'medium',  -- critical|high|medium|low|negligible
    probability     NUMERIC(4,3) NOT NULL DEFAULT 0.5,
    mitigation      TEXT,
    owner_id        UUID        REFERENCES auth.account(id),
    status          TEXT        NOT NULL DEFAULT 'open',    -- open|mitigated|accepted|closed
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Charter (§7 Charter)
CREATE TABLE IF NOT EXISTS portfolio.charter (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id        UUID        NOT NULL REFERENCES portfolio.component(id) UNIQUE,
    executive_summary   TEXT,
    objectives          TEXT[]      NOT NULL DEFAULT '{}',
    scope               TEXT,
    success_criteria    TEXT[]      NOT NULL DEFAULT '{}',
    constraints         TEXT[]      NOT NULL DEFAULT '{}',
    assumptions         TEXT[]      NOT NULL DEFAULT '{}',
    version             TEXT        NOT NULL DEFAULT '0.1.0',
    approved_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.charter_stakeholder (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    charter_id      UUID        NOT NULL REFERENCES portfolio.charter(id),
    user_id         UUID        NOT NULL REFERENCES auth.account(id),
    role            TEXT        NOT NULL DEFAULT 'stakeholder',
    approved        BOOLEAN     NOT NULL DEFAULT false,
    approved_at     TIMESTAMPTZ,
    UNIQUE (charter_id, user_id)
);

-- Metrics (§7 Metric)
CREATE TABLE IF NOT EXISTS portfolio.metric (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    name            TEXT        NOT NULL,
    metric_type     TEXT        NOT NULL DEFAULT 'kpi',  -- kpi|counter|gauge|histogram|rate|ratio
    value           NUMERIC(20,6) NOT NULL DEFAULT 0,
    unit            TEXT        NOT NULL DEFAULT '',
    period_start    TIMESTAMPTZ,
    period_end      TIMESTAMPTZ,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Version history (§7 VersionHistory)
CREATE TABLE IF NOT EXISTS portfolio.version_entry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    version         TEXT        NOT NULL,
    author_id       UUID        NOT NULL REFERENCES auth.account(id),
    message         TEXT        NOT NULL,
    diff_ref        TEXT,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- File references (§7 FileRef)
CREATE TABLE IF NOT EXISTS portfolio.file_ref (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    name            TEXT        NOT NULL,
    mime_type       TEXT        NOT NULL DEFAULT 'application/octet-stream',
    path            TEXT        NOT NULL,
    size_bytes      BIGINT      NOT NULL DEFAULT 0,
    checksum        TEXT,
    version         TEXT        NOT NULL DEFAULT '1.0.0',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    uploaded_by     UUID        NOT NULL REFERENCES auth.account(id),
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Document references (§7 DocumentRef)
CREATE TABLE IF NOT EXISTS portfolio.document_ref (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    title           TEXT        NOT NULL,
    doc_type        TEXT        NOT NULL DEFAULT 'markdown',
    content_ref     TEXT        NOT NULL,
    version         TEXT        NOT NULL DEFAULT '1.0.0',
    author_id       UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Catalogue entries (§7 CatalogueEntry)
CREATE TABLE IF NOT EXISTS portfolio.catalogue_entry (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id        UUID        NOT NULL REFERENCES portfolio.component(id),
    catalogue_owner_id  UUID        NOT NULL REFERENCES portfolio.component(id),
    tags                TEXT[]      NOT NULL DEFAULT '{}',
    searchable_text     TSVECTOR,
    metadata            JSONB       NOT NULL DEFAULT '{}',
    added_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Library assets (§7 LibraryAsset)
CREATE TABLE IF NOT EXISTS portfolio.library_asset (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id    UUID        NOT NULL REFERENCES portfolio.component(id),
    library_id      UUID        NOT NULL REFERENCES portfolio.component(id),
    name            TEXT        NOT NULL,
    asset_type      TEXT        NOT NULL DEFAULT 'template',
    content_ref     TEXT        NOT NULL,
    version         TEXT        NOT NULL DEFAULT '1.0.0',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Portfolio federation (§23 PortfolioFederation)
CREATE TABLE IF NOT EXISTS portfolio.federation (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT        NOT NULL,
    description     TEXT,
    created_by      UUID        NOT NULL REFERENCES auth.account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.federation_member (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    federation_id   UUID        NOT NULL REFERENCES portfolio.federation(id),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    peer_node_id    TEXT,
    sync_status     TEXT        NOT NULL DEFAULT 'synced',
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (federation_id, portfolio_id)
);

-- =============================================================================
--  WBS  (Work Breakdown Structure — Agile / Sprint board)
-- =============================================================================

CREATE TABLE IF NOT EXISTS wbs.wbs (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID        NOT NULL REFERENCES portfolio.portfolio(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    title           TEXT        NOT NULL,
    methodology     TEXT        NOT NULL DEFAULT 'agile',  -- agile | kanban | waterfall | scrum | custom
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS wbs.story (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wbs_id          UUID        NOT NULL REFERENCES wbs.wbs(id),
    parent_id       UUID        REFERENCES wbs.story(id),
    story_type      TEXT        NOT NULL,   -- feature|bug|task|enhancement|blocker|release|audit|strategy
    title           TEXT        NOT NULL,
    description     TEXT,
    priority        TEXT        NOT NULL DEFAULT 'normal',
    status          TEXT        NOT NULL DEFAULT 'backlog',
    estimate        NUMERIC(8,2),
    assignee_id     UUID        REFERENCES auth.account(id),
    sprint_id       UUID,
    due_at          TIMESTAMPTZ,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS wbs.sprint (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wbs_id          UUID        NOT NULL REFERENCES wbs.wbs(id),
    name            TEXT        NOT NULL,
    goal            TEXT,
    status          TEXT        NOT NULL DEFAULT 'planned',
    velocity        NUMERIC(8,2),
    starts_at       TIMESTAMPTZ,
    ends_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE wbs.story ADD CONSTRAINT fk_story_sprint
    FOREIGN KEY (sprint_id) REFERENCES wbs.sprint(id);

CREATE TABLE IF NOT EXISTS wbs.story_status_history (
    id              BIGSERIAL   PRIMARY KEY,
    story_id        UUID        NOT NULL REFERENCES wbs.story(id),
    from_status     TEXT,
    to_status       TEXT        NOT NULL,
    changed_by      UUID        REFERENCES auth.account(id),
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS wbs.release (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wbs_id          UUID        NOT NULL REFERENCES wbs.wbs(id),
    version         TEXT        NOT NULL,
    description     TEXT,
    release_date    TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'draft',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  MARKET  (Marketplace)
-- =============================================================================

CREATE TABLE IF NOT EXISTS market.listing (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id       UUID        NOT NULL REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    listing_type    TEXT        NOT NULL,  -- playbook|template|service|gig|job|contract|product|asset|resource
    title           TEXT        NOT NULL,
    description     TEXT,
    price           NUMERIC(16,2) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    hashtags        TEXT[]      NOT NULL DEFAULT '{}',
    status          TEXT        NOT NULL DEFAULT 'draft',
    visibility      TEXT        NOT NULL DEFAULT 'public',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.market_order (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id      UUID        NOT NULL REFERENCES market.listing(id),
    buyer_id        UUID        NOT NULL REFERENCES auth.account(id),
    seller_id       UUID        NOT NULL REFERENCES auth.account(id),
    amount          NUMERIC(16,2) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    status          TEXT        NOT NULL DEFAULT 'created',
    affiliate_link_id UUID,
    discount_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.review (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID        NOT NULL REFERENCES market.market_order(id),
    reviewer_id     UUID        NOT NULL REFERENCES auth.account(id),
    rating          SMALLINT    NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment         TEXT,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.campaign (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    listing_id      UUID        REFERENCES market.listing(id),
    title           TEXT        NOT NULL,
    campaign_type   TEXT        NOT NULL DEFAULT 'promote',  -- promote | fundraise | discount | bundle
    budget          NUMERIC(16,2),
    start_at        TIMESTAMPTZ,
    end_at          TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'draft',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Barter / trade offers
CREATE TABLE IF NOT EXISTS market.trade_offer (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    proposer_id     UUID        NOT NULL REFERENCES auth.account(id),
    recipient_id    UUID        REFERENCES auth.account(id),
    offered_items   JSONB       NOT NULL DEFAULT '[]',
    requested_items JSONB       NOT NULL DEFAULT '[]',
    notes           TEXT,
    status          TEXT        NOT NULL DEFAULT 'open',   -- open|accepted|rejected|countered|expired
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.subscription (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    subscriber_id   UUID        NOT NULL REFERENCES auth.account(id),
    publisher_id    UUID        NOT NULL REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    tier            TEXT        NOT NULL DEFAULT 'free',
    price           NUMERIC(16,2) NOT NULL DEFAULT 0,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    status          TEXT        NOT NULL DEFAULT 'active',
    renews_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (subscriber_id, publisher_id, component_id)
);

-- =============================================================================
--  EXCHANGE  (Financial + Asset Exchange)
-- =============================================================================

CREATE TABLE IF NOT EXISTS exchange.wallet (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    wallet_type     TEXT        NOT NULL,  -- personal | operations | investment | trading | marketplace
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    available       NUMERIC(20,4) NOT NULL DEFAULT 0,
    reserved        NUMERIC(20,4) NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, wallet_type, currency)
);

CREATE TABLE IF NOT EXISTS exchange.ledger_entry (
    id              BIGSERIAL   PRIMARY KEY,
    wallet_id       UUID        NOT NULL REFERENCES exchange.wallet(id),
    entry_kind      TEXT        NOT NULL,  -- credit | debit | reserve | release | fee | refund
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    reference_type  TEXT        NOT NULL,
    reference_id    TEXT        NOT NULL,
    balance_after   NUMERIC(20,4),
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Immutable ledger triggers
CREATE OR REPLACE FUNCTION exchange.reject_ledger_mutation()
RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'ledger_entry is immutable; use compensating entry';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_reject_ledger_update ON exchange.ledger_entry;
CREATE TRIGGER trg_reject_ledger_update
BEFORE UPDATE ON exchange.ledger_entry
FOR EACH ROW EXECUTE FUNCTION exchange.reject_ledger_mutation();

DROP TRIGGER IF EXISTS trg_reject_ledger_delete ON exchange.ledger_entry;
CREATE TRIGGER trg_reject_ledger_delete
BEFORE DELETE ON exchange.ledger_entry
FOR EACH ROW EXECUTE FUNCTION exchange.reject_ledger_mutation();

CREATE TABLE IF NOT EXISTS exchange.escrow (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID        NOT NULL REFERENCES market.market_order(id),
    payer_wallet_id UUID        NOT NULL REFERENCES exchange.wallet(id),
    payee_wallet_id UUID        NOT NULL REFERENCES exchange.wallet(id),
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    status          TEXT        NOT NULL DEFAULT 'funded',  -- funded|released|refunded|disputed
    release_deadline TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.bid (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    bidder_id       UUID        NOT NULL REFERENCES auth.account(id),
    listing_id      UUID        NOT NULL REFERENCES market.listing(id),
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    message         TEXT,
    status          TEXT        NOT NULL DEFAULT 'open',   -- open|accepted|rejected|countered|expired|withdrawn
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.offer (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    offerer_id      UUID        NOT NULL REFERENCES auth.account(id),
    recipient_id    UUID        REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    offer_type      TEXT        NOT NULL DEFAULT 'buy',   -- buy | sell | trade | invest | donate
    amount          NUMERIC(20,4),
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    terms           JSONB       NOT NULL DEFAULT '{}',
    message         TEXT,
    status          TEXT        NOT NULL DEFAULT 'open',
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.deal (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    bid_id          UUID        REFERENCES exchange.bid(id),
    offer_id        UUID        REFERENCES exchange.offer(id),
    buyer_id        UUID        NOT NULL REFERENCES auth.account(id),
    seller_id       UUID        NOT NULL REFERENCES auth.account(id),
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    terms           JSONB       NOT NULL DEFAULT '{}',
    due_diligence   JSONB       NOT NULL DEFAULT '{}',
    status          TEXT        NOT NULL DEFAULT 'negotiating',  -- negotiating|agreed|closed|cancelled
    closed_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.invoice (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    issuer_id       UUID        NOT NULL REFERENCES auth.account(id),
    recipient_id    UUID        NOT NULL REFERENCES auth.account(id),
    deal_id         UUID        REFERENCES exchange.deal(id),
    line_items      JSONB       NOT NULL DEFAULT '[]',
    subtotal        NUMERIC(20,4) NOT NULL,
    tax             NUMERIC(20,4) NOT NULL DEFAULT 0,
    total           NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    due_date        TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'draft',  -- draft|sent|paid|overdue|voided
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.payment (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id      UUID        REFERENCES exchange.invoice(id),
    payer_id        UUID        NOT NULL REFERENCES auth.account(id),
    payee_id        UUID        NOT NULL REFERENCES auth.account(id),
    wallet_id       UUID        REFERENCES exchange.wallet(id),
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    payment_method  TEXT        NOT NULL DEFAULT 'internal',
    external_ref    TEXT,
    status          TEXT        NOT NULL DEFAULT 'pending',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    paid_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.tax_record (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    tax_year        SMALLINT    NOT NULL,
    tax_type        TEXT        NOT NULL DEFAULT 'income',
    jurisdiction    TEXT        NOT NULL DEFAULT 'US',
    gross_income    NUMERIC(20,4) NOT NULL DEFAULT 0,
    deductions      NUMERIC(20,4) NOT NULL DEFAULT 0,
    tax_owed        NUMERIC(20,4) NOT NULL DEFAULT 0,
    tax_paid        NUMERIC(20,4) NOT NULL DEFAULT 0,
    status          TEXT        NOT NULL DEFAULT 'draft',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  FUND  (Fundraising / Capital Pools)
-- =============================================================================

CREATE TABLE IF NOT EXISTS fund.campaign (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    title           TEXT        NOT NULL,
    description     TEXT,
    campaign_type   TEXT        NOT NULL,  -- equity_crowdfund | donation | grant | presale | ico
    goal_amount     NUMERIC(20,4) NOT NULL,
    raised_amount   NUMERIC(20,4) NOT NULL DEFAULT 0,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    start_at        TIMESTAMPTZ,
    end_at          TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'draft',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS fund.contribution (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id     UUID        NOT NULL REFERENCES fund.campaign(id),
    contributor_id  UUID        NOT NULL REFERENCES auth.account(id),
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    contribution_type TEXT      NOT NULL DEFAULT 'donation',  -- donation|investment|grant|equity
    equity_pct      NUMERIC(7,4) NOT NULL DEFAULT 0,
    status          TEXT        NOT NULL DEFAULT 'confirmed',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS fund.capital_pool (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    pool_type       TEXT        NOT NULL,  -- operating | reserve | investment | mutual_aid
    name            TEXT        NOT NULL,
    balance         NUMERIC(20,4) NOT NULL DEFAULT 0,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    governance_model TEXT       NOT NULL DEFAULT 'owner',  -- owner | vote | multi_sig
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS fund.capital_allocation (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    pool_id         UUID        NOT NULL REFERENCES fund.capital_pool(id),
    recipient_id    UUID        NOT NULL REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    amount          NUMERIC(20,4) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    purpose         TEXT,
    status          TEXT        NOT NULL DEFAULT 'pending',
    approved_by     UUID        REFERENCES auth.account(id),
    approved_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  STUDIO
-- =============================================================================

CREATE TABLE IF NOT EXISTS studio.idea (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    title           TEXT        NOT NULL,
    summary         TEXT,
    stage           TEXT        NOT NULL DEFAULT 'idea',   -- idea|concept|prototype|design|mvp|launched
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    artifacts       JSONB       NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS studio.prototype (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id         UUID        NOT NULL REFERENCES studio.idea(id),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    prototype_type  TEXT        NOT NULL DEFAULT 'lo_fi',   -- lo_fi | hi_fi | functional | digital_twin
    status          TEXT        NOT NULL DEFAULT 'in_progress',
    artifacts       JSONB       NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS studio.design (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    title           TEXT        NOT NULL,
    design_type     TEXT        NOT NULL DEFAULT 'ui',  -- ui | architecture | blueprint | mockup | diagram
    tool            TEXT,
    file_ref        TEXT,
    version         TEXT        NOT NULL DEFAULT '0.1.0',
    status          TEXT        NOT NULL DEFAULT 'draft',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS studio.testbed (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    testbed_type    TEXT        NOT NULL DEFAULT 'sandbox',
    status          TEXT        NOT NULL DEFAULT 'active',
    config          JSONB       NOT NULL DEFAULT '{}',
    results         JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS studio.note (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    title           TEXT        NOT NULL,
    content         TEXT,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS studio.content_file (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    mime_type       TEXT        NOT NULL DEFAULT 'application/octet-stream',
    path            TEXT        NOT NULL,
    size_bytes      BIGINT      NOT NULL DEFAULT 0,
    checksum        TEXT,
    version         TEXT        NOT NULL DEFAULT '1.0',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS studio.tool_link (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    url             TEXT        NOT NULL,
    tool_type       TEXT        NOT NULL DEFAULT 'external',
    category        TEXT,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  COMMUNITY
-- =============================================================================

CREATE TABLE IF NOT EXISTS community.space (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    name            TEXT        NOT NULL,
    description     TEXT,
    space_type      TEXT        NOT NULL,  -- room | feed | channel | forum | group | chat
    visibility      TEXT        NOT NULL DEFAULT 'private',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    settings        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community.space_member (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id        UUID        NOT NULL REFERENCES community.space(id),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    role            TEXT        NOT NULL DEFAULT 'member',  -- owner | admin | moderator | member | guest
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (space_id, account_id)
);

CREATE TABLE IF NOT EXISTS community.post (
    id              BIGSERIAL   PRIMARY KEY,
    space_id        UUID        NOT NULL REFERENCES community.space(id),
    author_id       UUID        NOT NULL REFERENCES auth.account(id),
    parent_id       BIGINT      REFERENCES community.post(id),
    body            TEXT        NOT NULL,
    post_type       TEXT        NOT NULL DEFAULT 'post',   -- post | reply | announcement | poll
    visibility      TEXT        NOT NULL DEFAULT 'space',
    hashtags        TEXT[]      NOT NULL DEFAULT '{}',
    mentions        UUID[]      NOT NULL DEFAULT '{}',
    attachments     JSONB       NOT NULL DEFAULT '[]',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    edited_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community.message (
    id              BIGSERIAL   PRIMARY KEY,
    space_id        UUID        NOT NULL REFERENCES community.space(id),
    sender_id       UUID        NOT NULL REFERENCES auth.account(id),
    body            TEXT        NOT NULL,
    message_type    TEXT        NOT NULL DEFAULT 'chat',  -- chat | dm | broadcast | notification | alert
    thread_id       BIGINT      REFERENCES community.message(id),
    attachments     JSONB       NOT NULL DEFAULT '[]',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community.reaction (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id         BIGINT      REFERENCES community.post(id),
    message_id      BIGINT      REFERENCES community.message(id),
    actor_id        UUID        NOT NULL REFERENCES auth.account(id),
    emoji           TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (post_id, actor_id, emoji),
    UNIQUE (message_id, actor_id, emoji)
);

CREATE TABLE IF NOT EXISTS community.feed_item (
    id              BIGSERIAL   PRIMARY KEY,
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    event_type      TEXT        NOT NULL,  -- post | like | follow | campaign | marketplace | portfolio_update
    source_id       TEXT        NOT NULL,
    source_type     TEXT        NOT NULL,
    payload         JSONB       NOT NULL DEFAULT '{}',
    seen            BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community.notification (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    notification_type TEXT      NOT NULL,
    title           TEXT        NOT NULL,
    body            TEXT,
    source_type     TEXT,
    source_id       TEXT,
    read            BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  WORK  (Tasks / Gigs / Contracts / Time Entries)
-- =============================================================================

CREATE TABLE IF NOT EXISTS work.task (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    assignee_id     UUID        REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    title           TEXT        NOT NULL,
    description     TEXT,
    task_type       TEXT        NOT NULL DEFAULT 'task',
    priority        TEXT        NOT NULL DEFAULT 'normal',
    status          TEXT        NOT NULL DEFAULT 'todo',
    estimate_hours  NUMERIC(8,2),
    due_at          TIMESTAMPTZ,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.time_entry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    task_id         UUID        REFERENCES work.task(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    description     TEXT,
    hours           NUMERIC(8,2) NOT NULL,
    hourly_rate     NUMERIC(12,2),
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    started_at      TIMESTAMPTZ,
    ended_at        TIMESTAMPTZ,
    billable        BOOLEAN     NOT NULL DEFAULT true,
    invoiced        BOOLEAN     NOT NULL DEFAULT false,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.gig (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    listing_id      UUID        REFERENCES market.listing(id),
    title           TEXT        NOT NULL,
    description     TEXT,
    gig_type        TEXT        NOT NULL DEFAULT 'fixed',  -- fixed | hourly | retainer | equity
    price           NUMERIC(16,2) NOT NULL,
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    duration_days   INT,
    skills          TEXT[]      NOT NULL DEFAULT '{}',
    status          TEXT        NOT NULL DEFAULT 'open',   -- open|assigned|completed|cancelled
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.contract (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    party_a_id      UUID        NOT NULL REFERENCES auth.account(id),
    party_b_id      UUID        NOT NULL REFERENCES auth.account(id),
    gig_id          UUID        REFERENCES work.gig(id),
    deal_id         UUID        REFERENCES exchange.deal(id),
    title           TEXT        NOT NULL,
    terms           JSONB       NOT NULL DEFAULT '{}',
    value           NUMERIC(20,4),
    currency        CHAR(3)     NOT NULL DEFAULT 'USD',
    start_date      DATE,
    end_date        DATE,
    status          TEXT        NOT NULL DEFAULT 'draft',  -- draft|active|completed|terminated|disputed
    signed_at       TIMESTAMPTZ,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.okr (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    component_id    UUID        REFERENCES portfolio.component(id),
    title           TEXT        NOT NULL,
    objective       TEXT        NOT NULL,
    target_metric   TEXT        NOT NULL,
    current_value   NUMERIC(16,4) NOT NULL DEFAULT 0,
    target_value    NUMERIC(16,4) NOT NULL DEFAULT 1,
    period_start    DATE,
    period_end      DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.automation_rule (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    name            TEXT        NOT NULL,
    trigger_type    TEXT        NOT NULL,
    trigger_config  JSONB       NOT NULL,
    action_type     TEXT        NOT NULL,
    action_config   JSONB       NOT NULL,
    enabled         BOOLEAN     NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================================================
--  AO  (Autonomous Organizations / Cooperatives / Collectives / Federations)
-- =============================================================================

CREATE TABLE IF NOT EXISTS ao.autonomous_org (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES auth.account(id),
    portfolio_id    UUID        REFERENCES portfolio.portfolio(id),
    name            TEXT        NOT NULL,
    org_type        TEXT        NOT NULL DEFAULT 'collective',  -- collective|cooperative|autonomous_org|team|federation
    charter         JSONB       NOT NULL DEFAULT '{}',
    governance_model TEXT       NOT NULL DEFAULT 'flat',  -- flat|hierarchical|holacracy|dao
    status          TEXT        NOT NULL DEFAULT 'active',
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ao.org_member (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID        NOT NULL REFERENCES ao.autonomous_org(id),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    role            TEXT        NOT NULL DEFAULT 'member',
    equity_pct      NUMERIC(7,4) NOT NULL DEFAULT 0,
    voting_weight   NUMERIC(7,4) NOT NULL DEFAULT 1,
    status          TEXT        NOT NULL DEFAULT 'active',
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, account_id)
);

CREATE TABLE IF NOT EXISTS ao.proposal (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID        NOT NULL REFERENCES ao.autonomous_org(id),
    proposer_id     UUID        NOT NULL REFERENCES auth.account(id),
    title           TEXT        NOT NULL,
    body            TEXT        NOT NULL,
    proposal_type   TEXT        NOT NULL DEFAULT 'general',  -- general|budget|membership|policy|dissolution
    status          TEXT        NOT NULL DEFAULT 'draft',
    quorum_pct      NUMERIC(5,2) NOT NULL DEFAULT 51,
    pass_threshold  NUMERIC(5,2) NOT NULL DEFAULT 51,
    voting_ends_at  TIMESTAMPTZ,
    submitted_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ao.vote (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id     UUID        NOT NULL REFERENCES ao.proposal(id),
    voter_id        UUID        NOT NULL REFERENCES auth.account(id),
    choice          TEXT        NOT NULL,   -- yes | no | abstain
    weight          NUMERIC(12,4) NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (proposal_id, voter_id)
);

CREATE TABLE IF NOT EXISTS ao.governance_ledger (
    id              BIGSERIAL   PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES ao.autonomous_org(id),
    entry_type      TEXT        NOT NULL,
    ref_id          TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ao.org_federation (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_org_id   UUID        NOT NULL REFERENCES ao.autonomous_org(id),
    child_org_id    UUID        NOT NULL REFERENCES ao.autonomous_org(id),
    federation_type TEXT        NOT NULL DEFAULT 'member',   -- member | affiliate | subsidiary
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (parent_org_id, child_org_id)
);

-- =============================================================================
--  AI  (Agent Sessions / Action Logs / Recommendations / Search Index)
-- =============================================================================

CREATE TABLE IF NOT EXISTS ai.agent_session (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    model_name      TEXT        NOT NULL,
    context_window  JSONB       NOT NULL DEFAULT '{}',
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at        TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS ai.agent_action_log (
    id              BIGSERIAL   PRIMARY KEY,
    session_id      UUID        NOT NULL REFERENCES ai.agent_session(id),
    action_type     TEXT        NOT NULL,
    intent          TEXT        NOT NULL,
    input_payload   JSONB       NOT NULL,
    output_payload  JSONB       NOT NULL,
    requires_human_review BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai.recommendation (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    session_id      UUID        REFERENCES ai.agent_session(id),
    rec_type        TEXT        NOT NULL,  -- component|listing|user|resource|content|connection
    source_id       TEXT,
    recommended_id  TEXT        NOT NULL,
    recommended_type TEXT       NOT NULL,
    score           NUMERIC(6,5) NOT NULL DEFAULT 0,
    reason          TEXT,
    accepted        BOOLEAN,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai.persona_profile (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id) UNIQUE,
    interests       TEXT[]      NOT NULL DEFAULT '{}',
    behavior_tags   TEXT[]      NOT NULL DEFAULT '{}',
    preferences     JSONB       NOT NULL DEFAULT '{}',
    demographics    JSONB       NOT NULL DEFAULT '{}',
    sentiment_score NUMERIC(4,3) NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai.search_index (
    id              BIGSERIAL   PRIMARY KEY,
    source_type     TEXT        NOT NULL,   -- component | listing | community_post | user
    source_id       TEXT        NOT NULL,
    content_vector  TSVECTOR,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source_type, source_id)
);

-- =============================================================================
--  PROVIDER  (3rd-party integrations / affiliates)
-- =============================================================================

CREATE TABLE IF NOT EXISTS provider.registry (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT        NOT NULL UNIQUE,
    description     TEXT,
    provider_type   TEXT        NOT NULL DEFAULT 'integration',  -- integration | affiliate | payment | oauth | api
    category        TEXT        NOT NULL DEFAULT 'general',      -- finance | marketplace | community | productivity | ai | developer
    website_url     TEXT,
    logo_url        TEXT,
    docs_url        TEXT,
    api_base_url    TEXT,
    auth_type       TEXT        NOT NULL DEFAULT 'oauth2',      -- oauth2 | api_key | basic | jwt | none
    status          TEXT        NOT NULL DEFAULT 'active',
    version         TEXT        NOT NULL DEFAULT '1.0.0',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS provider.provider_resource (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id     UUID        NOT NULL REFERENCES provider.registry(id),
    resource_name   TEXT        NOT NULL,
    resource_type   TEXT        NOT NULL,  -- endpoint | webhook | file | data_stream | widget
    description     TEXT,
    endpoint        TEXT,
    schema          JSONB       NOT NULL DEFAULT '{}',
    version         TEXT        NOT NULL DEFAULT '1.0.0',
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS provider.account_connection (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    provider_id     UUID        NOT NULL REFERENCES provider.registry(id),
    external_ref    TEXT,
    access_token    TEXT,   -- encrypted
    refresh_token   TEXT,   -- encrypted
    token_expires_at TIMESTAMPTZ,
    scopes          TEXT[]  NOT NULL DEFAULT '{}',
    status          TEXT    NOT NULL DEFAULT 'active',
    metadata        JSONB   NOT NULL DEFAULT '{}',
    connected_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, provider_id)
);

-- Affiliate system
CREATE TABLE IF NOT EXISTS provider.affiliate (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id     UUID        NOT NULL REFERENCES provider.registry(id),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    affiliate_code  TEXT        NOT NULL UNIQUE,
    status          TEXT        NOT NULL DEFAULT 'active',
    commission_rate NUMERIC(6,4) NOT NULL DEFAULT 0.05,  -- e.g. 0.05 = 5%
    commission_type TEXT        NOT NULL DEFAULT 'percentage',  -- percentage | flat
    flat_commission NUMERIC(12,4),
    total_earned    NUMERIC(20,4) NOT NULL DEFAULT 0,
    payout_method   TEXT        NOT NULL DEFAULT 'wallet',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    registered_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS provider.affiliate_link (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    affiliate_id    UUID        NOT NULL REFERENCES provider.affiliate(id),
    provider_id     UUID        NOT NULL REFERENCES provider.registry(id),
    listing_id      UUID        REFERENCES market.listing(id),
    link_code       TEXT        NOT NULL UNIQUE,
    url             TEXT        NOT NULL,
    discount_type   TEXT        NOT NULL DEFAULT 'none',   -- none | percentage | flat
    discount_value  NUMERIC(12,4) NOT NULL DEFAULT 0,
    discount_code   TEXT,
    max_uses        INT,
    uses_count      INT         NOT NULL DEFAULT 0,
    expires_at      TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'active',
    metadata        JSONB       NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS provider.affiliate_commission (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    affiliate_id        UUID        NOT NULL REFERENCES provider.affiliate(id),
    affiliate_link_id   UUID        NOT NULL REFERENCES provider.affiliate_link(id),
    order_id            UUID        REFERENCES market.market_order(id),
    referred_account_id UUID        REFERENCES auth.account(id),
    sale_amount         NUMERIC(20,4) NOT NULL,
    commission_amount   NUMERIC(20,4) NOT NULL,
    currency            CHAR(3)     NOT NULL DEFAULT 'USD',
    status              TEXT        NOT NULL DEFAULT 'pending',  -- pending|approved|paid|rejected
    paid_at             TIMESTAMPTZ,
    metadata            JSONB       NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Prevent double-use of affiliate links (one per account per link)
CREATE TABLE IF NOT EXISTS provider.affiliate_link_usage (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    affiliate_link_id UUID      NOT NULL REFERENCES provider.affiliate_link(id),
    account_id      UUID        NOT NULL REFERENCES auth.account(id),
    used_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (affiliate_link_id, account_id)
);

-- =============================================================================
--  INDEXES
-- =============================================================================

-- kern
CREATE INDEX IF NOT EXISTS idx_kern_event_topic_time
    ON kern.event_log (topic, emitted_at DESC);
CREATE INDEX IF NOT EXISTS idx_kern_heartbeat_node
    ON kern.heartbeat_log (node_id, emitted_at DESC);

-- auth
CREATE INDEX IF NOT EXISTS idx_identity_account
    ON auth.identity (account_id);
CREATE INDEX IF NOT EXISTS idx_identity_profile_identity
    ON auth.identity_profile (identity_id, profile_type);
CREATE INDEX IF NOT EXISTS idx_connection_profile
    ON auth.connection_registry (profile_id, provider);
CREATE INDEX IF NOT EXISTS idx_contact_profile
    ON auth.contact_directory (profile_id, relationship_type);
CREATE INDEX IF NOT EXISTS idx_vault_profile
    ON auth.asset_vault (profile_id, asset_type);
CREATE INDEX IF NOT EXISTS idx_account_role_account
    ON auth.account_role (account_id, role_name);

-- portfolio
CREATE INDEX IF NOT EXISTS idx_portfolio_workspace
    ON portfolio.portfolio (workspace_id);
CREATE INDEX IF NOT EXISTS idx_component_portfolio
    ON portfolio.component (portfolio_id, category, status);
CREATE INDEX IF NOT EXISTS idx_component_tags
    ON portfolio.component USING gin(tags);
CREATE INDEX IF NOT EXISTS idx_component_hashtags
    ON portfolio.component USING gin(hashtags);
CREATE INDEX IF NOT EXISTS idx_graph_edge_from
    ON portfolio.graph_edge (from_id, kind);
CREATE INDEX IF NOT EXISTS idx_graph_edge_to
    ON portfolio.graph_edge (to_id, kind);
CREATE INDEX IF NOT EXISTS idx_event_log_portfolio
    ON portfolio.event_log (portfolio_id, sequence);
CREATE INDEX IF NOT EXISTS idx_component_action_log_component
    ON portfolio.component_action_log (component_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_metric_component
    ON portfolio.metric (component_id, metric_type);

-- wbs
CREATE INDEX IF NOT EXISTS idx_story_wbs_status
    ON wbs.story (wbs_id, status);
CREATE INDEX IF NOT EXISTS idx_story_assignee
    ON wbs.story (assignee_id, status);

-- market
CREATE INDEX IF NOT EXISTS idx_listing_seller_type
    ON market.listing (seller_id, listing_type, status);
CREATE INDEX IF NOT EXISTS idx_listing_tags
    ON market.listing USING gin(tags);
CREATE INDEX IF NOT EXISTS idx_order_buyer
    ON market.market_order (buyer_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_subscription_subscriber
    ON market.subscription (subscriber_id, status);

-- exchange
CREATE INDEX IF NOT EXISTS idx_wallet_account
    ON exchange.wallet (account_id);
CREATE INDEX IF NOT EXISTS idx_ledger_wallet_time
    ON exchange.ledger_entry (wallet_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_bid_listing
    ON exchange.bid (listing_id, status);
CREATE INDEX IF NOT EXISTS idx_offer_offerer
    ON exchange.offer (offerer_id, status);

-- community
CREATE INDEX IF NOT EXISTS idx_message_space_time
    ON community.message (space_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_post_space_time
    ON community.post (space_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_feed_account
    ON community.feed_item (account_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notification_account
    ON community.notification (account_id, read, created_at DESC);

-- work
CREATE INDEX IF NOT EXISTS idx_task_owner_status
    ON work.task (owner_id, status);
CREATE INDEX IF NOT EXISTS idx_time_entry_account
    ON work.time_entry (account_id, started_at DESC);

-- ao
CREATE INDEX IF NOT EXISTS idx_org_member_org
    ON ao.org_member (org_id, status);
CREATE INDEX IF NOT EXISTS idx_proposal_org
    ON ao.proposal (org_id, status);

-- ai
CREATE INDEX IF NOT EXISTS idx_recommendation_account
    ON ai.recommendation (account_id, rec_type);
CREATE INDEX IF NOT EXISTS idx_search_index_fts
    ON ai.search_index USING gin(content_vector);
CREATE INDEX IF NOT EXISTS idx_search_index_tags
    ON ai.search_index USING gin(tags);

-- provider
CREATE INDEX IF NOT EXISTS idx_affiliate_provider
    ON provider.affiliate (provider_id, status);
CREATE INDEX IF NOT EXISTS idx_affiliate_link_affiliate
    ON provider.affiliate_link (affiliate_id, status);
CREATE INDEX IF NOT EXISTS idx_commission_affiliate
    ON provider.affiliate_commission (affiliate_id, status, created_at DESC);
