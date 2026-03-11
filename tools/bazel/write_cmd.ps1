param(
    [Parameter(Mandatory = $true)]
    [string]$OutFile,
    [Parameter(Mandatory = $true)]
    [string]$CommandLine
)

$lines = @(
    "@echo off",
    "setlocal",
    "if not \"%BUILD_WORKSPACE_DIRECTORY%\"==\"\" (cd /d \"%BUILD_WORKSPACE_DIRECTORY%\")",
    "$CommandLine %*"
)

Set-Content -Path $OutFile -Value $lines -Encoding Ascii
