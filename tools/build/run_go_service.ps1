param(
    [Parameter(Mandatory = $false)]
    [ValidateSet('gateway','auth','portfolio','exchange','ims','office','bank','marketplace','studio','community','developer','profile','organizations','engine','database')]
    [string]$Service = 'gateway',
    [Parameter(Mandatory = $false)]
    [string]$Port,
    [Parameter(Mandatory = $false)]
    [switch]$Silent,
    [Parameter(Mandatory = $false)]
    [switch]$Debug
)

$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$env:GOWORK = Join-Path $root 'go.work'
$env:GOCACHE = Join-Path $root '.cache\go-build'
$env:GOMODCACHE = Join-Path $root '.cache\go-mod'

New-Item -ItemType Directory -Force -Path $env:GOCACHE | Out-Null
New-Item -ItemType Directory -Force -Path $env:GOMODCACHE | Out-Null

$target = switch ($Service) {
    'gateway'   { './kogi-network/gateway' }
    'auth'      { './kogi-network/services/auth' }
    'portfolio' { './kogi-network/services/portfolio' }
    'exchange'  { './kogi-network/services/exchange' }
    'ims'       { './kogi-network/services/ims' }
    'office'    { './kogi-network/services/office' }
    'bank'      { './kogi-network/services/bank' }
    'marketplace' { './kogi-network/services/marketplace' }
    'studio'    { './kogi-network/services/studio' }
    'community' { './kogi-network/services/community' }
    'developer' { './kogi-network/services/developer' }
    'profile'   { './kogi-network/services/profile' }
    'organizations' { './kogi-network/services/organizations' }
    'engine'    { './kogi-network/services/engine' }
    'database'  { './kogi-network/services/database' }
}

if ($Port) {
    $env:KOGI_PORT = $Port
}

$args = @()
if ($Silent) {
    $args += '--silent'
}
if ($Debug) {
    $args += '--debug'
}

Write-Host "Running $Service from $target"
go run $target @args
