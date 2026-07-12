# ETW Provider Code Generation Instructions

This document explains how to generate a Rust provider file (like `kernel_process.rs`) from an ETW manifest XML file (like `Microsoft-Windows-Kernel-Process.xml`), matching the pattern used in the `kernel_file.rs` reference implementation.

---

## Overview

The generate code uses `etw_provider!` macro (from `etw-macros` crate) + `#[derive(EtwEvent)]` to produce types that can parse ETW events from the given provider.

**Input:** An ETW instrumentation manifest XML file.
**Output:** A `.rs` file in `src/providers/` containing:
- An `enum` listing all known events (each variant wraps a struct)
- Per-event structs with `#[etw_event]` + `#[etw_prop]` attributes

---

## Mapping Steps

### 1. Provider metadata

From the `<provider>` element:

```xml
<provider name="Microsoft-Windows-Kernel-Process" guid="{22fb2cd6-..." ...>
```

| XML attribute | Rust code |
|---|---|
| `name` | `name = "..."` in `#[etw_provider(name = "...", guid = "...")]` |
| `guid` (strip braces) | `guid = "..."` in `#[etw_provider(...)]` |

The enum name is derived from the provider name by removing the `Microsoft-Windows-` prefix and converting to PascalCase with the `Kernel` prefix kept, then adding `Event` suffix. Example: `Microsoft-Windows-Kernel-Process` → `KernelProcessEvent`.

Reference: `kernel_file.rs:6`:
```rust
#[etw_provider(name = "Microsoft-Windows-Kernel-File", guid = "EDD08927-9CC4-4E65-B970-C2560FB5C289")]
pub enum KernelFileEvent {
```

### 2. Keyword mask map

Collect all `<keyword>` elements and their `mask` hex values:

```xml
<keyword name="KERNEL_FILE_KEYWORD_FILEIO" mask="0x20" />
```

Build a map: keyword name → mask value (as u64).

### 3. Events (`<event>` elements)

Each `<event>` produces one struct variant. For each:

| XML attribute | Mapping |
|---|---|
| `value` | `id = ...` in `#[etw_event]` |
| `version` | `version = ...` in `#[etw_event]` |
| `keywords` | Split on whitespace, look up each in keyword mask map, OR all masks → `mask = 0x...` |

**Struct naming:**
1. Take the `symbol` attribute
2. Strip any trailing `_V\d+` suffix (e.g., `Create_V1` → `Create`)
3. Append `V{version}` at the end (e.g., `Create` + version 1 → `CreateV1`)

Reference: `kernel_file.rs:27-61`:
```rust
#[etw_event(id = 12, version = 0, mask = 0xa0)]
pub struct CreateV0 { ... }

#[etw_event(id = 12, version = 1, mask = 0xa0)]
pub struct CreateV1 { ... }
```

### 4. Template data fields (`<template>` + `<data>` elements)

The `<event>` element's `template` attribute references a `<template tid="...">`. Each `<data>` child becomes a struct field.

#### 4a. Field name

Take the `name` attribute from `<data>` and convert to `snake_case`.

| PascalCase | snake_case |
|---|---|
| `FileKey` | `file_key` |
| `ProcessID` | `process_id` |
| `CreateTime` | `create_time` |
| `ImageName` | `image_name` |
| `StackBase` | `stack_base` |
| `IOSize` | `io_size` |
| `IOFlags` | `io_flags` |

#### 4b. Type mapping (`inType` → Rust type + attribute)

| `inType` | Rust field type | `#[etw_prop]` annotation |
|---|---|---|
| `win:Pointer` | `usize` | `#[etw_prop(name = "...", parse_as = ferrisetw::parser::Pointer)]` |
| `win:UInt8` | `u8` | `#[etw_prop(name = "...")]` |
| `win:UInt16` | `u16` | `#[etw_prop(name = "...")]` |
| `win:UInt32` | `u32` | `#[etw_prop(name = "...")]` |
| `win:UInt64` | `u64` | `#[etw_prop(name = "...")]` |
| `win:UnicodeString` | `String` | `#[etw_prop(name = "...")]` |
| `win:AnsiString` | `String` | `#[etw_prop(name = "...")]` |
| `win:FILETIME` | `i64` (unix timestamp ms) | `#[etw_prop(name = "...", parse_as = ferrisetw::parser::FileTime)]` |
| `win:GUID` | `windows::core::GUID` | `#[etw_prop(name = "...")]` (no `parse_as` needed) |
| `win:SID` | `String` (SDDL format) | `#[etw_prop(name = "...")]` (no `parse_as` needed) |

