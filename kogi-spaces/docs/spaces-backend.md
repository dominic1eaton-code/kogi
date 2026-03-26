# Kogi Spaces Backend

This backend provides the Spaces domain model (spaces, rooms, channels, events, feeds, timelines, and network linking) with the `kogi-portfolio` master spreadsheet as the canonical data source. The Rust library exposes FFI functions for the Go microservice, which then serves HTTP endpoints for the UI.

## Architecture
- **Source of truth**: `kogi-portfolio` master spreadsheet (sheet `SHT-024` Community).
- **Rust core**: `kogi-spaces` (models, portfolio-backed system, snapshots, FFI).
- **Go service**: `kogi-spaces/service` (HTTP endpoints for UI).

## Portfolio Mapping
Spaces and related resources are derived from portfolio rows using:
- `item_type` or `container_type` (preferred)
- `tags` or `topics` (fallback)
- `ext` metadata keys: `space`, `spaces`, `kogi_space`, `space_profile`

### Type Conventions
- Space types: `space`, `community`, `team`, `organization`, `guild`, `collective`, `hub`
- Room types: `room`, `chat`, `voice`, `video`, `stage`
- Channel types: `channel`, `feed`, `timeline`, `broadcast`, `announcements`
- Event types: `event`, `session`, `workshop`, `webinar`
- Workspace types: `workspace`

## HTTP Endpoints (Go Service)

### Core
- `GET /health`
- `POST /api/v1/spaces/init`
- `GET /api/v1/spaces/state`
- `POST /api/v1/spaces/refresh`

### Spaces
- `GET /api/v1/spaces`
- `GET /api/v1/spaces/{space_id}`
- `GET /api/v1/spaces/owner/{owner_id}`
- `GET /api/v1/spaces/{space_id}/members`
- `POST /api/v1/spaces/publish`

### Resources
- `GET /api/v1/spaces/rooms`
- `GET /api/v1/spaces/channels`
- `GET /api/v1/spaces/events`
- `GET /api/v1/spaces/workspaces`

### Snapshots (UI-ready)
- `GET /api/v1/spaces/dashboard`
- `GET /api/v1/spaces/feed`
- `GET /api/v1/spaces/timeline`
- `GET /api/v1/spaces/rooms/overview`
- `GET /api/v1/spaces/channels/overview`
- `GET /api/v1/spaces/events/overview`
- `GET /api/v1/spaces/network/overview`
- `GET /api/v1/spaces/network/linknet`
- `GET /api/v1/spaces/network/linktree`
- `GET /api/v1/spaces/network/linkforest`

## Rust FFI Surface
The Go service calls into the Rust DLL/CLI using these function names:
- `spaces_init`
- `spaces_health`
- `spaces_state`
- `spaces_refresh`
- `spaces_spaces`
- `spaces_space`
- `spaces_spaces_by_owner`
- `spaces_members`
- `spaces_rooms`
- `spaces_channels`
- `spaces_events`
- `spaces_workspaces`
- `spaces_publish_space`
- `spaces_dashboard_snapshot`
- `spaces_feed_snapshot`
- `spaces_timeline_snapshot`
- `spaces_rooms_snapshot`
- `spaces_channels_snapshot`
- `spaces_events_snapshot`
- `spaces_network_overview_snapshot`
- `spaces_network_linknet_snapshot`
- `spaces_network_linktree_snapshot`
- `spaces_network_linkforest_snapshot`

## Notes
- The Spaces system treats `kogi-portfolio` as the authoritative registry for space definitions.
- Members are synthesized from portfolio ownership and collaboration fields (owners, editors, contributors, viewers, watchers).
- Network endpoints provide LinkNet/LinkTree/LinkForest snapshots aligned to the platform docs.
