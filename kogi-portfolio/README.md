# Kogi Portfolio

The `kogi-portfolio` crate implements the master portfolio spreadsheet runtime, snapshots, and API-friendly orchestration via `PortfolioSystem`. It powers dashboard, analytics, registry, and cross-grid link views, and exposes an optional C ABI for Go services.

**Key Modules**
- `src/runtime.rs` - Hypergrid-backed portfolio runtime + workbook sync.
- `src/spreadsheet.rs` - Spreadsheet schema, computed columns, scoring.
- `src/ui.rs` - Snapshot builders for UI and dashboards.
- `src/model.rs` - `PortfolioSystem` orchestration layer.
- `src/ffi.rs` - C ABI bridge for Go services (`kogi_portfolio` DLL).

**Build (Rust)**
```bash
cd kogi-portfolio
cargo build
```

**Build DLL (Rust cdylib)**
```bash
cd kogi-portfolio
cargo build --release
```

The build emits `kogi_portfolio.dll` (Windows) in `kogi-portfolio/target/release`.

**Score Preference**
Scoring can prefer policy-driven or AI-driven inputs, or blend both:
- `policy_first` (default)
- `ai_first`
- `blend` (configurable weight)

You can set this via the init API (see below) or by updating `KogiPortfolioConfig.score_preference`.

**Service API (Curl)**
Run the Go service first (see `kogi-portfolio/service` below). Examples:

```bash
# Health
curl http://localhost:9012/health

# Initialize with custom owner + scoring preference
curl -X POST http://localhost:9012/api/v1/portfolio/init ^
  -H "Content-Type: application/json" ^
  -d "{\"owner_id\":\"00000000-0000-0000-0000-000000000001\",\"score_preference\":\"policy\"}"

# State
curl http://localhost:9012/api/v1/portfolio/state

# Items view
curl http://localhost:9012/api/v1/portfolio/items

# Dashboard snapshot
curl http://localhost:9012/api/v1/portfolio/dashboard

# Analytics snapshot
curl http://localhost:9012/api/v1/portfolio/analytics

# Registry snapshot
curl http://localhost:9012/api/v1/portfolio/registry

# Link forest view (optional root)
curl "http://localhost:9012/api/v1/portfolio/link-forest?root_component_id=00000000-0000-0000-0000-000000000002"

# Link forest rows
curl "http://localhost:9012/api/v1/portfolio/link-forest/rows?root_component_id=00000000-0000-0000-0000-000000000002"

# Link forest sheet
curl "http://localhost:9012/api/v1/portfolio/link-forest/sheet?root_component_id=00000000-0000-0000-0000-000000000002"
```
