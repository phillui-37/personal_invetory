param(
    [string]$Verb,
    [string]$Area,
    [string]$Target
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib.ps1')

$repoRoot = Get-RepoRoot

function Start-Backend {
    Invoke-InDirectory (Join-Path $repoRoot 'backend') { cargo run -p app }
}

function Start-Frontend {
    param([string]$FrontendTarget)

    Assert-FrontendTarget -RepoRoot $repoRoot -Target $FrontendTarget
    Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter run -d $FrontendTarget }
}

function Build-Backend {
    Invoke-InDirectory (Join-Path $repoRoot 'backend') { cargo build -p app --release }
    $binaryName = Get-BackendBinaryName
    Copy-Artifact `
        -Source (Join-Path $repoRoot "backend/target/release/$binaryName") `
        -Destination (Join-Path $repoRoot "dist/backend/$binaryName")
}

function Build-Frontend {
    param([string]$FrontendTarget)

    Assert-FrontendTarget -RepoRoot $repoRoot -Target $FrontendTarget

    switch ($FrontendTarget) {
        'android' {
            Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter build apk --release }
            Copy-Artifact `
                -Source (Join-Path $repoRoot 'frontend/build/app/outputs/flutter-apk/app-release.apk') `
                -Destination (Join-Path $repoRoot 'dist/frontend/android/app-release.apk')
        }
        'macos' {
            Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter build macos --release }
            $artifact = Find-SingleItem -Directory (Join-Path $repoRoot 'frontend/build/macos/Build/Products/Release') -Filter '*.app'
            Copy-Artifact -Source $artifact -Destination (Join-Path $repoRoot ("dist/frontend/macos/{0}" -f (Split-Path -Leaf $artifact)))
        }
        'ios' {
            Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter build ios --release --no-codesign }
            $artifact = Find-SingleItem -Directory (Join-Path $repoRoot 'frontend/build/ios/iphoneos') -Filter '*.app'
            Copy-Artifact -Source $artifact -Destination (Join-Path $repoRoot ("dist/frontend/ios/{0}" -f (Split-Path -Leaf $artifact)))
        }
        'windows' {
            Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter build windows --release }
            Copy-Artifact `
                -Source (Join-Path $repoRoot 'frontend/build/windows/x64/runner/Release') `
                -Destination (Join-Path $repoRoot 'dist/frontend/windows/Release')
        }
        'linux' {
            Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter build linux --release }
            Copy-Artifact `
                -Source (Join-Path $repoRoot 'frontend/build/linux/x64/release/bundle') `
                -Destination (Join-Path $repoRoot 'dist/frontend/linux/bundle')
        }
        default {
            Fail "unsupported frontend target '$FrontendTarget'"
        }
    }
}

function Test-Backend {
    Invoke-InDirectory (Join-Path $repoRoot 'backend') { cargo test }
}

function Test-Frontend {
    Invoke-InDirectory (Join-Path $repoRoot 'frontend') {
        flutter build apk --debug
        flutter test
    }
}

function Test-All {
    Test-Backend
    Test-Frontend
}

function Clean-Repo {
    if (Test-Path -LiteralPath (Join-Path $repoRoot 'dist')) {
        Remove-Item -LiteralPath (Join-Path $repoRoot 'dist') -Recurse -Force
    }

    Invoke-InDirectory (Join-Path $repoRoot 'backend') { cargo clean }
    Invoke-InDirectory (Join-Path $repoRoot 'frontend') { flutter clean }
}

function Generate-Api {
    $specUrl = if ($env:OPENAPI_SPEC_URL) { $env:OPENAPI_SPEC_URL } else { 'http://localhost:8080/api/v1/system/openapi' }
    & openapi-generator-cli generate `
        -i $specUrl `
        -g dart `
        -o (Join-Path $repoRoot 'frontend/lib/api')
}

switch ("$Verb:$Area:$Target") {
    'start:backend:' {
        if ($PSBoundParameters.Count -ne 2) { Show-Usage }
        Start-Backend
        break
    }
    'start:frontend:macos' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Start-Frontend $Target
        break
    }
    'start:frontend:windows' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Start-Frontend $Target
        break
    }
    'start:frontend:linux' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Start-Frontend $Target
        break
    }
    'start:frontend:ios' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Start-Frontend $Target
        break
    }
    'start:frontend:android' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Start-Frontend $Target
        break
    }
    'build:backend:' {
        if ($PSBoundParameters.Count -ne 2) { Show-Usage }
        Build-Backend
        break
    }
    'build:frontend:macos' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Build-Frontend $Target
        break
    }
    'build:frontend:windows' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Build-Frontend $Target
        break
    }
    'build:frontend:linux' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Build-Frontend $Target
        break
    }
    'build:frontend:ios' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Build-Frontend $Target
        break
    }
    'build:frontend:android' {
        if ($PSBoundParameters.Count -ne 3) { Show-Usage }
        Build-Frontend $Target
        break
    }
    'test:backend:' {
        if ($PSBoundParameters.Count -ne 2) { Show-Usage }
        Test-Backend
        break
    }
    'test:frontend:' {
        if ($PSBoundParameters.Count -ne 2) { Show-Usage }
        Test-Frontend
        break
    }
    'test:all:' {
        if ($PSBoundParameters.Count -ne 2) { Show-Usage }
        Test-All
        break
    }
    'clean::' {
        if ($PSBoundParameters.Count -ne 1) { Show-Usage }
        Clean-Repo
        break
    }
    'gen-api::' {
        if ($PSBoundParameters.Count -ne 1) { Show-Usage }
        Generate-Api
        break
    }
    default {
        Show-Usage
    }
}
