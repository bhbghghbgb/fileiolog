#Requires -Version 5.1
<#
    run-all.ps1: Run every project under ./experiments that has a run.ps1.
    Runs sequentially, each in its own project folder, shows all console output,
    tracks success/failure per project, and reports a summary at the end.

    Exit code: 0 if all succeeded, 1 if any failed.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RootDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$projectDirs = Get-ChildItem -LiteralPath $RootDir -Directory | Sort-Object Name

$results = @()

Write-Host ("=" * 60)
Write-Host " Running all experiments (sequential)"
Write-Host ("=" * 60)

foreach ($dir in $projectDirs) {
    $runScript = Join-Path $dir.FullName "run.ps1"

    if (-not (Test-Path -LiteralPath $runScript)) {
        Write-Host "[SKIP]  $($dir.Name)  (no run.ps1)" -ForegroundColor DarkGray
        continue
    }

    $name = $dir.Name
    Write-Host ""
    Write-Host ("-" * 60) -ForegroundColor Cyan
    Write-Host "  >>> $name" -ForegroundColor Cyan
    Write-Host ("-" * 60) -ForegroundColor Cyan

    $global:LASTEXITCODE = 0
    $failCode = 0
    try {
        Push-Location $dir.FullName
        try {
            & $runScript
        } finally {
            Pop-Location
        }
        if (-not $?) {
            $failCode = 1
        } elseif ($LASTEXITCODE -ne 0) {
            $failCode = $LASTEXITCODE
        }
    } catch {
        Write-Warning "  $name threw an unhandled error: $_"
        $failCode = 1
    }

    $results += [pscustomobject]@{
        Name   = $name
        Status = if ($failCode -eq 0) { "SUCCESS" } else { "FAIL ($failCode)" }
        Code   = $failCode
    }
}

Write-Host ""
Write-Host ("=" * 60)
Write-Host " Summary"
Write-Host ("=" * 60)

foreach ($r in $results) {
    $color = if ($r.Code -eq 0) { "Green" } else { "Red" }
    Write-Host ("  {0,-6} {1}" -f $r.Status, $r.Name) -ForegroundColor $color
}

$failed = @($results | Where-Object { $_.Code -ne 0 })
$succeeded = $results.Count - $failed.Count

Write-Host ("=" * 60)
Write-Host " Succeeded: $succeeded / $($results.Count)"
if ($failed.Count -gt 0) {
    Write-Host " Failed:    $($failed.Count)" -ForegroundColor Red
    Write-Host ("  " + (($failed | ForEach-Object { $_.Name }) -join ", ")) -ForegroundColor Red
    exit 1
} else {
    Write-Host " All projects passed." -ForegroundColor Green
    exit 0
}