**Rationale for `parse_as`:**
- `Pointer`: The `Pointer` type handles 32-bit vs 64-bit automatically. The field type is `usize` and `EtwPropConvert<Pointer> for usize` converts it.
- `FileTime`: The `FileTime` parser returns `ferrisetw::parser::FileTime`. The field type is `i64` (unix timestamp in milliseconds), converted via `EtwPropConvert<FileTime> for i64`.
- `GUID`: The parser directly supports `try_parse::<windows::core::GUID>()`, so no `parse_as` needed.
- `SID`: The parser's `try_parse::<String>()` handles SID → SDDL string conversion.

**For types without `parse_as` (primitives, String, GUID):**
The `#[derive(EtwEvent)]` macro generates: `field: parser.try_parse::<FieldType>("PropertyName")?`

**For types with `parse_as`:**
The macro generates:
```rust
let __val: IntermediateType = parser.try_parse("PropertyName")?;
field: <FieldType as EtwPropConvert<IntermediateType>>::convert(__val)
```

#### 4c. Reference existing implementations in `src/etw.rs`

The file `src/etw.rs` already has:
```rust
impl EtwPropConvert<ferrisetw::parser::Pointer> for usize {
    fn convert(value: ferrisetw::parser::Pointer) -> Self { *value }
}
```

For `FileTime` → `i64`, add:
```rust
impl EtwPropConvert<ferrisetw::parser::FileTime> for i64 {
    fn convert(value: ferrisetw::parser::FileTime) -> Self {
        value.as_unix_timestamp()
    }
}
```

### 5. Struct field ordering

Fields must appear in the **exact order** they appear in the `<template>` XML element. The ETW event layout is positional, not named.

### 6. Events without template

If an `<event>` element has no `template` attribute, it has no data payload. Skip it (do not include in the generated code).

### 7. Module registration

Add `pub mod kernel_process;` to `src/providers/mod.rs`.

---

## Output file structure

```rust
#![allow(dead_code)]

use crate::etw::etw_provider;

etw_provider! {
    #[etw_provider(name = "Microsoft-Windows-Kernel-Process", guid = "22FB2CD6-0E7B-422B-A0C7-2FAD1FD0E716")]
    pub enum KernelProcessEvent {
        // ── Event ID 1 (v0, v1, v2, v3) ──────────────────────────
        #[etw_event(id = 1, version = 0, mask = 0x10)]
        pub struct SomeEventV0 {
            // fields...
        }

        // ... more events ...
    }
}
```

## Derive macro behavior

The `#[derive(EtwEvent)]` (applied inside `etw_provider!`) generates an `impl EtwEventParse` that:
1. Iterates over struct fields in declaration order
2. For each `#[etw_prop]`, calls `parser.try_parse("PropertyName")` (possibly via `parse_as` intermediate + `EtwPropConvert`)
3. Returns the populated struct

The `etw_provider!` macro also generates:
- Constants `PROVIDER_NAME` and `PROVIDER_GUID`
- An enum with `try_parse()` method that matches on `(event_id, version)`
- A `build_provider()` helper method

## Debugging

If `cargo check` fails:
- Ensure all `EtwPropConvert` impls exist in `src/etw.rs` for any `parse_as` types used
- For `windows::core::GUID` fields, ensure the `windows` crate is in `Cargo.toml` (it should be, as ferrisetw depends on it)
- Verify field order matches XML template order exactly
- Check that struct names don't clash (e.g., `CreateV0` vs `CreateNewFileV0` is fine because they have different IDs)
