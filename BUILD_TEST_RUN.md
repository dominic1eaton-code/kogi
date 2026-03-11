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

## Bazel Build + Run

Bazel wrappers call the native toolchains and let you build/run the platform from a single entrypoint.

Build everything:
```bash
bazel build //:build_all
```

Example runs:
```bash
bazel run //:kernel_run
bazel run //:server_run
bazel run //:gateway_run
bazel run //:engine_grpc_run
```

Pass extra args to run targets after `--`, for example:
```bash
bazel run //:engine_service_run -- --debug
```

If Bazel cannot find a toolchain, set an override env var to the full path (Windows examples):
```powershell
$env:KOGI_BAZEL_CARGO = "C:\\Users\\<you>\\.cargo\\bin\\cargo.exe"
$env:KOGI_BAZEL_GO = "C:\\Go\\bin\\go.exe"
$env:KOGI_BAZEL_ZIG = "C:\\path\\to\\zig.exe"
$env:KOGI_BAZEL_JAVAC = "C:\\Program Files\\Java\\jdk-22\\bin\\javac.exe"
```

```powershell

[System.Environment]::GetEnvironmentVariables([System.EnvironmentVariableTarget]::Process)

[System.Environment]::GetEnvironmentVariables([System.EnvironmentVariableTarget]::Machine)

[System.Environment]::GetEnvironmentVariables([System.EnvironmentVariableTarget]::User)

(Get-Command cargo).Source
(Get-Command go).Source
(Get-Command zig).Source
(Get-Command javac).Source

$env:KOGI_BAZEL_CARGO = "C:\\Users\\domin\\scoop\\shims\\cargo.exe"

$env:KOGI_BAZEL_GO = "C:\\Users\\domin\\scoop\\shims\\go.exe"

$env:KOGI_BAZEL_ZIG = "C:\\global\\zig\\0.15.0\\x86_64\\zig.exe"

$env:KOGI_BAZEL_JAVAC = "C:\\Program Files\\Common Files\\Oracle\\Java\\javapath\\javac.exe"

```

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
