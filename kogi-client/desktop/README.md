# kogi-desktop-client

Java desktop client MVP.
The desktop UI now includes:
- `kogi.office` application views (dashboard, portfolio, timeline, workspace, assistant) backed by office endpoints.
- unified KOGI screen/workflow navigator (reconciled from v2/v3 screen-flow documents).

## Run
```powershell
# start server first
cargo run --manifest-path kogi-server/Cargo.toml

# in another shell
javac kogi-desktop-client/src/main/java/com/kogi/desktop/*.java
java -cp kogi-desktop-client/src/main/java com.kogi.desktop.Main
```
