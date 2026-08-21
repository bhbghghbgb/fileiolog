#Requires -Version 5.1
<#
    tdh-enumerator: FileIo with all flags/masks + file-ops-trigger
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
    Start-Process -FilePath "powershell.exe" -ArgumentList @(
        "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "`"$scriptPath`""
    ) -Verb RunAs -Wait
    exit
}

# ── Paths ─────────────────────────────────────────────────────
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$ExePath    = Join-Path (Split-Path -Parent (Split-Path -Parent $ScriptDir)) "target\debug\tdh-enumerator.exe"
$OutputDir  = Join-Path $ScriptDir "output"

if (-not (Test-Path $ExePath)) {
    Write-Host "Executable not found: $ExePath" -ForegroundColor Red
    Write-Host "Build first with: cargo build -p tdh-enumerator" -ForegroundColor Yellow
    exit 1
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# ── Config ────────────────────────────────────────────────────
$Guid     = "90cbdc39-4a3e-11d1-84f4-0000f80464e3"
$Duration = 10

# ── Helper ────────────────────────────────────────────────────
function Invoke-Run {
    param(
        [string] $Label,
        [string] $Tag,
        [string[]] $ExtraArgs
    )
    Write-Host "`n[$Tag] $Label" -ForegroundColor Cyan

    $outFile = Join-Path $OutputDir "tdh_output_$Tag"
    $argsList = @(
        "-g", $Guid,
        "-d", $Duration.ToString(),
        "--trigger",
        "-o", $outFile
    ) + $ExtraArgs

    & $ExePath @argsList

    if ($LASTEXITCODE -ne 0) {
        Write-Warning "tdh-enumerator exited with code $LASTEXITCODE"
    }
}

# ── Runs ──────────────────────────────────────────────────────
Write-Host ("=" * 60)
Write-Host " tdh-enumerator: FileIo all flags/masks + trigger"
Write-Host ("=" * 60)

# --- Individual EnableFlags ---
Invoke-Run "EF:DISK_FILE_IO (0x00000200) - FileIo_Name events"            "ef_disk_file_io"    @("--enable-flags", "0x00000200")
Invoke-Run "EF:FILE_IO (0x02000000) - FileIo_OpEnd events"                "ef_file_io"         @("--enable-flags", "0x02000000")
Invoke-Run "EF:FILE_IO_INIT (0x04000000) - Create/Close/Read/Write/etc"   "ef_file_io_init"    @("--enable-flags", "0x04000000")
Invoke-Run "EF:VAMAP (0x00008000) - MapFile events"                       "ef_vamap"           @("--enable-flags", "0x00008000")

# --- FLT masks via group-mask (Masks[4]) ---
$gm0 = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000"

Invoke-Run "GM:PERF_FLT_IO_INIT (Masks[4]=0x80080000) - PreOpInit/PostOpInit"                         "gm_flt_io_init"   @("--group-mask", "0x00000000,0x00000000,0x00000000,0x00000000,0x80080000,0x00000000,0x00000000,0x00000000")
Invoke-Run "GM:PERF_FLT_IO (Masks[4]=0x80100000) - PreOpCompletion/PostOpCompletion (IRP)"            "gm_flt_io"        @("--group-mask", "0x00000000,0x00000000,0x00000000,0x00000000,0x80100000,0x00000000,0x00000000,0x00000000")
Invoke-Run "GM:PERF_FLT_FASTIO (Masks[4]=0x80200000) - PreOpCompletion/PostOpCompletion (FastIO)"    "gm_flt_fastio"    @("--group-mask", "0x00000000,0x00000000,0x00000000,0x00000000,0x80200000,0x00000000,0x00000000,0x00000000")
Invoke-Run "GM:PERF_FLT_IO_FAILURE (Masks[4]=0x80400000) - PreOpFailure/PostOpFailure"               "gm_flt_io_failure" @("--group-mask", "0x00000000,0x00000000,0x00000000,0x00000000,0x80400000,0x00000000,0x00000000,0x00000000")

# --- GM equivalents of EnableFlags (Masks[0]) ---
Invoke-Run "GM:PERF_FILENAME (Masks[0]=0x00000200) - same as EF:DISK_FILE_IO"     "gm_perf_filename"      @("--group-mask", "0x00000200,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000")
Invoke-Run "GM:PERF_FILE_IO (Masks[0]=0x02000000) - same as EF:FILE_IO"          "gm_perf_file_io"       @("--group-mask", "0x02000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000")
Invoke-Run "GM:PERF_FILE_IO_INIT (Masks[0]=0x04000000) - same as EF:FILE_IO_INIT" "gm_perf_file_io_init"  @("--group-mask", "0x04000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000")
Invoke-Run "GM:PERF_VAMAP (Masks[0]=0x00008000) - same as EF:VAMAP"              "gm_perf_vamap"         @("--group-mask", "0x00008000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000")

# --- Everything combined ---
Invoke-Run "COMBO:ALL_EF + ALL_FLT - all EnableFlags + all FLT masks" "combo_all" @(
    "--enable-flags", "0x06008200",
    "--group-mask",   "0x00000000,0x00000000,0x00000000,0x00000000,0x80780000,0x00000000,0x00000000,0x00000000"
)

# ── Done ──────────────────────────────────────────────────────
Write-Host "`n$("=" * 60)" -ForegroundColor Green
Write-Host " Done. Output in: $OutputDir" -ForegroundColor Green
Write-Host ("=" * 60) -ForegroundColor Green
