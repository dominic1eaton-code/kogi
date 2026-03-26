# Kogi Hub Backend

This backend provides the Hub domain model and governance + economics systems with the `kogi-portfolio` master spreadsheet as the canonical data source. The Rust library exposes FFI functions for the Go microservice, which then serves HTTP endpoints for the UI.

## Architecture
- **Source of truth**: `kogi-portfolio` master spreadsheet (organizations, teams, governance, contracts, and IP assets are all recorded as portfolio rows).
- **Rust core**: `kogi-hub` (models, portfolio-backed system, snapshots, FFI).
- **Go service**: `kogi-hub/service` (HTTP endpoints for UI).

## Hub Capabilities
- Teams, organizations, collectives, cooperatives, federations, autonomous cells, and DAOs.
- Voting, governance policies, proposals, and decentralized governance tracking.
- Allocation, distribution, restitution, and negotiation workflows.
- Contract, agreement, licensing, rights, and IP management.
- Independent worker member management and participation analytics.

## Portfolio Mapping
Hub resources are derived from portfolio rows using:
- `item_type` or `container_type` (preferred)
- `tags` or `topics` (fallback)
- `ext` metadata keys: `governance_model`, `rights`, `counterparty`, `quorum_pct`

### Entity Type Conventions
- `team`, `organization`, `collective`, `cooperative`, `federation`
- `autonomous`, `cell`, `dao`, `guild`, `consortium`, `network`
- `foundation`, `fund`, `studio`, `committee`, `council`, `circle`

### Contract and IP Conventions
- Contract types: `contract`, `agreement`, `license`, `licence`, `mou`, `nda`, `charter`
- Rights/IP tags: `rights`, `patent`, `copyright`, `trademark`, `watermark`, `branding`, `logo`, `mark`

## HTTP Endpoints (Go Service)

### Core
- `GET /health`
- `POST /api/v1/hub/init`
- `GET /api/v1/hub/state`
- `POST /api/v1/hub/refresh`

### Data
- `GET /api/v1/hub/entities`
- `GET /api/v1/hub/members`
- `GET /api/v1/hub/governance/data`
- `GET /api/v1/hub/voting/data`
- `GET /api/v1/hub/contracts`
- `GET /api/v1/hub/rights`
- `GET /api/v1/hub/ip`
- `GET /api/v1/hub/allocations`
- `GET /api/v1/hub/distributions`
- `GET /api/v1/hub/restitutions`
- `GET /api/v1/hub/negotiations/data`

### Snapshots (UI-ready)
- `GET /api/v1/hub/dashboard`
- `GET /api/v1/hub/governance`
- `GET /api/v1/hub/voting`
- `GET /api/v1/hub/allocation`
- `GET /api/v1/hub/distribution`
- `GET /api/v1/hub/collaboration`
- `GET /api/v1/hub/restitution`
- `GET /api/v1/hub/negotiations`
- `GET /api/v1/hub/teams`
- `GET /api/v1/hub/organizations`
- `GET /api/v1/hub/collectives`
- `GET /api/v1/hub/cooperatives`
- `GET /api/v1/hub/federations`
- `GET /api/v1/hub/autonomous`
- `GET /api/v1/hub/open-source`
- `GET /api/v1/hub/group-economics`
- `GET /api/v1/hub/resource-crowdfund`
- `GET /api/v1/hub/community-showcase`
- `GET /api/v1/hub/contracts/overview`
- `GET /api/v1/hub/ip/overview`

## Rust FFI Surface
The Go service calls into the Rust DLL/CLI using these function names:
- `hub_init`
- `hub_health`
- `hub_state`
- `hub_refresh`
- `hub_entities`
- `hub_members`
- `hub_governance`
- `hub_voting`
- `hub_contracts`
- `hub_rights`
- `hub_ip_assets`
- `hub_allocations`
- `hub_distributions`
- `hub_restitutions`
- `hub_negotiations`
- `hub_dashboard_snapshot`
- `hub_governance_snapshot`
- `hub_voting_snapshot`
- `hub_allocation_snapshot`
- `hub_distribution_snapshot`
- `hub_collaboration_snapshot`
- `hub_restitution_snapshot`
- `hub_negotiations_snapshot`
- `hub_teams_snapshot`
- `hub_organizations_snapshot`
- `hub_collectives_snapshot`
- `hub_cooperatives_snapshot`
- `hub_federations_snapshot`
- `hub_autonomous_snapshot`
- `hub_open_source_snapshot`
- `hub_group_economics_snapshot`
- `hub_resource_crowdfund_snapshot`
- `hub_community_showcase_snapshot`
- `hub_contracts_snapshot`
- `hub_ip_snapshot`

## Notes
- The Hub system treats `kogi-portfolio` as the authoritative registry for hub entities, governance, contracts, and IP assets.
- Snapshot endpoints are UI-friendly starters and can be replaced with live analytics as portfolio data grows.
