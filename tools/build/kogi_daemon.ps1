param(
    [Parameter(Mandatory = $false)]
    [ValidateSet('start','stop','cycle','status')]
    [string]$Action = 'start',
    [Parameter(Mandatory = $false)]
    [string[]]$Target = @('all'),
    [switch]$Debug,
    [switch]$NoSilent
)

$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$stateDir = Join-Path $root '.cache\kogi'
$stateFile = Join-Path $stateDir 'background.json'
$logDir = Join-Path $stateDir 'logs'

New-Item -ItemType Directory -Force -Path $stateDir | Out-Null
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

$env:GOWORK = Join-Path $root 'go.work'
$env:GOCACHE = Join-Path $root '.cache\go-build'
$env:GOMODCACHE = Join-Path $root '.cache\go-mod'
New-Item -ItemType Directory -Force -Path $env:GOCACHE | Out-Null
New-Item -ItemType Directory -Force -Path $env:GOMODCACHE | Out-Null

$serviceNames = @(
    'gateway',
    'auth',
    'portfolio',
    'exchange',
    'ims',
    'office',
    'bank',
    'marketplace',
    'studio',
    'community',
    'developer',
    'profile',
    'organizations',
    'engine',
    'database'
)

$allNames = $serviceNames + @('server')

function Load-State {
    if (Test-Path $stateFile) {
        try {
            return Get-Content $stateFile -Raw | ConvertFrom-Json
        } catch {
            return @{}
        }
    }
    return @{}
}

function Save-State($state) {
    ($state | ConvertTo-Json -Depth 6) | Set-Content -Path $stateFile
}

function Resolve-Targets([string[]]$targets) {
    if ($targets -contains 'all') {
        return $allNames
    }
    if ($targets -contains 'services') {
        return $serviceNames
    }
    return $targets
}

function Is-Running([int]$pid) {
    $proc = Get-Process -Id $pid -ErrorAction SilentlyContinue
    return $null -ne $proc
}

function Start-ServiceProcess([string]$name, [hashtable]$state) {
    if ($state.ContainsKey($name) -and $state[$name].pid) {
        $existing = [int]$state[$name].pid
        if (Is-Running $existing) {
            Write-Host "[skip] $name already running (pid=$existing)"
            return
        }
    }

    $flags = @()
    if (-not $NoSilent) {
        $flags += '--silent'
    }
    if ($Debug) {
        $flags += '--debug'
    }

    $stdout = Join-Path $logDir "$name.out.log"
    $stderr = Join-Path $logDir "$name.err.log"

    switch ($name) {
        'server' {
            $cmd = 'cargo'
            $args = @('run', '--manifest-path', 'kogi-server/Cargo.toml', '--') + $flags
        }
        'gateway' {
            $cmd = 'go'
            $args = @('run', './kogi-services/go/gateway') + $flags
        }
        default {
            $cmd = 'go'
            $args = @('run', "./kogi-services/go/services/$name") + $flags
        }
    }

    Write-Host "[start] $name -> $cmd $($args -join ' ')"
    $proc = Start-Process -FilePath $cmd -ArgumentList $args -WorkingDirectory $root -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru -WindowStyle Hidden

    $state[$name] = @{
        pid = $proc.Id
        command = "$cmd $($args -join ' ')"
        started_at = (Get-Date).ToString('o')
        silent = (-not $NoSilent)
        debug = $Debug
        stdout = $stdout
        stderr = $stderr
    }
}

function Stop-ServiceProcess([string]$name, [hashtable]$state) {
    if (-not $state.ContainsKey($name)) {
        Write-Host "[skip] $name not tracked"
        return
    }
    $pid = [int]$state[$name].pid
    if ($pid -eq 0) {
        Write-Host "[skip] $name has no pid"
        $state.Remove($name) | Out-Null
        return
    }
    if (Is-Running $pid) {
        Write-Host "[stop] $name pid=$pid"
        Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
    } else {
        Write-Host "[skip] $name pid=$pid not running"
    }
    $state.Remove($name) | Out-Null
}

$state = Load-State
$targets = Resolve-Targets $Target

switch ($Action) {
    'start' {
        foreach ($name in $targets) {
            Start-ServiceProcess $name $state
        }
        Save-State $state
    }
    'stop' {
        foreach ($name in $targets) {
            Stop-ServiceProcess $name $state
        }
        Save-State $state
    }
    'cycle' {
        foreach ($name in $targets) {
            Stop-ServiceProcess $name $state
        }
        foreach ($name in $targets) {
            Start-ServiceProcess $name $state
        }
        Save-State $state
    }
    'status' {
        foreach ($name in $targets) {
            if ($state.ContainsKey($name) -and $state[$name].pid) {
                $pid = [int]$state[$name].pid
                $running = Is-Running $pid
                $started = $state[$name].started_at
                Write-Host "[status] $name pid=$pid running=$running started=$started"
            } else {
                Write-Host "[status] $name not running"
            }
        }
    }
}
