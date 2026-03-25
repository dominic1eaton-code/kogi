# Kogi Platform — Monorepo

**Independent Worker Operating System**

A multi-language monorepo built with Bazel, implementing the Kogi platform prototype.

---

## Architecture

```
kogi/
├── kernel/         Zig  — Core data structures, event bus, provisioner
├── api/            Go   — REST + gRPC API gateway (Gin)
├── analytics/      Scala — Analytics & reporting service (Akka HTTP)
├── desktop/        Java  — Desktop client (JavaFX)
├── web/            Angular — Web client (Angular Material)
└── proto/          Protocol Buffer definitions
```

## Stack

| Layer      | Language | Framework       | Port  |
|------------|----------|-----------------|-------|
| Kernel     | Zig 0.13 | stdlib          | N/A   |
| API        | Go 1.22  | Gin             | 8080  |
| Analytics  | Scala 2.13| Akka HTTP      | 8081  |
| Desktop    | Java 21  | JavaFX          | —     |
| Web        | TypeScript | Angular 17    | 4200  |
| Build      | —        | Bazel 7         | —     |

---

## Prerequisites

- [Bazel 7](https://bazel.build/install)
- [Zig 0.13](https://ziglang.org/download/)
- [Go 1.22](https://go.dev/dl/)
- [JDK 21](https://adoptium.net/)
- [Node.js 20](https://nodejs.org/)

---

## Build & Run

### Build everything
```bash
bazel build //...
```

### Run the Go API server
```bash
bazel run //api:server
# or directly:
cd api && go run main.go
```

### Run the Scala analytics server
```bash
bazel run //analytics:analytics_server
# or via sbt:
cd analytics && sbt run
```

### Run the JavaFX desktop client
```bash
bazel run //desktop:app
# or via Maven:
cd desktop && mvn javafx:run
```

### Run the Angular web client
```bash
cd web && npm install && npm start
# opens http://localhost:4200
```

### Run Zig kernel tests
```bash
cd kernel && zig build test
```

---

## Development Workflow

### Regenerate Go BUILD files (Gazelle)
```bash
bazel run //:gazelle
```

### Update Go dependencies
```bash
bazel run //:gazelle-update-repos
```

### Run all tests
```bash
bazel test //...
```

### Build in debug mode
```bash
bazel build --config=dev //...
```

### Build for production
```bash
bazel build --config=prod //...
```

---

## Environment Variables

| Variable              | Default                    | Description              |
|-----------------------|----------------------------|--------------------------|
| `KOGI_API_PORT`       | `8080`                     | Go API server port       |
| `KOGI_ANALYTICS_PORT` | `8081`                     | Scala analytics port     |
| `KOGI_ENV`            | (unset = debug)            | Set to `production`      |
| `KOGI_API_URL`        | `http://localhost:8080/api/v1` | Desktop client API URL |

---

## Module Overview

### Kernel (Zig)
- `portfolio.zig`   — Portfolio, PortfolioItem, Collection, Directory
- `wbs.zig`         — WBS, WorkPackage, Epic, Story, Task hierarchy
- `executor.zig`    — Executor types and Process lifecycle
- `event_bus.zig`   — Publish/subscribe event system
- `provisioner.zig` — User provisioning (Account + Workspace + Portfolio)
- `kernel.zig`      — Central orchestrator

### API (Go)
- REST endpoints for portfolios, items, WBS, stories, projects
- JWT authentication middleware
- SSE event stream endpoint
- Proxies to kernel FFI (planned)

### Analytics (Scala)
- Portfolio health scoring
- Story velocity metrics
- Project risk assessment
- Platform-level metrics aggregation

### Desktop (Java/JavaFX)
- Dark-themed JavaFX shell with nav sidebar
- Portfolio browser
- Story board (Kanban)
- Workspace tool launcher

### Web (Angular)
- Dark Material Design theme
- Dashboard with live stats
- Drag-and-drop Kanban story board
- Portfolio browser with item management
- Lazy-loaded feature modules
- JWT auth with interceptor

---

## Roadmap

- [ ] Zig → Go FFI bridge (kernel as shared library)
- [ ] gRPC service implementations
- [ ] PostgreSQL persistence layer (Go)
- [ ] Real-time WebSocket event stream
- [ ] Crowdfunding & capital pool module
- [ ] AO governance engine
- [ ] Desktop ↔ API integration complete
- [ ] Angular PWA support
