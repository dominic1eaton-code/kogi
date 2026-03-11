def _ensure_list(value):
    if type(value) == type([]):
        return value
    return [value]


def _ps_quote(value):
    return "'" + value.replace("'", "''") + "'"


def _ps_chain(commands):
    parts = []
    for cmd in commands:
        parts.append(cmd)
        parts.append("if ($$LASTEXITCODE -ne 0) { exit $$LASTEXITCODE }")
    return "; ".join(parts)


def _bash_chain(commands):
    return " && ".join(commands)


def _ps_array_literal(values):
    if not values:
        return "@()"
    return "@(" + ", ".join([_ps_quote(v) for v in values]) + ")"


def kogi_build(name, commands, srcs = [], workdir = "", required_tools = []):
    commands = _ensure_list(commands)
    ps_root = "$$execroot = Get-Location; $$out = Join-Path $$execroot \"$@\"; "
    ps_root += "$$paths = @(); "
    ps_root += "if ($$env:JAVA_HOME) { $$paths += (Join-Path $$env:JAVA_HOME \"bin\") }; "
    ps_root += "if ($$env:CARGO_HOME) { $$paths += (Join-Path $$env:CARGO_HOME \"bin\") }; "
    ps_root += "if ($$env:GOBIN) { $$paths += $$env:GOBIN }; "
    ps_root += "if ($$env:GOROOT) { $$paths += (Join-Path $$env:GOROOT \"bin\") }; "
    ps_root += "if ($$env:ZIG_HOME) { $$paths += $$env:ZIG_HOME }; "
    ps_root += "if ($$env:NVM_HOME) { $$paths += $$env:NVM_HOME }; "
    ps_root += "if ($$env:NVM_SYMLINK) { $$paths += $$env:NVM_SYMLINK }; "
    ps_root += "if ($$paths.Count -gt 0) { $$env:PATH = ($$paths -join ';') + ';' + $$env:PATH }; "
    ps_root += "function Resolve-Tool([string]$$name) { "
    ps_root += "  $$overrideVar = \"KOGI_BAZEL_\" + $$name.ToUpper(); "
    ps_root += "  $$override = [Environment]::GetEnvironmentVariable($$overrideVar); "
    ps_root += "  if ($$override -and (Test-Path $$override)) { return $$override }; "
    ps_root += "  $$cmd = Get-Command $$name -ErrorAction SilentlyContinue; "
    ps_root += "  if ($$cmd) { return $$cmd.Source }; "
    ps_root += "  $$candidates = @(); "
    ps_root += "  switch ($$name) { "
    ps_root += "    'cargo' { "
    ps_root += "      if ($$env:CARGO_HOME) { $$candidates += (Join-Path $$env:CARGO_HOME 'bin\\cargo.exe') }; "
    ps_root += "      if ($$env:USERPROFILE) { $$candidates += (Join-Path $$env:USERPROFILE '.cargo\\bin\\cargo.exe') }; "
    ps_root += "      if ($$env:USERPROFILE) { $$candidates += (Join-Path $$env:USERPROFILE 'scoop\\apps\\rustup\\current\\.cargo\\bin\\cargo.exe') }; "
    ps_root += "    } "
    ps_root += "    'go' { "
    ps_root += "      if ($$env:GOROOT) { $$candidates += (Join-Path $$env:GOROOT 'bin\\go.exe') }; "
    ps_root += "      if ($$env:GOBIN) { $$candidates += (Join-Path $$env:GOBIN 'go.exe') }; "
    ps_root += "      if ($$env:USERPROFILE) { $$candidates += (Join-Path $$env:USERPROFILE 'go\\bin\\go.exe') }; "
    ps_root += "      if ($$env:USERPROFILE) { $$candidates += (Join-Path $$env:USERPROFILE 'scoop\\apps\\go\\current\\bin\\go.exe') }; "
    ps_root += "      if ($$env:USERPROFILE) { $$candidates += (Join-Path $$env:USERPROFILE 'scoop\\apps\\golang\\current\\bin\\go.exe') }; "
    ps_root += "      $$candidates += 'C:\\\\Go\\\\bin\\\\go.exe'; "
    ps_root += "    } "
    ps_root += "    'zig' { "
    ps_root += "      if ($$env:ZIG_HOME) { $$candidates += (Join-Path $$env:ZIG_HOME 'zig.exe') }; "
    ps_root += "      if ($$env:USERPROFILE) { $$candidates += (Join-Path $$env:USERPROFILE 'scoop\\apps\\zig\\current\\zig.exe') }; "
    ps_root += "      $$candidates += 'C:\\\\zig\\\\zig.exe'; "
    ps_root += "      $$candidates += 'C:\\\\Zig\\\\zig.exe'; "
    ps_root += "      $$candidates += 'C:\\\\Program Files\\\\Zig\\\\zig.exe'; "
    ps_root += "      if (Test-Path 'C:\\\\global\\\\zig') { "
    ps_root += "        $$zigDirs = Get-ChildItem 'C:\\\\global\\\\zig' -Directory -ErrorAction SilentlyContinue; "
    ps_root += "        foreach ($$dir in $$zigDirs) { "
    ps_root += "          $$candidate = Join-Path $$dir.FullName 'zig.exe'; "
    ps_root += "          if (Test-Path $$candidate) { $$candidates += $$candidate } "
    ps_root += "          $$childDirs = Get-ChildItem $$dir.FullName -Directory -ErrorAction SilentlyContinue; "
    ps_root += "          foreach ($$child in $$childDirs) { "
    ps_root += "            $$childCandidate = Join-Path $$child.FullName 'zig.exe'; "
    ps_root += "            if (Test-Path $$childCandidate) { $$candidates += $$childCandidate } "
    ps_root += "          } "
    ps_root += "        } "
    ps_root += "      } "
    ps_root += "    } "
    ps_root += "    'javac' { "
    ps_root += "      if ($$env:JAVA_HOME) { $$candidates += (Join-Path $$env:JAVA_HOME 'bin\\javac.exe') }; "
    ps_root += "      if ($$env:ProgramFiles -and (Test-Path (Join-Path $$env:ProgramFiles 'Java'))) { "
    ps_root += "        $$jdkDirs = Get-ChildItem (Join-Path $$env:ProgramFiles 'Java') -Directory -ErrorAction SilentlyContinue | Where-Object { $$_.Name -like 'jdk*' } | Sort-Object Name -Descending; "
    ps_root += "        if ($$jdkDirs -and $$jdkDirs.Count -gt 0) { "
    ps_root += "          $$candidates += (Join-Path $$jdkDirs[0].FullName 'bin\\javac.exe') "
    ps_root += "        } "
    ps_root += "      } "
    ps_root += "    } "
    ps_root += "    'sbt' { "
    ps_root += "      $$candidates += 'C:\\\\Program Files (x86)\\\\sbt\\\\bin\\\\sbt.bat'; "
    ps_root += "      $$candidates += 'C:\\\\Program Files\\\\sbt\\\\bin\\\\sbt.bat'; "
    ps_root += "    } "
    ps_root += "    'npm' { "
    ps_root += "      $$candidates += 'C:\\\\Program Files\\\\nodejs\\\\npm.cmd'; "
    ps_root += "    } "
    ps_root += "    'gradle' { "
    ps_root += "      $$candidates += 'C:\\\\Program Files\\\\Gradle\\\\bin\\\\gradle.bat'; "
    ps_root += "    } "
    ps_root += "  } "
    ps_root += "  foreach ($$candidate in $$candidates) { if ($$candidate -and (Test-Path $$candidate)) { return $$candidate } }; "
    ps_root += "  return $$null; "
    ps_root += "} "
    ps_root += "function Ensure-Tool([string]$$name) { "
    ps_root += "  $$path = Resolve-Tool $$name; "
    ps_root += "  if (-not $$path) { "
    ps_root += "    Write-Error (\"Required tool '\" + $$name + \"' not found. Install it or set KOGI_BAZEL_\" + $$name.ToUpper() + \" to its full path.\"); "
    ps_root += "    exit 1; "
    ps_root += "  }; "
    ps_root += "  Set-Alias -Name $$name -Value $$path -Scope Global -Force; "
    ps_root += "} "
    ps_root += "$$required = %s; " % _ps_array_literal(required_tools)
    ps_root += "foreach ($$tool in $$required) { Ensure-Tool $$tool }; "
    ps_root += "$$root = Split-Path -Parent \"$(location //:WORKSPACE.bazel)\"; "
    if workdir:
        ps_root += "Set-Location (Join-Path $$root \"%s\"); " % workdir
    else:
        ps_root += "Set-Location $$root; "

    ps_cmd = (
        ps_root
        + "$$LASTEXITCODE = 0; "
        + _ps_chain(commands)
        + "; \"ok\" | Out-File -Encoding ascii $$out"
    )

    bash_root = "root=$(dirname $(location //:WORKSPACE.bazel)); "
    if workdir:
        bash_root += "cd \"$root/%s\"" % workdir
    else:
        bash_root += "cd \"$root\""

    bash_cmd = "set -e; %s; %s; echo ok > $@" % (
        bash_root,
        _bash_chain(commands),
    )

    native.genrule(
        name = name,
        srcs = srcs + ["//:WORKSPACE.bazel"],
        outs = [name + ".stamp"],
        cmd = bash_cmd,
        cmd_ps = ps_cmd,
    )


def kogi_run(name, command, srcs = [], workdir = ""):
    cmd_ps_command = command
    if workdir:
        cmd_ps_command = "cd /d %s && %s" % (workdir, command)

    ps_cmd = (
        "& \"$(location //tools/bazel:write_cmd.ps1)\" "
        + "-OutFile \"$@\" "
        + "-CommandLine %s" % _ps_quote(cmd_ps_command)
    )

    bash_lines = [
        "#!/usr/bin/env bash",
        "set -e",
        "if [ -n \"$BUILD_WORKSPACE_DIRECTORY\" ]; then cd \"$BUILD_WORKSPACE_DIRECTORY\"; fi",
    ]
    if workdir:
        bash_lines.append("cd \"%s\"" % workdir)
    bash_lines.append(command + " \"$@\"")

    bash_script = "\n".join(bash_lines)
    bash_cmd = "cat > $@ <<'EOF'\n%s\nEOF\nchmod +x $@" % bash_script

    native.genrule(
        name = name,
        srcs = srcs,
        tools = ["//tools/bazel:write_cmd.ps1"],
        outs = [name + ".cmd"],
        executable = True,
        cmd = bash_cmd,
        cmd_ps = ps_cmd,
    )
