@echo off
setlocal

set GUID=90cbdc39-4a3e-11d1-84f4-0000f80464e3
set DURATION=10

:: FileIo provider GUID: 90cbdc39-4a3e-11d1-84f4-0000f80464e3
:: All masks based on experiments/fileio-trace-test and experiments/fileio-flag-discovery

echo ============================================================
echo  tdh-enumerator: FileIo with all flags/masks + trigger
echo ============================================================
echo.

:: --- Individual EnableFlags ---
echo [1/13] EF:DISK_FILE_IO (0x00000200) - FileIo_Name events
tdh-enumerator -g %GUID% -d %DURATION% --enable-flags 0x00000200 --trigger -o ef_disk_file_io
echo.

echo [2/13] EF:FILE_IO (0x02000000) - FileIo_OpEnd events
tdh-enumerator -g %GUID% -d %DURATION% --enable-flags 0x02000000 --trigger -o ef_file_io
echo.

echo [3/13] EF:FILE_IO_INIT (0x04000000) - Create/Cleanup/Close/Read/Write/etc
tdh-enumerator -g %GUID% -d %DURATION% --enable-flags 0x04000000 --trigger -o ef_file_io_init
echo.

echo [4/13] EF:VAMAP (0x00008000) - MapFile events
tdh-enumerator -g %GUID% -d %DURATION% --enable-flags 0x00008000 --trigger -o ef_vamap
echo.

:: --- Individual GroupMask entries (Masks[4] for FLT masks) ---
echo [5/13] GM:PERF_FLT_IO_INIT (Masks[4]=0x80080000) - PreOpInit/PostOpInit
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x00000000,0x00000000,0x00000000,0x00000000,0x80080000,0x00000000,0x00000000,0x00000000" --trigger -o gm_flt_io_init
echo.

echo [6/13] GM:PERF_FLT_IO (Masks[4]=0x80100000) - PreOpCompletion/PostOpCompletion (IRP)
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x00000000,0x00000000,0x00000000,0x00000000,0x80100000,0x00000000,0x00000000,0x00000000" --trigger -o gm_flt_io
echo.

echo [7/13] GM:PERF_FLT_FASTIO (Masks[4]=0x80200000) - PreOpCompletion/PostOpCompletion (FastIO)
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x00000000,0x00000000,0x00000000,0x00000000,0x80200000,0x00000000,0x00000000,0x00000000" --trigger -o gm_flt_fastio
echo.

echo [8/13] GM:PERF_FLT_IO_FAILURE (Masks[4]=0x80400000) - PreOpFailure/PostOpFailure
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x00000000,0x00000000,0x00000000,0x00000000,0x80400000,0x00000000,0x00000000,0x00000000" --trigger -o gm_flt_io_failure
echo.

:: --- GroupMask equivalents of EnableFlags (Masks[0]) ---
echo [9/13] GM:PERF_FILENAME (Masks[0]=0x00000200) - same as EF:DISK_FILE_IO
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x00000200,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000" --trigger -o gm_perf_filename
echo.

echo [10/13] GM:PERF_FILE_IO (Masks[0]=0x02000000) - same as EF:FILE_IO
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x02000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000" --trigger -o gm_perf_file_io
echo.

echo [11/13] GM:PERF_FILE_IO_INIT (Masks[0]=0x04000000) - same as EF:FILE_IO_INIT
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x04000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000" --trigger -o gm_perf_file_io_init
echo.

echo [12/13] GM:PERF_VAMAP (Masks[0]=0x00008000) - same as EF:VAMAP
tdh-enumerator -g %GUID% -d %DURATION% --group-mask "0x00008000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000" --trigger -o gm_perf_vamap
echo.

:: --- Everything combined ---
echo [13/13] COMBO:ALL_EF + ALL_FLT - all EnableFlags + all FLT masks
tdh-enumerator -g %GUID% -d %DURATION% --enable-flags 0x06008200 --group-mask "0x00000000,0x00000000,0x00000000,0x00000000,0x80780000,0x00000000,0x00000000,0x00000000" --trigger -o combo_all
echo.

echo ============================================================
echo  Done. All output files have prefix tdh_output_*.json
echo ============================================================
endlocal
