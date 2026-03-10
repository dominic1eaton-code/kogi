-- Kogi MVP schema (PostgreSQL 16+)

CREATE EXTENSION IF NOT EXISTS pgcrypto;

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

CREATE TABLE IF NOT EXISTS auth.account (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    handle TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    persona TEXT NOT NULL DEFAULT 'independent_worker',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.session (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    access_token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address INET,
    device_fingerprint TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.identity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.identity_persona (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id UUID NOT NULL REFERENCES auth.identity(id),
    persona_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_id, persona_type)
);

CREATE TABLE IF NOT EXISTS auth.identity_role (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id UUID NOT NULL REFERENCES auth.identity(id),
    role_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_id, role_name)
);

CREATE TABLE IF NOT EXISTS auth.identity_worker_type (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id UUID NOT NULL REFERENCES auth.identity(id),
    worker_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_id, worker_type)
);

CREATE TABLE IF NOT EXISTS auth.identity_profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id UUID NOT NULL REFERENCES auth.identity(id),
    profile_type TEXT NOT NULL,
    name TEXT NOT NULL,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    options JSONB NOT NULL DEFAULT '{}'::jsonb,
    parameters JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.connection_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    connection_type TEXT NOT NULL,
    provider TEXT NOT NULL,
    account_ref TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.contact_directory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    contact_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    organization TEXT,
    relationship_type TEXT NOT NULL DEFAULT 'professional',
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.asset_vault (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    asset_type TEXT NOT NULL,
    name TEXT NOT NULL,
    value_amount NUMERIC(20,4),
    currency CHAR(3) DEFAULT 'USD',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_tool (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    tool_name TEXT NOT NULL,
    tool_category TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_program (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    program_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_project (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    project_ref TEXT NOT NULL,
    source_module TEXT NOT NULL DEFAULT 'wbs',
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.module_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id TEXT NOT NULL UNIQUE,
    module_kind TEXT NOT NULL,
    language TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.event_log (
    id BIGSERIAL PRIMARY KEY,
    topic TEXT NOT NULL,
    payload JSONB NOT NULL,
    emitted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS kern.scheduler_job (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID,
    topic TEXT NOT NULL,
    cron_expr TEXT NOT NULL,
    payload_template JSONB NOT NULL,
    next_run_at TIMESTAMPTZ NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.workspace (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    workspace_type TEXT NOT NULL DEFAULT 'personal',
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.portfolio (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES portfolio.workspace(id),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'private',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auth.profile_workspace_binding (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    workspace_id UUID NOT NULL REFERENCES portfolio.workspace(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (profile_id, workspace_id)
);

CREATE TABLE IF NOT EXISTS auth.profile_portfolio_binding (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES auth.identity_profile(id),
    portfolio_id UUID NOT NULL REFERENCES portfolio.portfolio(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (profile_id, portfolio_id)
);

CREATE TABLE IF NOT EXISTS portfolio.portfolio_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolio.portfolio(id),
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.resource (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES portfolio.portfolio_item(id),
    resource_type TEXT NOT NULL,
    value_amount NUMERIC(16,2),
    currency CHAR(3) DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio.portfolio_collaborator (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolio.portfolio(id),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    role TEXT NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (portfolio_id, account_id)
);

CREATE TABLE IF NOT EXISTS wbs.wbs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolio.portfolio(id),
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS wbs.story (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wbs_id UUID NOT NULL REFERENCES wbs.wbs(id),
    story_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT NOT NULL DEFAULT 'normal',
    status TEXT NOT NULL DEFAULT 'backlog',
    assignee_id UUID REFERENCES auth.account(id),
    due_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS wbs.sprint (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wbs_id UUID NOT NULL REFERENCES wbs.wbs(id),
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'planned',
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS wbs.story_status_history (
    id BIGSERIAL PRIMARY KEY,
    story_id UUID NOT NULL REFERENCES wbs.story(id),
    from_status TEXT,
    to_status TEXT NOT NULL,
    changed_by UUID REFERENCES auth.account(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.listing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID NOT NULL REFERENCES auth.account(id),
    listing_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    price NUMERIC(16,2) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.market_order (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES market.listing(id),
    buyer_id UUID NOT NULL REFERENCES auth.account(id),
    seller_id UUID NOT NULL REFERENCES auth.account(id),
    amount NUMERIC(16,2) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'created',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.review (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES market.market_order(id),
    reviewer_id UUID NOT NULL REFERENCES auth.account(id),
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.wallet (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    wallet_type TEXT NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    available NUMERIC(20,4) NOT NULL DEFAULT 0,
    reserved NUMERIC(20,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, wallet_type, currency)
);

CREATE TABLE IF NOT EXISTS exchange.ledger_entry (
    id BIGSERIAL PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES exchange.wallet(id),
    entry_kind TEXT NOT NULL,
    amount NUMERIC(20,4) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    reference_type TEXT NOT NULL,
    reference_id TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exchange.escrow (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES market.market_order(id),
    payer_wallet_id UUID NOT NULL REFERENCES exchange.wallet(id),
    payee_wallet_id UUID NOT NULL REFERENCES exchange.wallet(id),
    amount NUMERIC(20,4) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'funded',
    release_deadline TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION exchange.reject_ledger_mutation()
RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'ledger_entry is immutable; use compensating entry';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_reject_ledger_update ON exchange.ledger_entry;
CREATE TRIGGER trg_reject_ledger_update
BEFORE UPDATE ON exchange.ledger_entry
FOR EACH ROW
EXECUTE FUNCTION exchange.reject_ledger_mutation();

DROP TRIGGER IF EXISTS trg_reject_ledger_delete ON exchange.ledger_entry;
CREATE TRIGGER trg_reject_ledger_delete
BEFORE DELETE ON exchange.ledger_entry
FOR EACH ROW
EXECUTE FUNCTION exchange.reject_ledger_mutation();

CREATE TABLE IF NOT EXISTS studio.idea (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    title TEXT NOT NULL,
    stage TEXT NOT NULL DEFAULT 'idea',
    summary TEXT,
    artifacts JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community.space (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    name TEXT NOT NULL,
    space_type TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'private',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community.message (
    id BIGSERIAL PRIMARY KEY,
    space_id UUID NOT NULL REFERENCES community.space(id),
    sender_id UUID NOT NULL REFERENCES auth.account(id),
    body TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.okr (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    title TEXT NOT NULL,
    objective TEXT NOT NULL,
    target_metric TEXT NOT NULL,
    current_value NUMERIC(16,4) NOT NULL DEFAULT 0,
    target_value NUMERIC(16,4) NOT NULL DEFAULT 1,
    period_start DATE,
    period_end DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS work.automation_rule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    trigger_config JSONB NOT NULL,
    action_type TEXT NOT NULL,
    action_config JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS fund.campaign (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    title TEXT NOT NULL,
    campaign_type TEXT NOT NULL,
    goal_amount NUMERIC(20,4) NOT NULL,
    raised_amount NUMERIC(20,4) NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS fund.capital_pool (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    pool_type TEXT NOT NULL,
    name TEXT NOT NULL,
    balance NUMERIC(20,4) NOT NULL DEFAULT 0,
    governance_model TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ao.autonomous_org (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES auth.account(id),
    name TEXT NOT NULL,
    charter JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ao.proposal (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES ao.autonomous_org(id),
    proposer_id UUID NOT NULL REFERENCES auth.account(id),
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    submitted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ao.vote (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id UUID NOT NULL REFERENCES ao.proposal(id),
    voter_id UUID NOT NULL REFERENCES auth.account(id),
    choice TEXT NOT NULL,
    weight NUMERIC(12,4) NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (proposal_id, voter_id)
);

CREATE TABLE IF NOT EXISTS ao.governance_ledger (
    id BIGSERIAL PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES ao.autonomous_org(id),
    entry_type TEXT NOT NULL,
    ref_id TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai.agent_session (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES auth.account(id),
    model_name TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS ai.agent_action_log (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES ai.agent_session(id),
    action_type TEXT NOT NULL,
    intent TEXT NOT NULL,
    input_payload JSONB NOT NULL,
    output_payload JSONB NOT NULL,
    requires_human_review BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_kern_event_topic_time
    ON kern.event_log (topic, emitted_at DESC);

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

CREATE INDEX IF NOT EXISTS idx_story_wbs_status
    ON wbs.story (wbs_id, status);

CREATE INDEX IF NOT EXISTS idx_message_space_time
    ON community.message (space_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_wallet_account
    ON exchange.wallet (account_id);

CREATE INDEX IF NOT EXISTS idx_ledger_wallet_time
    ON exchange.ledger_entry (wallet_id, created_at DESC);
