# Kogi Independent Worker Operating System (MVP Prototype)

Kogi is a polyglot operating platform for independent workers, cooperatives, and autonomous organizations.

This repository has been refactored into a multi-language monorepo aligned to the platform documents (`PRD`, `RFC`, `Requirements`, `LLSDS`, architecture docs).

## Platform Components

- `kogi-kernel` (Zig): low-level kernel orchestration, module registry, scheduler, cache/memory, process/file management, RBAC, mode barrier, FFI.
- `kogi-host` (Rust): host runtime and system application (HostApp/HostModel/HostSystem) that boots the kernel + modules and orchestrates gateway + module services + engine/database services with an interactive shell.
- `kogi-server` (Rust): backend server that boots `kogi-host` (HostApp), provides client-facing access, and bridges client messages to the gateway + host.
- `kogi-network` (Go): networking gateway, pub/sub, component communication, plus module, engine, and database service facades.
- `kogi-engine` (Scala): platform data/data-processing engine (analytics, telemetry, recommendations/discovery/explore, search, query optimization).
- `kogi-client/desktop` (Java): desktop client shell.
- `kogi-client/web` (Angular + TypeScript): web client shell.
- `kogi-client/mobile` (Kotlin Android + iOS scaffold): mobile/device client infrastructure.
- `kogi-database` (PostgreSQL): schema, immutable ledger, seed data.
- `kogi-modules` (service manifests + Rust systems): office, bank, exchange, marketplace, studio, community, developer, profile, organizations, database.
- `kogi-infra` (OpenAPI + gRPC + event topics): shared API/event contracts.

Existing Zig OS modules remain under `src/os` and are preserved as foundation code.

## Build System

- `WORKSPACE.bazel`, `MODULE.bazel`, and `BUILD.bazel` define the Bazel monorepo source graph.
- Component-level `BUILD.bazel` files expose source groups for each subsystem.
- `tools/build/build_all.ps1` documents local language-specific build commands.

## Bazel Build + Run

Bazel targets wrap the native toolchains (zig/cargo/go/sbt/npm/gradle) so every component can build/run via `bazel build` and `bazel run`.

Build everything:
```powershell
bazel build //:build_all
```

Common build targets:
- `bazel build //:kernel_build`
- `bazel build //:host_build`
- `bazel build //:server_build`
- `bazel build //:services_build`
- `bazel build //:engine_build`
- `bazel build //:modules_build`
- `bazel build //:database_build`
- `bazel build //:infra_build`
- `bazel build //:desktop_build`
- `bazel build //:web_build`
- `bazel build //:mobile_android_build`

Run targets:
- `bazel run //:kernel_run`
- `bazel run //:host_run`
- `bazel run //:server_run`
- `bazel run //:gateway_run`
- `bazel run //:engine_service_run`
- `bazel run //:engine_run`
- `bazel run //:engine_grpc_run`
- `bazel run //:desktop_run`
- `bazel run //:web_run`
- `bazel run //:mobile_android_run`

Go service run targets are available for each service (`auth_run`, `portfolio_run`, `exchange_run`, `ims_run`, `office_run`, `bank_run`, `marketplace_run`, `studio_run`, `community_run`, `developer_run`, `profile_run`, `organizations_run`, `engine_service_run`, `database_service_run`).

All run targets accept extra args after `--`, for example:
```powershell
bazel run //:engine_service_run -- --debug
bazel run //:gateway_run -- --silent
```

Notes:
- `mobile_ios_build` is a placeholder that prints a macOS/Xcode requirement.
- Ensure the native toolchains are installed and on `PATH`.
- Bazel wrappers are tuned for Windows PowerShell/`cmd`. On macOS/Linux, use the native toolchains or adapt the Bazel commands as needed.
- If Bazel cannot locate a tool, set an override env var to the full path (e.g., `KOGI_BAZEL_CARGO`, `KOGI_BAZEL_GO`, `KOGI_BAZEL_ZIG`, `KOGI_BAZEL_JAVAC`, `KOGI_BAZEL_SBT`, `KOGI_BAZEL_NPM`, `KOGI_BAZEL_GRADLE`).

## Quick Start

```powershell
# from repo root (`go.work` maps to `./kogi-network`)
# existing Zig OS demo
zig build run

# kernel MVP demo
cd kogi-kernel
zig build run

# host (standalone CLI)
cargo run --manifest-path kogi-host/Cargo.toml
# host one-shot mode (skip interactive shell)
cargo run --manifest-path kogi-host/Cargo.toml -- --once

# server (boots kogi-host for clients)
cargo run --manifest-path kogi-server/Cargo.toml

# go services
go run ./kogi-network/gateway
go run ./kogi-network/services/auth
go run ./kogi-network/services/portfolio
go run ./kogi-network/services/exchange
go run ./kogi-network/services/ims
go run ./kogi-network/services/office
go run ./kogi-network/services/bank
go run ./kogi-network/services/marketplace
go run ./kogi-network/services/studio
go run ./kogi-network/services/community
go run ./kogi-network/services/developer
go run ./kogi-network/services/profile
go run ./kogi-network/services/organizations
go run ./kogi-network/services/engine
go run ./kogi-network/services/database
# run in silent/debug modes
go run ./kogi-network/gateway --silent
go run ./kogi-network/services/engine --debug
# database system (Rust module invoked by the Go database service)
cargo run --manifest-path kogi-modules/database/Cargo.toml -- --request "{\"action\":\"status\"}"
# if a default port is occupied
$env:KOGI_GATEWAY_PORT = "18090"; go run ./kogi-network/gateway
# helper script alternative
.\tools\build\run_go_service.ps1 -Service gateway -Port 18090
# background manager (gateway + services + kogi-server)
.\tools\build\kogi_daemon.ps1 -Action start

# scala engine
cd kogi-engine
sbt compile
sbt "runMain kogi.engine.Main"
# gRPC engine server
sbt "runMain kogi.engine.EngineGrpcServer"

# desktop client
javac -d kogi-client/desktop/target/classes kogi-client/desktop/src/main/java/com/kogi/desktop/*.java
java -cp kogi-client/desktop/target/classes com.kogi.desktop.Main

# web client
cd kogi-client/web
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
- `kogi.database`: PostgreSQL management, CRUD/query access, snapshots/backups/restores, scaling, optimization.

## Key Design Constraints Implemented

- Kernel as central orchestrator (registry + event bus + scheduler + provisioner pattern).
- Event-driven cross-module integration contracts.
- Immutable financial ledger enforcement in PostgreSQL.
- Separation of kernel mode/user mode with RBAC checks at kernel boundary.
- Monorepo layout prepared for modular-monolith to microservices extraction.
- IMS supports multi-profile identities (personal/work/business/community) with profile-scoped accounts, settings, portfolios, integrations, tools, projects, and programs.
- Module manifests and kernel runtime enforce isolated memory/process/file/resource boundaries per module, network-managed by Go infrastructure.
- Unified v2/v3 screen-flow reconciliation exposed by server endpoints (`/api/v1/screens/unified`, `/api/v1/screens/unified/flat`) and consumed by both web and desktop clients.


## notes/todo
- host telemetry service (optional; server now exposes host access)
- session management system
- authentication, authorization, RBAC+access control, privileges+persmissions+certificates+keys+tokens, security+privacy+protection
