#Requires -Version 5.1
<#
    tdh-enumerator: Wrapper that runs the FileIo (all flags/masks) scenario.
    Delegates to run_fileio_all.ps1 and propagates its exit code.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ChildScript = Join-Path $ScriptDir "run_fileio_all.ps1"

if (-not (Test-Path $ChildScript)) {
    Write-Host "Missing: $ChildScript" -ForegroundColor Red
    exit 1
}

& $ChildScript
exit $LASTEXITCODE