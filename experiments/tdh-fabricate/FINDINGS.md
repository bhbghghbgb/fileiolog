# TDH Fabricate Experiment: Understanding `TdhGetEventInformation` Field Requirements

## Purpose

This experiment investigates what fields in an `EVENT_RECORD` are needed for the Windows TDH (Trace Data Helper) API `TdhGetEventInformation` to successfully return event schema information. The specific goal is to understand if and how event records can be **fabricated** (created from scratch or modified from existing records) to query schemas for rare/hard-to-capture events.

**Scope**: Legacy kernel tracing providers only (NOT manifest-based user tracing).

---

## Key Findings (Summary)

| Finding | Detail |
|---------|--------|
| **Fabrication works for manifest-based providers** | `Microsoft-Windows-Kernel-Process` (GUID `22fb2cd6-...`) and `Microsoft-Windows-Kernel-File` (GUID `edd08927-...`) succeed with only `ProviderId` + `EventDescriptor` |
| **Fabrication FAILS for legacy kernel providers** | `FileIo` kernel provider (GUID `90cbdc39-...`) returns `ERROR_NOT_FOUND` (1168) regardless of flags, properties, or descriptor values |
| **No combination of Flags/EventProperty fixes the kernel provider** | Tested `CLASSIC_HEADER` (0x0010), `LEGACY_EVENTLOG` (0x0004), `EXTENDED_INFO` (0x0001) - all fail |
| **Admin privileges required for kernel tracing** | Experiments 1, 3, 5, 6, 7 require admin to capture real events; experiments 2 and 4 work without admin |

---

## Architecture

### How `TdhGetEventInformation` Locates Schema

The API uses a metadata-source priority order:

1. **TraceLogging payload** (self-describing schema in the event)
2. **Manifest publisher** (`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WINEVT\Publishers\{GUID}`)
3. **WMI MOF repository** (legacy kernel providers)
4. **WPP** (requires external TMF/PDB)

The critical difference:
- **Manifest-based providers** have their metadata registered in the WINEVT registry under `Publishers\{GUID}`, with the provider binary containing the `WEVT_TEMPLATE` resource. TDH can find this without any ETW session context.
- **Legacy kernel providers** (like FileIo GUID `90cbdc39-...`) use the WMI MOF repository. TDH apparently cannot locate their metadata from fabricated records alone.

### EVENT_RECORD Structure

```c
typedef struct _EVENT_RECORD {
    EVENT_HEADER EventHeader;           // Key fields for TDH lookup
    ETW_BUFFER_CONTEXT BufferContext;   // Logger ID, processor info
    USHORT ExtendedDataCount;
    USHORT UserDataLength;              // Size of payload
    PEVENT_HEADER_EXTENDED_DATA_ITEM ExtendedData;
    PVOID UserData;                     // The actual event payload bytes
    PVOID UserContext;
} EVENT_RECORD;
```

Inside `EVENT_HEADER`, the critical fields for TDH are:

| Field | Type | Purpose |
|-------|------|---------|
| `ProviderId` | GUID | **Primary lookup key** - identifies which provider's schema to search |
| `EventDescriptor.Id` | u16 | Event ID within the provider |
| `EventDescriptor.Version` | u8 | Schema version |
| `EventDescriptor.Opcode` | u8 | Used in schema matching |
| `EventDescriptor.Task` | u16 | Used in schema matching |
| `EventDescriptor.Keyword` | u64 | Used in schema matching |
| `Flags` | u16 | Determines code path (WPP/Classic/Manifest) |
| `EventProperty` | u16 | Further disambiguation (XML/LegacyEventlog) |

---

## Experiment Results

### Experiment 2: Minimal Fabrication (No Admin Required)

Created `EVENT_RECORD` structures with only `ProviderId` + `EventDescriptor` set, all other fields zeroed.

#### Legacy Kernel Provider (FileIo - `90cbdc39-...`)

**ALL FAIL** with `ERROR_NOT_FOUND` (1168), regardless of:

| Flag Combination | Result |
|------------------|--------|
| Flags=0 (default) | FAIL |
| Flags=0x0010 (CLASSIC_HEADER) | FAIL |
| Flags=0x0001 (EXTENDED_INFO) | FAIL |
| EventProperty=0x0004 (LEGACY_EVENTLOG) | FAIL |
| CLASSIC_HEADER + real opcode values | FAIL |

Tested event IDs: 0, 32, 64, 65, 66, 67, 68, 76, 96, 97, 98, 99, 100, 101

#### Manifest-Based Providers

**ALL SUCCEED** with only `ProviderId` + `EventDescriptor`:

| Provider | Event | ID | Version | Properties | Status |
|----------|-------|----|---------|------------|--------|
| Microsoft-Windows-Kernel-Process | ProcessStart | 1 | 1 | 6 | OK |
| Microsoft-Windows-Kernel-Process | ProcessStop | 2 | 1 | 15 | OK |
| Microsoft-Windows-Kernel-Process | ThreadStart | 3 | 1 | 10 | OK |
| Microsoft-Windows-Kernel-Process | ThreadStop | 4 | 1 | 11 | OK |
| Microsoft-Windows-Kernel-File | FileCreate | 12 | 0 | 7 | OK |
| Microsoft-Windows-Kernel-File | FileCleanup | 13 | 0 | 4 | OK |
| Microsoft-Windows-Kernel-File | FileClose | 14 | 0 | 4 | OK |
| Microsoft-Windows-Kernel-File | FileRead | 15 | 0 | 7 | OK |
| Microsoft-Windows-Kernel-File | FileWrite | 16 | 0 | 7 | OK |
| Microsoft-Windows-Kernel-File | FileNameCreate | 10 | 0 | 2 | OK |
| Microsoft-Windows-Kernel-File | FileNameDelete | 11 | 0 | 2 | OK |
| Microsoft-Windows-Kernel-File | OperationEnd | 24 | 0 | 3 | OK |

### Experiment 4: Version Probing

For FileIo kernel provider (all versions 0-5), all event IDs tested return `ERROR_NOT_FOUND`. This confirms the issue is with provider lookup, not version matching.

---

## Conclusions

### 1. Fabrication is Possible for Manifest-Based Providers

For providers registered in the WINEVT\Publishers registry (manifest-based), you can fabricate an `EVENT_RECORD` with **only two fields**:
- `EventHeader.ProviderId` (the provider GUID)
- `EventHeader.EventDescriptor` (Id, Version, Opcode, Task, Level, Keyword)

All other fields (`Flags`, `EventProperty`, `BufferContext`, `UserData`, etc.) can be zeroed. TDH will still find and return the schema.

### 2. Fabrication Does NOT Work for Legacy Kernel Providers

Legacy kernel providers (FileIo, DiskIo, etc.) use the WMI MOF repository for metadata, not the WINEVT registry. `TdhGetEventInformation` cannot locate their schemas from fabricated records.

**Workaround**: You must capture at least one real event from the provider to get the schema. The existing `tdh-enumerator` tool does this - it starts a kernel trace, captures events, and calls TDH on the real event records.

### 3. Minimum Fields for Manifest-Based Fabrication

| Field | Required? | Notes |
|-------|-----------|-------|
| `ProviderId` | **YES** | Primary lookup key |
| `EventDescriptor.Id` | **YES** | Event type identifier |
| `EventDescriptor.Version` | **YES** | Schema version |
| `EventDescriptor.Opcode` | No | Can be zero |
| `EventDescriptor.Task` | No | Can be zero |
| `EventDescriptor.Keyword` | No | Can be zero |
| `EventDescriptor.Level` | No | Can be zero |
| `Flags` | No | Can be zero (for manifest providers) |
| `EventProperty` | No | Can be zero |
| `BufferContext` | No | Can be zero |
| `UserData` | No | Can be null/empty |
| `ExtendedData` | No | Can be null/zero |

### 4. Why Legacy Kernel Providers Can't Be Fabricated

