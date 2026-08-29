#Requires -Version 5.1
<#
    file-ops-trigger: Trigger file system operations for ETW tracing.
    No elevation needed - this is a user-mode utility.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Paths ─────────────────────────────────────────────────────
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$ExePath    = Join-Path (Split-Path -Parent (Split-Path -Parent $ScriptDir)) "target\debug\file-ops-trigger.exe"
$OutputDir  = Join-Path $ScriptDir "output"

if (-not (Test-Path $ExePath)) {
    Write-Host "Executable not found: $ExePath" -ForegroundColor Red
    Write-Host "Build first with: cargo build -p file-ops-trigger" -ForegroundColor Yellow
    exit 1
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# ── Run ───────────────────────────────────────────────────────
Write-Host ("=" * 60)
Write-Host " file-ops-trigger: Triggering file system operations"
Write-Host ("=" * 60)

& $ExePath $OutputDir

if ($LASTEXITCODE -ne 0) {
    Write-Warning "file-ops-trigger exited with code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host "`n$("=" * 60)" -ForegroundColor Green
Write-Host " Done." -ForegroundColor Green
Write-Host ("=" * 60) -ForegroundColor Green
