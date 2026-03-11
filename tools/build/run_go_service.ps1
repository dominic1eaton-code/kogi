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
    'gateway'   { './kogi-services/go/gateway' }
    'auth'      { './kogi-services/go/services/auth' }
    'portfolio' { './kogi-services/go/services/portfolio' }
    'exchange'  { './kogi-services/go/services/exchange' }
    'ims'       { './kogi-services/go/services/ims' }
    'office'    { './kogi-services/go/services/office' }
    'bank'      { './kogi-services/go/services/bank' }
    'marketplace' { './kogi-services/go/services/marketplace' }
    'studio'    { './kogi-services/go/services/studio' }
    'community' { './kogi-services/go/services/community' }
    'developer' { './kogi-services/go/services/developer' }
    'profile'   { './kogi-services/go/services/profile' }
    'organizations' { './kogi-services/go/services/organizations' }
    'engine'    { './kogi-services/go/services/engine' }
    'database'  { './kogi-services/go/services/database' }
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
