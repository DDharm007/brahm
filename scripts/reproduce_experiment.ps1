<#
.SYNOPSIS
    Reproduces BRAHM Stage 07 validation & protected workspace pipeline cleanly from scratch.
.DESCRIPTION
    Applies BRAHM protection (`brahm apply --profile research-mode`) to `demo/brahm-commerce-demo`
    inside a fresh temporary workspace without mutating any genuine demo source files, replays
    workspace file accesses (including decoys/canaries) into `brahm_monitoring.db`, checks
    BLAKE3 integrity, generates all Stage 06 security intelligence reports, and optionally rolls
    back temporary state cleanly.
#>
param(
    [switch]$KeepArtifacts
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$brahmExe = Join-Path $repoRoot "target\release\brahm.exe"
$demoRepo = Join-Path $repoRoot "demo\brahm-commerce-demo"

if (-not (Test-Path $brahmExe)) {
    Write-Host "[BRAHM] Building release binary..."
    Push-Location $repoRoot
    cargo build --release
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
    Pop-Location
}

$workDir = Join-Path $env:TEMP ("brahm_repro_" + [guid]::NewGuid().ToString('N').Substring(0, 8))
$stateDir = Join-Path $workDir "state"
$reportDir = Join-Path $workDir "reports"
New-Item -ItemType Directory -Force -Path $stateDir, $reportDir | Out-Null

Write-Host "[1/5] Verifying unprotected demo target tests..."
Push-Location $demoRepo
if (-not (Test-Path "node_modules")) {
    npm install
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
}
npx tsc
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
node --test dist/tests/*.test.js
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
Remove-Item -Recurse -Force "dist" -ErrorAction SilentlyContinue
Pop-Location

Write-Host "[2/5] Applying BRAHM research-mode protection via 'brahm apply'..."
$applyRaw = & $brahmExe apply $demoRepo --state-dir $stateDir --profile research-mode --json
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$applied = $applyRaw | ConvertFrom-Json
$sid = $applied.session_id
Write-Host "      Session ID:           $sid"
Write-Host "      Inspection Workspace: $($applied.inspection_workspace)"
Write-Host "      Staged Decoys/Canary: $($applied.decoy_count) / $($applied.canary_count)"

Write-Host "[3/5] Executing sandbox agent inspection across all 48 workspace entries..."
$manifest = Get-Content (Join-Path $stateDir "sessions\$sid\BRAHM_INSPECTION_MANIFEST.json") -Raw | ConvertFrom-Json
$allPaths = $manifest.entries | ForEach-Object { $_.relative_path }
$agentJson = & $brahmExe agent run $sid --state-dir $stateDir --paths @allPaths
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$agentRes = $agentJson | ConvertFrom-Json
Write-Host "      Accessed Files: $($agentRes.accessed_files.Count) | Blocked: $($agentRes.blocked_attempts.Count)"

Write-Host "[4/5] Verifying cryptographic BLAKE3 integrity and generating intelligence reports..."
$intRaw = & $brahmExe integrity check --state-dir $stateDir
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$intObj = $intRaw | ConvertFrom-Json
& $brahmExe reports generate --state-dir $stateDir --output $reportDir | Out-Null
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$mon = Get-Content (Join-Path $reportDir "BRAHM_MONITORING_REPORT.json") -Raw | ConvertFrom-Json
Write-Host "      Total SQLite Events: $($mon.total_events)"
Write-Host "      Canary Detections:   $($mon.canary_events_count)"
Write-Host "      Baseline Clean:      $($intObj.baseline_matches) | Decoy Intact: $($intObj.decoy_integrity_valid)"

if (-not $KeepArtifacts) {
    Write-Host "[5/5] Rolling back isolated workspace and cleaning temp directory..."
    & $brahmExe protection rollback --state-dir $stateDir | Out-Null
    Remove-Item -Recurse -Force $workDir -ErrorAction SilentlyContinue
} else {
    Write-Host "[5/5] Artifacts preserved under: $workDir"
}

Write-Host "Reproduction completed cleanly."
