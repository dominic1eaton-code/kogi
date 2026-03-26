# Kogi Office Backend

This backend provides the Office domain model and the WorkManagementSystem with the `kogi-portfolio` master spreadsheet as the canonical data source. The Rust library exposes FFI functions for the Go microservice, which then serves HTTP endpoints for the UI.

## Architecture
- **Source of truth**: `kogi-portfolio` master spreadsheet (projects `SHT-004`, tasks `SHT-005`, resources `SHT-006`, assets `SHT-007`, artifacts `SHT-008`).
- **Rust core**: `kogi-office` (models, portfolio-backed system, snapshots, FFI).
- **Go service**: `kogi-office/service` (HTTP endpoints for UI).

## WorkManagementSystem
The Office system materializes a WorkManagementSystem that aligns to the platform docs:
- Workspace with dashboard, backlog, governance, content, boards, timelines, analytics, resources, studio, and WBS.
- Work content library for files, documents, contracts, agreements, SOPs, policies, procedures, frameworks, and models.
- Work boards for agile, kanban, scrum, notes, pipeline, ideas/concepts/design, and custom boards.
- Work timelines for schedules, gantts, calendars, roadmaps, and timeboxes (PI, sprint, custom).
- Work analytics for forecasting, analysis, telemetry, optimization, personalization, performance, KPIs, OKRs, and data tracking.
- Work resource management for budgeting, reporting, allocation, delegation, and TODO buckets.
- Work studio for requirements management and design systems.
- WBS hierarchy for work package, theme, initiative, epic, story, and task.

## Portfolio Mapping
Office work items are derived from portfolio rows using:
- `item_type` or `container_type` (preferred)
- `tags` or `topics` (fallback)
- `ext` metadata keys: `story_type`, `description`, `attachments`

### Work Item Type Conventions
- `work_package`, `theme`, `initiative`, `epic`, `story`, `task`
- `portfolio`, `program`, `project`
- `resource`, `asset`, `artifact`

### Story Type Conventions
`story_type` maps to `StoryType` values such as: feature, bug, testing, capability, issue, defect, enhancement, innovation, audit, enabler, blocker, use case, business case, requirement, documentation, milestone, goal, objective, outcome, mission, vision, risk, analysis, strategy, tactic, operation, plan, report, release, deployment, distribution, template, archive, gig, job, contract, consultation, booking, meeting, appointment.

## HTTP Endpoints (Go Service)

### Core
- `GET /health`
- `POST /api/v1/office/init`
- `GET /api/v1/office/state`
- `POST /api/v1/office/refresh`

### Work Management
- `GET /api/v1/office/work/items`
- `GET /api/v1/office/work/management`

### Snapshots (UI-ready)
- `GET /api/v1/office/overview`
- `GET /api/v1/office/inbox`
- `GET /api/v1/office/schedule`
- `GET /api/v1/office/calendar`
- `GET /api/v1/office/contacts`

### Studio Snapshots
- `GET /api/v1/office/studio/overview`
- `GET /api/v1/office/studio/ideas`
- `GET /api/v1/office/studio/concepts`
- `GET /api/v1/office/studio/designs`
- `GET /api/v1/office/studio/blueprints`
- `GET /api/v1/office/studio/mockups`
- `GET /api/v1/office/studio/prototypes`
- `GET /api/v1/office/studio/testing`
- `GET /api/v1/office/studio/notes`
- `GET /api/v1/office/studio/docs`
- `GET /api/v1/office/studio/content`

### Work Snapshots
- `GET /api/v1/office/work/backlog`
- `GET /api/v1/office/work/boards`
- `GET /api/v1/office/work/timeline`
- `GET /api/v1/office/work/analytics`
- `GET /api/v1/office/work/resources`
- `GET /api/v1/office/work/content`
- `GET /api/v1/office/work/governance`

## Rust FFI Surface
The Go service calls into the Rust DLL/CLI using these function names:
- `office_init`
- `office_health`
- `office_state`
- `office_refresh`
- `office_work_items`
- `office_work_management`
- `office_overview_snapshot`
- `office_inbox_snapshot`
- `office_schedule_snapshot`
- `office_calendar_snapshot`
- `office_contacts_snapshot`
- `office_studio_overview_snapshot`
- `office_studio_ideas_snapshot`
- `office_studio_concepts_snapshot`
- `office_studio_designs_snapshot`
- `office_studio_blueprints_snapshot`
- `office_studio_mockups_snapshot`
- `office_studio_prototypes_snapshot`
- `office_studio_testing_snapshot`
- `office_studio_notes_snapshot`
- `office_studio_docs_snapshot`
- `office_studio_content_snapshot`
- `office_work_backlog_snapshot`
- `office_work_boards_snapshot`
- `office_work_timeline_snapshot`
- `office_work_analytics_snapshot`
- `office_work_resources_snapshot`
- `office_work_content_snapshot`
- `office_work_governance_snapshot`

## Notes
- The Office system treats `kogi-portfolio` as the authoritative registry for office work definitions.
- Work items synthesize ownership, dependencies, and scheduling from the portfolio row fields.
- Snapshot endpoints are UI-friendly starters and can be replaced by live analytics when needed.