The FileIo kernel provider (`90cbdc39-4a3e-11d1-84f4-0000f80464e3`) is a legacy ETW provider that:
- Uses MOF (Managed Object Format) class definitions, not XML manifests
- Has metadata in the WMI repository (`%SystemRoot%\System32\wbem\`), not in the WINEVT registry
- `TdhGetEventInformation` does not search the WMI repository when given a fabricated record
- The API apparently relies on ETW session context (BufferContext, LoggerId) or other session state to locate MOF metadata

---

## Running the Experiments

### Prerequisites

- Windows 10/11 with Rust toolchain
- Administrator privileges (for experiments 1, 3, 5, 6, 7)

### Build

```bash
cargo build -p tdh-fabricate
```

### Run (Without Admin - Experiments 2 & 4 Only)

```bash
cargo run -p tdh-fabricate
```

### Run (With Admin - All Experiments)

```powershell
# From an elevated PowerShell:
cargo run -p tdh-fabricate
```

**UAC Note**: Windows User Account Control will prompt for administrator privileges when running kernel ETW tracing. If running from a non-elevated terminal, you may need to:
1. Open an elevated Command Prompt or PowerShell
2. Navigate to the project directory
3. Run `cargo run -p tdh-fabricate`

The program gracefully handles non-admin by skipping capture-based experiments.

---

## Adapting to Another Provider

The `tdh-fabricate` experiment is designed to work with the FileIo scope by default. To test with another provider:

### Step 1: Identify the Provider Type

| Provider Type | How to Identify | Can Fabricate? |
|---------------|-----------------|----------------|
| Manifest-based (user-mode) | Registered in `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WINEVT\Publishers\{GUID}` | **YES** |
| Legacy kernel (MOF) | Uses EnableFlags, not keyword masks | **NO** |

### Step 2: Find the Provider GUID

```powershell
# List all registered providers:
Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WINEVT\Publishers" | 
    ForEach-Object { $_.PSChildName }
```

### Step 3: Find Event IDs and Versions

For manifest providers, check the manifest XML:
```powershell
# Example: Find Microsoft-Windows-Kernel-Process events
wevtutil gp Microsoft-Windows-Kernel-Process /f:xml
```

For legacy kernel providers, check the MOF:
```powershell
# Check MOF definitions
Get-Content "$env:SystemRoot\System32\wbem\mof\*.mof" | Select-String "class"
```

### Step 4: Modify the Experiment

In `experiments/tdh-fabricate/src/fabricate.rs`, add your provider GUID:

```rust
pub const MY_PROVIDER_GUID: GUID = GUID {
    data1: 0xXXXXXXXX,
    data2: 0xXXXX,
    data3: 0xXXXX,
    data4: [0xXX, 0xXX, 0xXX, 0xXX, 0xXX, 0xXX, 0xXX, 0xXX],
};
```

In `experiments/tdh-fabricate/src/experiments.rs`, add test cases:

```rust
let my_test_cases: Vec<(&str, u16, u8, u8)> = vec![
    ("MyEvent_v1", 1, 1, 0),
    ("MyEvent_v2", 1, 2, 0),
];
```

### Step 5: Run

```bash
cargo run -p tdh-fabricate
```

---

## Files

| File | Purpose |
|------|---------|
| `experiments/tdh-fabricate/Cargo.toml` | Dependencies (ferrisetw, windows, hex) |
| `experiments/tdh-fabricate/src/main.rs` | Experiment orchestrator |
| `experiments/tdh-fabricate/src/fabricate.rs` | Core `FabricatedRecord` type for creating/modifying EVENT_RECORDs |
| `experiments/tdh-fabricate/src/tdh_helpers.rs` | TDH API wrapper functions |
| `experiments/tdh-fabricate/src/experiments.rs` | All 7 experiment implementations |

---

## Related Work

- **krabsetw issue #212**: Microsoft engineer confirmed that modifying `EventDescriptor.Version` in a real EVENT_RECORD changes which schema TDH returns. This proves you can modify fields in real records, but fabrication from scratch requires the provider to be in the WINEVT registry.
- **siliceum blog**: TDH sometimes returns incorrect schemas for certain event versions; PerfView and WPA have hand-patched parsers as workarounds.
- **tdh-enumerator**: The existing experiment in this workspace that successfully enumerates all event types by capturing real events and calling TDH on them.
