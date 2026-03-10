param(
    [Parameter(Mandatory = $false)]
    [ValidateSet('gateway','auth','portfolio','exchange','ims','office')]
    [string]$Service = 'gateway',
    [Parameter(Mandatory = $false)]
    [string]$Port
)

$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$env:GOWORK = Join-Path $root 'go.work'
$env:GOCACHE = Join-Path $root '.cache\go-build'
$env:GOMODCACHE = Join-Path $root '.cache\go-mod'

New-Item -ItemType Directory -Force -Path $env:GOCACHE | Out-Null
New-Item -ItemType Directory -Force -Path $env:GOMODCACHE | Out-Null

$target = switch ($Service) {
    'gateway'   { './kogi-services/go/gateway' }
    'auth'      { './kogi-services/go/services/auth' }
    'portfolio' { './kogi-services/go/services/portfolio' }
    'exchange'  { './kogi-services/go/services/exchange' }
    'ims'       { './kogi-services/go/services/ims' }
    'office'    { './kogi-services/go/services/office' }
}

if ($Port) {
    $env:KOGI_PORT = $Port
}

Write-Host "Running $Service from $target"
go run $target
