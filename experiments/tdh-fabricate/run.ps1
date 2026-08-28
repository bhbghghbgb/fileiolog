#Requires -Version 5.1
<#
    tdh-fabricate: Understand TdhGetEventInformation field requirements.
    Partial elevation: experiments 1,3,5,6,7 need admin; 2,4 work without.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Auto-elevate ──────────────────────────────────────────────
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator
)
if (-not $isAdmin) {
    Write-Host "Relaunching as Administrator (needed for experiments 1,3,5,6,7)..." -ForegroundColor Yellow
    $scriptPath = $MyInvocation.MyCommand.Path
    Start-Process -FilePath "powershell.exe" -ArgumentList @(
        "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "`"$scriptPath`""
    ) -Verb RunAs -Wait
    exit
}

# ── Paths ─────────────────────────────────────────────────────
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$ExePath    = Join-Path (Split-Path -Parent (Split-Path -Parent $ScriptDir)) "target\debug\tdh-fabricate.exe"
$OutputDir  = Join-Path $ScriptDir "output"

if (-not (Test-Path $ExePath)) {
    Write-Host "Executable not found: $ExePath" -ForegroundColor Red
    Write-Host "Build first with: cargo build -p tdh-fabricate" -ForegroundColor Yellow
    exit 1
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# ── Run ───────────────────────────────────────────────────────
Write-Host ("=" * 60)
Write-Host " tdh-fabricate: TDH fabrication experiments"
Write-Host ("=" * 60)

& $ExePath -o $OutputDir

if ($LASTEXITCODE -ne 0) {
    Write-Warning "tdh-fabricate exited with code $LASTEXITCODE"
}

Write-Host "`n$("=" * 60)" -ForegroundColor Green
Write-Host " Done. Output in: $OutputDir" -ForegroundColor Green
Write-Host ("=" * 60) -ForegroundColor Green
