# kogi-server

Rust backend platform server for Kogi MVP.

## Endpoints
- `GET /health`
- `GET /api/v1/system`
- `GET /api/v1/engine/system`
- `GET /api/v1/modules`
- `GET /api/v1/ims/identities`
- `GET /api/v1/ims/profiles`
- `GET /api/v1/autonomy/capabilities`
- `GET /api/v1/kernel/modules/isolation`
- `GET /api/v1/office`
- `GET /api/v1/office/dashboard`
- `GET /api/v1/office/portfolio`
- `GET /api/v1/office/timeline`
- `GET /api/v1/office/workspace`
- `GET /api/v1/office/assistant`
- `GET /api/v1/screens/unified`
- `GET /api/v1/screens/unified/flat`

## Run
```powershell
cargo run --manifest-path kogi-server/Cargo.toml
```
