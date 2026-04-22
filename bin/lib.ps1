Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Fail {
    param([string]$Message)

    throw "error: $Message"
}

function Show-Usage {
    @'
usage:
  .\bin\app.ps1 start backend
  .\bin\app.ps1 start frontend <macos|windows|linux|ios|android>
  .\bin\app.ps1 build backend
  .\bin\app.ps1 build frontend <macos|windows|linux|ios|android>
  .\bin\app.ps1 test <backend|frontend|all>
  .\bin\app.ps1 clean
  .\bin\app.ps1 gen-api
'@ | Write-Error
    exit 1
}

function Get-RepoRoot {
    return (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
}

function Get-HostPlatform {
    if ($IsWindows) { return 'windows' }
    if ($IsMacOS) { return 'macos' }
    if ($IsLinux) { return 'linux' }
    return 'unknown'
}

function Get-BackendBinaryName {
    if ((Get-HostPlatform) -eq 'windows') {
        return 'app.exe'
    }

    return 'app'
}

function Assert-FrontendTarget {
    param(
        [string]$RepoRoot,
        [string]$Target
    )

    $hostPlatform = Get-HostPlatform

    switch ($Target) {
        'macos' {
            if ($hostPlatform -ne 'macos') { Fail "frontend target '$Target' requires a macOS host" }
        }
        'windows' {
            if ($hostPlatform -ne 'windows') { Fail "frontend target '$Target' requires a Windows host" }
        }
        'linux' {
            if ($hostPlatform -ne 'linux') { Fail "frontend target '$Target' requires a Linux host" }
        }
        'ios' {
            if ($hostPlatform -ne 'macos') { Fail "frontend target '$Target' requires a macOS host" }
        }
        'android' {
            if ($hostPlatform -notin @('macos', 'windows', 'linux')) {
                Fail "frontend target '$Target' is unsupported on host '$hostPlatform'"
            }
        }
        default {
            Fail "unsupported frontend target '$Target'"
        }
    }

    $targetDir = Join-Path $RepoRoot "frontend/$Target"
    if (-not (Test-Path -LiteralPath $targetDir -PathType Container)) {
        Fail "frontend target '$Target' is not configured in this repo"
    }
}

function Copy-Artifact {
    param(
        [string]$Source,
        [string]$Destination
    )

    if (-not (Test-Path -LiteralPath $Source)) {
        Fail "missing artifact '$Source'"
    }

    $parent = Split-Path -Parent $Destination
    if ($parent) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    if (Test-Path -LiteralPath $Destination) {
        Remove-Item -LiteralPath $Destination -Recurse -Force
    }

    Copy-Item -LiteralPath $Source -Destination $Destination -Recurse -Force
}

function Find-SingleItem {
    param(
        [string]$Directory,
        [string]$Filter
    )

    if (-not (Test-Path -LiteralPath $Directory -PathType Container)) {
        Fail "missing search directory '$Directory'"
    }

    $items = Get-ChildItem -LiteralPath $Directory -Filter $Filter
    if ($items.Count -eq 0) {
        Fail "no artifacts matching '$Filter' under '$Directory'"
    }
    if ($items.Count -ne 1) {
        Fail "multiple artifacts matching '$Filter' under '$Directory'"
    }

    return $items[0].FullName
}

function Invoke-InDirectory {
    param(
        [string]$Path,
        [scriptblock]$Script
    )

    Push-Location $Path
    try {
        & $Script
    }
    finally {
        Pop-Location
    }
}
