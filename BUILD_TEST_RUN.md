# Build, Test, and Run KOGI

This guide covers the standard local workflow for the KOGI application.

## Prerequisites

- Zig `0.15.2` or newer
- A terminal (PowerShell, cmd, bash, zsh, etc.)

Check your Zig version:

```bash
zig version
```

## Build

From the project root:

```bash
zig build
```

This builds and installs binaries into `zig-out/bin`.

## Run

Run the default app entrypoint through Zig:

```bash
zig build run
```

This launches the interactive KOGI shell (`kogi>` prompt).

Run the OS entrypoint explicitly:

```bash
zig build run-os
```

Pass runtime arguments:

```bash
zig build run -- <arg1> <arg2>
```

Run the compiled binary directly:

- Windows:

```powershell
.\zig-out\bin\kogi.exe
.\zig-out\bin\kogi-os.exe
```

- macOS/Linux:

```bash
./zig-out/bin/kogi
./zig-out/bin/kogi-os
```

## Interactive Shell Quickstart

Once the shell starts (`kogi>`), try:

```text
help
features
status
identity list
task list
database list
exit
```

## Test

Run all tests defined by the build graph:

```bash
zig build test
```

## Useful Commands

Show available build steps and options:

```bash
zig build --help
```

Fetch dependencies (if needed):

```bash
zig build --fetch
```

## Troubleshooting

- If `zig` is not found, add Zig to your `PATH`.
- If the Zig version is too old, install `0.15.2+`.
- If build cache issues occur, remove `.zig-cache` and retry:

```bash
# Windows PowerShell
Remove-Item -Recurse -Force .zig-cache

# macOS/Linux
rm -rf .zig-cache
```
