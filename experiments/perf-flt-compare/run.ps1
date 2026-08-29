#Requires -Version 5.1
<#
    perf-flt-compare: Compare PERF_FLT_IO vs PERF_FLT_FASTIO event behavior.
    Requires elevation (ETW kernel tracing needs admin).
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Auto-elevate ──────────────────────────────────────────────
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator
)
if (-not $isAdmin) {
    Write-Host "Relaunching as Administrator..." -ForegroundColor Yellow
    $scriptPath = $MyInvocation.MyCommand.Path
    try {
        $p = Start-Process -FilePath "powershell.exe" -ArgumentList @(
            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "`"$scriptPath`""
        ) -Verb RunAs -Wait -PassThru
        exit $p.ExitCode
    } catch {
        Write-Warning "Elevation failed: $_"
        exit 1
    }
}

# ── Paths ─────────────────────────────────────────────────────
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$ExePath    = Join-Path (Split-Path -Parent (Split-Path -Parent $ScriptDir)) "target\debug\perf-flt-compare.exe"
$OutputDir  = Join-Path $ScriptDir "output"

if (-not (Test-Path $ExePath)) {
    Write-Host "Executable not found: $ExePath" -ForegroundColor Red
    Write-Host "Build first with: cargo build -p perf-flt-compare" -ForegroundColor Yellow
    exit 1
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# ── Run ───────────────────────────────────────────────────────
Write-Host ("=" * 60)
Write-Host " perf-flt-compare: PERF_FLT_IO vs PERF_FLT_FASTIO comparison"
Write-Host ("=" * 60)

& $ExePath -o $OutputDir

if ($LASTEXITCODE -ne 0) {
    Write-Warning "perf-flt-compare exited with code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host "`n$("=" * 60)" -ForegroundColor Green
Write-Host " Done. Output in: $OutputDir" -ForegroundColor Green
Write-Host ("=" * 60) -ForegroundColor Green
