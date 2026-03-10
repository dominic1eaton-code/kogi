-- Kogi MVP schema (SQLite)
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS auth_account (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_profile (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    handle TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    persona TEXT NOT NULL DEFAULT 'independent_worker',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_session (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    access_token_hash TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    ip_address TEXT,
    device_fingerprint TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_identity (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_identity_persona (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    identity_id TEXT NOT NULL REFERENCES auth_identity(id),
    persona_type TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (identity_id, persona_type)
);

CREATE TABLE IF NOT EXISTS auth_identity_role (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    identity_id TEXT NOT NULL REFERENCES auth_identity(id),
    role_name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (identity_id, role_name)
);

CREATE TABLE IF NOT EXISTS auth_identity_worker_type (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    identity_id TEXT NOT NULL REFERENCES auth_identity(id),
    worker_type TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (identity_id, worker_type)
);

CREATE TABLE IF NOT EXISTS auth_identity_profile (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    identity_id TEXT NOT NULL REFERENCES auth_identity(id),
    profile_type TEXT NOT NULL,
    name TEXT NOT NULL,
    settings TEXT NOT NULL DEFAULT '{}',
    options TEXT NOT NULL DEFAULT '{}',
    parameters TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_connection_registry (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    connection_type TEXT NOT NULL,
    provider TEXT NOT NULL,
    account_ref TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_contact_directory (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    contact_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    organization TEXT,
    relationship_type TEXT NOT NULL DEFAULT 'professional',
    status TEXT NOT NULL DEFAULT 'active',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_asset_vault (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    asset_type TEXT NOT NULL,
    name TEXT NOT NULL,
    value_amount REAL,
    currency TEXT DEFAULT 'USD',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_profile_tool (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    tool_name TEXT NOT NULL,
    tool_category TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_profile_program (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    program_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_profile_project (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    project_ref TEXT NOT NULL,
    source_module TEXT NOT NULL DEFAULT 'wbs',
    status TEXT NOT NULL DEFAULT 'active',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS kern_module_registry (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    module_id TEXT NOT NULL UNIQUE,
    module_kind TEXT NOT NULL,
    language TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    registered_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS kern_event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL,
    payload TEXT NOT NULL,
    emitted_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS kern_scheduler_job (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT,
    topic TEXT NOT NULL,
    cron_expr TEXT NOT NULL,
    payload_template TEXT NOT NULL,
    next_run_at TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS portfolio_workspace (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    workspace_type TEXT NOT NULL DEFAULT 'personal',
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS portfolio_portfolio (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    workspace_id TEXT NOT NULL REFERENCES portfolio_workspace(id),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'private',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS auth_profile_workspace_binding (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    workspace_id TEXT NOT NULL REFERENCES portfolio_workspace(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (profile_id, workspace_id)
);

CREATE TABLE IF NOT EXISTS auth_profile_portfolio_binding (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    profile_id TEXT NOT NULL REFERENCES auth_identity_profile(id),
    portfolio_id TEXT NOT NULL REFERENCES portfolio_portfolio(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (profile_id, portfolio_id)
);

CREATE TABLE IF NOT EXISTS portfolio_portfolio_item (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    portfolio_id TEXT NOT NULL REFERENCES portfolio_portfolio(id),
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS portfolio_resource (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    item_id TEXT NOT NULL REFERENCES portfolio_portfolio_item(id),
    resource_type TEXT NOT NULL,
    value_amount REAL,
    currency TEXT DEFAULT 'USD',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS portfolio_portfolio_collaborator (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    portfolio_id TEXT NOT NULL REFERENCES portfolio_portfolio(id),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    role TEXT NOT NULL,
    permissions TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (portfolio_id, account_id)
);

CREATE TABLE IF NOT EXISTS wbs_wbs (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    portfolio_id TEXT NOT NULL REFERENCES portfolio_portfolio(id),
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS wbs_story (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    wbs_id TEXT NOT NULL REFERENCES wbs_wbs(id),
    story_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT NOT NULL DEFAULT 'normal',
    status TEXT NOT NULL DEFAULT 'backlog',
    assignee_id TEXT REFERENCES auth_account(id),
    due_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS wbs_sprint (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    wbs_id TEXT NOT NULL REFERENCES wbs_wbs(id),
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'planned',
    starts_at TEXT,
    ends_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS wbs_story_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    story_id TEXT NOT NULL REFERENCES wbs_story(id),
    from_status TEXT,
    to_status TEXT NOT NULL,
    changed_by TEXT REFERENCES auth_account(id),
    changed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS market_listing (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    seller_id TEXT NOT NULL REFERENCES auth_account(id),
    listing_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    price REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS market_market_order (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    listing_id TEXT NOT NULL REFERENCES market_listing(id),
    buyer_id TEXT NOT NULL REFERENCES auth_account(id),
    seller_id TEXT NOT NULL REFERENCES auth_account(id),
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'created',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS market_review (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    order_id TEXT NOT NULL REFERENCES market_market_order(id),
    reviewer_id TEXT NOT NULL REFERENCES auth_account(id),
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS exchange_wallet (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    wallet_type TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    available REAL NOT NULL DEFAULT 0,
    reserved REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (account_id, wallet_type, currency)
);

CREATE TABLE IF NOT EXISTS exchange_ledger_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_id TEXT NOT NULL REFERENCES exchange_wallet(id),
    entry_kind TEXT NOT NULL,
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    reference_type TEXT NOT NULL,
    reference_id TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS exchange_escrow (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    order_id TEXT NOT NULL REFERENCES market_market_order(id),
    payer_wallet_id TEXT NOT NULL REFERENCES exchange_wallet(id),
    payee_wallet_id TEXT NOT NULL REFERENCES exchange_wallet(id),
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'funded',
    release_deadline TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS trg_reject_ledger_update
BEFORE UPDATE ON exchange_ledger_entry
BEGIN
    SELECT RAISE(ABORT, 'ledger_entry is immutable; use compensating entry');
END;

CREATE TRIGGER IF NOT EXISTS trg_reject_ledger_delete
BEFORE DELETE ON exchange_ledger_entry
BEGIN
    SELECT RAISE(ABORT, 'ledger_entry is immutable; use compensating entry');
END;

CREATE TABLE IF NOT EXISTS studio_idea (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    title TEXT NOT NULL,
    stage TEXT NOT NULL DEFAULT 'idea',
    summary TEXT,
    artifacts TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS community_space (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    name TEXT NOT NULL,
    space_type TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'private',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS community_message (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    space_id TEXT NOT NULL REFERENCES community_space(id),
    sender_id TEXT NOT NULL REFERENCES auth_account(id),
    body TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS work_okr (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    title TEXT NOT NULL,
    objective TEXT NOT NULL,
    target_metric TEXT NOT NULL,
    current_value REAL NOT NULL DEFAULT 0,
    target_value REAL NOT NULL DEFAULT 1,
    period_start TEXT,
    period_end TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS work_automation_rule (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    trigger_config TEXT NOT NULL,
    action_type TEXT NOT NULL,
    action_config TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fund_campaign (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    title TEXT NOT NULL,
    campaign_type TEXT NOT NULL,
    goal_amount REAL NOT NULL,
    raised_amount REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fund_capital_pool (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    pool_type TEXT NOT NULL,
    name TEXT NOT NULL,
    balance REAL NOT NULL DEFAULT 0,
    governance_model TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ao_autonomous_org (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES auth_account(id),
    name TEXT NOT NULL,
    charter TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ao_proposal (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    org_id TEXT NOT NULL REFERENCES ao_autonomous_org(id),
    proposer_id TEXT NOT NULL REFERENCES auth_account(id),
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    submitted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ao_vote (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    proposal_id TEXT NOT NULL REFERENCES ao_proposal(id),
    voter_id TEXT NOT NULL REFERENCES auth_account(id),
    choice TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (proposal_id, voter_id)
);

CREATE TABLE IF NOT EXISTS ao_governance_ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    org_id TEXT NOT NULL REFERENCES ao_autonomous_org(id),
    entry_type TEXT NOT NULL,
    ref_id TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ai_agent_session (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    account_id TEXT NOT NULL REFERENCES auth_account(id),
    model_name TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT
);

CREATE TABLE IF NOT EXISTS ai_agent_action_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES ai_agent_session(id),
    action_type TEXT NOT NULL,
    intent TEXT NOT NULL,
    input_payload TEXT NOT NULL,
    output_payload TEXT NOT NULL,
    requires_human_review INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_kern_event_topic_time
    ON kern_event_log (topic, emitted_at DESC);

CREATE INDEX IF NOT EXISTS idx_identity_account
    ON auth_identity (account_id);

CREATE INDEX IF NOT EXISTS idx_identity_profile_identity
    ON auth_identity_profile (identity_id, profile_type);

CREATE INDEX IF NOT EXISTS idx_connection_profile
    ON auth_connection_registry (profile_id, provider);

CREATE INDEX IF NOT EXISTS idx_contact_profile
    ON auth_contact_directory (profile_id, relationship_type);

CREATE INDEX IF NOT EXISTS idx_vault_profile
    ON auth_asset_vault (profile_id, asset_type);

CREATE INDEX IF NOT EXISTS idx_story_wbs_status
    ON wbs_story (wbs_id, status);

CREATE INDEX IF NOT EXISTS idx_message_space_time
    ON community_message (space_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_wallet_account
    ON exchange_wallet (account_id);

CREATE INDEX IF NOT EXISTS idx_ledger_wallet_time
    ON exchange_ledger_entry (wallet_id, created_at DESC);
