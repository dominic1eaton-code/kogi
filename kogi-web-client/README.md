# kogi-web-client

Angular + TypeScript web client MVP.
The web client includes:
- `kogi.office` application views: dashboard, portfolio, timeline, workspace, assistant.
- unified KOGI module/workflow screen series, profile switching, and live catalog sync from `kogi-server`.

## Run
```powershell
# start server first
cargo run --manifest-path kogi-server/Cargo.toml

# in another shell
cd kogi-web-client
npm install
npm run start
```
