# Kogi Independent Worker Operating System (MVP Prototype)

Kogi is a polyglot operating platform for independent workers, cooperatives, and autonomous organizations.

This repository has been refactored into a multi-language monorepo aligned to the platform documents (`PRD`, `RFC`, `Requirements`, `LLSDS`, architecture docs).

## Platform Components

- `kogi-kernel` (Zig): low-level kernel orchestration, module registry, scheduler, cache/memory, process/file management, RBAC, mode barrier, FFI.
- `kogi-host` (Rust): central coordinator/orchestrator for kernel, modules, services, server, and engine with interactive shell CLI.
- `kogi-server` (Rust): backend server connecting kernel, host, and clients.
- `kogi-services/go` (Go): networking gateway, pub/sub, component communication, and microservice infrastructure.
- `kogi-engine` (Scala): platform data/data-processing engine (analytics, recommendations, discovery, exploration, streaming).
- `kogi-desktop-client` (Java): desktop client shell.
- `kogi-web-client` (Angular + TypeScript): web client shell.
- `kogi-mobile` (Kotlin Android + iOS scaffold): mobile/device client infrastructure.
- `kogi-database` (PostgreSQL): schema, immutable ledger, seed data.
- `kogi-modules` (service manifests): office, bank, exchange, marketplace, studio, community, developer, profile, organizations.
- `kogi-contracts` (OpenAPI + event topics): shared API/event contracts.

Existing Zig OS modules remain under `src/os` and are preserved as foundation code.

## Build System

- `WORKSPACE.bazel`, `MODULE.bazel`, and `BUILD.bazel` define the Bazel monorepo source graph.
- Component-level `BUILD.bazel` files expose source groups for each subsystem.
- `tools/build/build_all.ps1` documents local language-specific build commands.

## Quick Start

```powershell
# from repo root (`go.work` maps to `./kogi-services/go`)
# existing Zig OS demo
zig build run

# kernel MVP demo
cd kogi-kernel
zig build run

# host
cargo run --manifest-path kogi-host/Cargo.toml
# host one-shot mode (skip interactive shell)
cargo run --manifest-path kogi-host/Cargo.toml -- --once

# server
cargo run --manifest-path kogi-server/Cargo.toml

# go services
go run ./kogi-services/go/gateway
go run ./kogi-services/go/services/auth
go run ./kogi-services/go/services/portfolio
go run ./kogi-services/go/services/exchange
go run ./kogi-services/go/services/ims
go run ./kogi-services/go/services/office
# if a default port is occupied
$env:KOGI_GATEWAY_PORT = "18090"; go run ./kogi-services/go/gateway
# helper script alternative
.\tools\build\run_go_service.ps1 -Service gateway -Port 18090

# scala engine
cd kogi-engine
sbt compile
sbt "runMain kogi.engine.Main"

# desktop client
javac kogi-desktop-client/src/main/java/com/kogi/desktop/*.java
java -cp kogi-desktop-client/src/main/java com.kogi.desktop.Main

# web client
cd kogi-web-client
npm install
npm run start
```

## Module Domains (MVP)

- `kogi.office`: dashboard, portfolio, timeline, workspace, assistant, strategy, operations, governance.
- `kogi.bank`: wallet/account types, fundraising/capital/tax foundations.
- `kogi.exchange`: bids/offers/deals/escrow/trading and due diligence.
- `kogi.marketplace`: barter + items/skills/labor/resources marketplace flows.
- `kogi.studio`: ideas, prototypes, designs, testbeds, notes/content/files.
- `kogi.community`: spaces, chats, feeds, messaging, social context.
- `kogi.developer`: API/SDK, extensions, integration tooling.
- `kogi.profile`: personas, settings, preferences, user configuration.
- `kogi.organizations`: collectives/cooperatives/AO/team management.

## Key Design Constraints Implemented

- Kernel as central orchestrator (registry + event bus + scheduler + provisioner pattern).
- Event-driven cross-module integration contracts.
- Immutable financial ledger enforcement in PostgreSQL.
- Separation of kernel mode/user mode with RBAC checks at kernel boundary.
- Monorepo layout prepared for modular-monolith to microservices extraction.
- IMS supports multi-profile identities (personal/work/business/community) with profile-scoped accounts, settings, portfolios, integrations, tools, projects, and programs.
- Module manifests and kernel runtime enforce isolated memory/process/file/resource boundaries per module, network-managed by Go infrastructure.
- Unified v2/v3 screen-flow reconciliation exposed by server endpoints (`/api/v1/screens/unified`, `/api/v1/screens/unified/flat`) and consumed by both web and desktop clients.
