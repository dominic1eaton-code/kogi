# Kogi Office

- Module ID: `kogi.office`
- Runtime language: `hybrid-rust-go`
- Host entrypoint: `kogi-services/go/services/office`
- Network manager: `kogi-go-network`

## Scope
- Dashboard for active projects/programs, attention items, notifications, direct messages, event feeds, personas/roles, and quick links.
- Portfolio for tiled/tree/modular grid inventory and focus view with binder/book/notebook/playbook/folder/file/version/metadata containers.
- Timeline for calendars, schedules, roadmaps, gantts, and personal timelines.
- Workspace for work/operations/tactics/strategy/governance with stories, work packages, CMS collections, and tools/toolchains.
- Assistant for digital AI chat context, discovery, recommendations, subscriptions, explore, and for-you cards.

## Integrations
`jira`, `monday`, `base44`, `claude`, `chatgpt`, `grok`, `openai`, `gitlab`, `github`

## Service Endpoints
- `GET /api/v1/office`
- `GET /api/v1/office/dashboard`
- `GET /api/v1/office/portfolio`
- `GET /api/v1/office/timeline`
- `GET /api/v1/office/workspace`
- `GET /api/v1/office/assistant`
