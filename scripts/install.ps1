<#
.SYNOPSIS
    Installs the standalone BRAHM CLI executable on Windows without requiring Rust or Cargo.
.DESCRIPTION
    Downloads the latest pre-built Windows binary from GitHub Releases (with SHA-256 verification)
    OR installs from a local prebuilt binary (-LocalBinary), copies it into
    %LOCALAPPDATA%\brahm\bin\brahm.exe, and adds that folder to the user PATH.
#>
param(
    [string]$Version = "latest",
    [string]$LocalBinary = "",
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA "brahm\bin"),
    [switch]$SkipPathUpdate
)

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$destExe = Join-Path $InstallDir "brahm.exe"

if ($LocalBinary -ne "") {
    if (-not (Test-Path $LocalBinary)) {
        throw "Local binary not found at '$LocalBinary'."
    }
    Copy-Item -Path $LocalBinary -Destination $destExe -Force
    Write-Host "[BRAHM] Installed from local binary to $destExe"
} else {
    $baseUrl = if ($Version -eq "latest") {
        "https://github.com/DDharm007/brahm/releases/latest/download"
    } else {
        "https://github.com/DDharm007/brahm/releases/download/$Version"
    }
    $assetUrl = "$baseUrl/brahm-windows-x86_64.exe"
    $sumsUrl = "$baseUrl/SHA256SUMS.txt"

    Write-Host "[BRAHM] Downloading standalone Windows binary (no Rust required)..."
    Invoke-WebRequest -Uri $assetUrl -OutFile $destExe -UseBasicParsing

    try {
        $sumsRaw = (Invoke-WebRequest -Uri $sumsUrl -UseBasicParsing).Content
        $expectedLine = ($sumsRaw -split "`n" | Where-Object { $_ -match "brahm-windows-x86_64\.exe" } | Select-Object -First 1)
        if ($expectedLine) {
            $expectedHash = ($expectedLine.Trim() -split "\s+")[0].ToUpperInvariant()
            $actualHash = (Get-FileHash -Path $destExe -Algorithm SHA256).Hash.ToUpperInvariant()
            if ($expectedHash -ne $actualHash) {
                Remove-Item -Force $destExe -ErrorAction SilentlyContinue
                throw "SHA-256 verification failed! Expected $expectedHash, got $actualHash"
            }
            Write-Host "[BRAHM] SHA-256 checksum verified: $actualHash"
        }
    } catch {
        Write-Warning "Could not verify remote checksum cleanly ($($_.Exception.Message))."
    }
}

if (-not $SkipPathUpdate) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ([string]::IsNullOrEmpty($userPath)) {
        [Environment]::SetEnvironmentVariable("Path", $InstallDir, "User")
    } elseif (($userPath -split ';') -notcontains $InstallDir) {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    }
}

& $destExe --version
Write-Host "[BRAHM] Ready. Open a new terminal and run: brahm apply C:\path\to\your-project --shield"
