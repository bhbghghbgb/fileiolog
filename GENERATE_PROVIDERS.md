# Instructions: Generating Provider Event Structs from ETW Manifests

## Overview

Generate Rust source files under `src/providers/` from ETW manifest files in `etw_manifests/`. Each provider gets its own file containing event structs defined via the `etw_provider!` proc macro.

There are **two ETW session types** that dictate different code patterns:

| Property | Kernel Session (`kind = "kernel"`) | User Session (`kind = "user"`) |
|---|---|---|
| Enable mechanism | `enable_flag` (EVENT_TRACE_FLAG_*) | `keyword_mask` (provider keyword masks) |
| Group mask support | Yes (`group_mask` for PERFINFO_GROUPMASK bits) | No |
| Provider builder | `Provider::kernel(...)` | `Provider::by_guid(...)` with `.add_filter()` |
| Keyword/flag module | `flags` | `masks` |

## Manifest Formats

### MOF Format (`.mof`)
- Older WMI Managed Object Format
- Classes inherit from a parent version class (e.g., `FileIo_V2`, `FileIo`, `FileIo_V0`)
- Each event class has a `dynamic: ToInstance` annotation with `EventType{...}`, `EventVersion(N)`, `Description`, `EventTypeName{...}`
- Fields annotated with `[WmiDataId(N)]`, type (`uint32`, `uint64`, `pointer`, `string`), `Description`
- Version hierarchy: `EventVersion(0)` = oldest, higher numbers = newer

### XML Format (`.xml`)
- Standard Windows Instrumentation Manifest
- `<provider>` contains `<keywords>`, `<tasks>`, `<events>`, `<templates>`
- Each `<event>` has `value` (event ID), `symbol`, `version`, `task`, `keywords` (space-separated), `template` (references a `<template>`)
- Templates define the data layout with `<data name="..." inType="win:..." />` fields
- Versioned templates use `_V1`, `_V2` suffixes

## File Structure

Each provider file must follow this exact structure:

```rust
#![allow(dead_code)]

use crate::etw::etw_provider;

pub mod masks {  // or `pub mod flags` for kernel providers
    // keyword/flag constants from the manifest
}

etw_provider! {
    #[etw_provider(kind = "kernel"|"user", name = "...", guid = "...")]
    pub enum ProviderEventEnumName {
        // event structs...
    }
}
```

### File naming convention
- Kernel session providers: `kernel_trace_<provider_suffix>.rs` (e.g., `kernel_trace_fileio.rs`)
- User session providers: `user_trace_<provider_suffix>.rs` (e.g., `user_trace_kernel_file.rs`)
- Provider suffix is a lowercase abbreviation of the provider name

### Enum naming convention
- Kernel: `KernelTrace<PascalSuffix>Event` (e.g., `KernelTraceFileIoEvent`)
- User: `UserTrace<PascalSuffix>Event` (e.g., `UserTraceKernelFileEvent`)

### Mod registration
Add the new module to `src/providers/mod.rs`:
```rust
pub mod <filename_without_extension>;
```

## Manifest-to-Code Mapping

### Provider Attributes

| Manifest | Code |
|---|---|
| `<provider name="X" guid="{Y}">` | `#[etw_provider(kind = "user", name = "X", guid = "Y")]` |
| `Guid("{Y}")` in MOF class header | `#[etw_provider(kind = "kernel", guid = "Y")]` |
| Provider GUID (no dashes in code) | Remove `{}` braces, keep dashes in the GUID string |

### Keywords / Flags Module

**XML manifests**: Extract `<keyword name="..." mask="0x..." />` entries.

**MOF manifests**: Keywords are implicit from `EVENT_TRACE_FLAG_*` values. Define them manually based on Windows SDK documentation.

For **user** providers, the module is called `masks` and constants are `u64`:
```rust
pub mod masks {
    pub const KEYWORD_NAME: u64 = 0xHEX;
}
```

For **kernel** providers, the module is called `flags` and constants are `u32`:
```rust
pub mod flags {
    pub const EVENT_TRACE_FLAG_NAME: u32 = 0xHEX;
    // Extended PERFINFO_GROUPMASK bits for minifilter events:
    pub const PERF_FLT_IO_INIT: u32 = 0x80080000;
}
```

### Event Structs

#### Struct naming
- Use the manifest **template name** as the base, converted to PascalCase
- Append version suffix if versioned: `<TemplatePascal>V<N>` (e.g., `CreateArgsV0`, `CreateArgsV1`)
- If the struct is shared by multiple events with the same template, use the template name directly
- If a template name has no version suffix in the manifest (e.g., `CreateArgs`), and only one version exists, omit the V suffix

#### Struct grouping rules
- **One struct per unique template+version combination**. Multiple events sharing the same template and version map to one struct with multiple `#[etw_event]` attributes.
- If two different events share the same template at the same version, they get `#[etw_event]` on the same struct.
- If events share a template at different versions, each version gets its own struct.

#### `#[etw_event]` attributes

Each `#[etw_event]` must specify:
- `name = "EventPascalNameV<N>"` - derived from the manifest event symbol
- `id = <event_value>` - the `value` from `<event>` (XML) or `EventType` (MOF)
- `version = <version>` - the `version` from `<event>` (XML) or `EventVersion` (MOF class)

Plus one of:
- `enable_flag = flags::FLAG_NAME` (kernel providers) - for events accessible via EnableFlags
- `group_mask = flags::PERF_...` (kernel providers) - for events only accessible via PERFINFO_GROUPMASK
- `keyword_mask = masks::KEYWORD_NAME` (user providers) - can be a `|` combination of multiple keywords

**Event name in `#[etw_event]`**: Derived from the manifest event symbol or EventTypeName, converted to PascalCase with version suffix. Examples:
- XML symbol `NameCreate` + version 0 → `name = "NameCreateV0"`
- MOF `EventTypeName("Name")` at version 0 → `name = "NameV0"`
- XML symbol `Create_V1` → strip `_V1` suffix, use version attribute: `name = "CreateV1"`

#### `#[etw_prop]` attributes for fields

Fields are derived from the template's `<data>` elements (XML) or `WmiDataId`-annotated fields (MOF).

Each `#[etw_prop]` must specify:
- `name = "FieldName"` - **exactly** the manifest field name (e.g., `FileObject`, `IrpPtr`, `ThreadId`)

Plus one of (for non-default types):
- `parse_as = ferrisetw::parser::Pointer` - for `win:Pointer` / `pointer` types (Rust type: `usize`)
- `parse_as = ferrisetw::native::time::FileTime` - for `win:FILETIME` types (Rust type: `time::OffsetDateTime`)
- No `parse_as` - for `win:UInt32`/`UInt64`/`UInt16`/`UInt8`/`UnicodeString`/`AnsiString`/`GUID`/`SID`

**String fields**: `win:UnicodeString` and `win:AnsiString` both map to `String` with no `parse_as`.

### Type Mapping Table

| Manifest Type | Rust Type | `parse_as` |
|---|---|---|
| `win:Pointer` / `pointer` | `usize` | `ferrisetw::parser::Pointer` |
| `win:UInt32` / `uint32` | `u32` | (none) |
| `win:UInt64` / `uint64` | `u64` | (none) |
| `win:UInt16` | `u16` | (none) |
| `win:UInt8` | `u8` | (none) |
| `win:FILETIME` / `extension("WmiTime")` | `time::OffsetDateTime` | `ferrisetw::native::time::FileTime` |
| `win:UnicodeString` / `string` with `format("w")` | `String` | (none) |
| `win:AnsiString` | `String` | (none) |
| `win:GUID` | `windows::core::GUID` | (none) |
| `win:SID` | `String` | (none) |
| `extension("SizeT")` | `usize` | `ferrisetw::parser::Pointer` |

### Field Order

**Field order in the struct MUST exactly match the order in the manifest template.** The ETW parser reads fields sequentially. Do not reorder.

For XML: follow the order of `<data>` elements within the `<template>`.
For MOF: follow the `[WmiDataId(N)]` order (1, 2, 3, ...).

### Struct field naming
- Convert manifest field names from PascalCase to snake_case
- Examples: `FileObject` → `file_object`, `IrpPtr` → `irp_ptr`, `ThreadId` → `thread_id`, `IOSize` → `io_size`, `TTID` → `ttid`

### Comments

Each event struct must have a preceding comment block:

```rust
// ── <TemplateName>V<N> (v=<version>) ──────────────────────────
// XML template: <TemplateTid>
// Events: <EventSymbol1> (id=<N>), <EventSymbol2> (id=<M>)
```

For MOF-based providers:
```rust
// ── <ClassName> V<N> ────────────────────────────────────────
// Class: <ClassName> (EventVersion(<N>), EventType{<ids>})
```

Version section separators:
```rust
// ══════════════════════════════════════════════════════
// ProviderName V<N> (EventVersion <N>) — description
// ══════════════════════════════════════════════════════
```

### Empty structs

If an event has no template (no data fields), use an empty struct:
```rust
#[etw_event(name = "...", id = N, version = 0, keyword_mask = masks::...)]
pub struct EmptyStructName {}
```

### Custom impl blocks

If a struct needs custom methods, add an `impl { }` block inside the struct definition:
```rust
pub struct SomeEvent {
    #[etw_prop(name = "Field", parse_as = ferrisetw::parser::Pointer)]
    pub field: usize,

    impl {
        pub fn custom_method(&self) -> ReturnType {
            // ...
        }
    }
}
```

### Standalone types

If a struct needs an associated enum or type for custom logic, define it **outside** the `etw_provider!` macro but in the same file:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SomeKind {
    VariantA,
    VariantB,
}
```

## Generation Algorithm

### Step 1: Parse the manifest

For XML manifests:
1. Read `<provider name="..." guid="...">`
2. Extract all `<keyword>` entries for the masks/flags module
3. Extract all `<template>` entries — these define the data layouts
4. Extract all `<event>` entries — these define which events map to which templates

For MOF manifests:
1. Read the class hierarchy to determine version groups
2. Extract each event class and its `EventType{...}` values
3. Extract field definitions in `WmiDataId` order
4. Determine the provider GUID from the top-level class annotation

### Step 2: Determine session type

- If the manifest uses `EVENT_TRACE_FLAG_*` constants or inherits from `MSNT_SystemTrace` → **kernel** session
- If the manifest uses keyword masks from `<keywords>` → **user** session

### Step 3: Group events by template+version

Group events that share the same template at the same version into one struct. Events at different versions get separate structs.

### Step 4: Generate code

For each template+version group:
1. Create the struct with fields matching the template in order
2. Add `#[etw_event]` for each event in the group
3. Map types according to the type mapping table
4. Add `parse_as` attributes where needed
5. Write the comment block

### Step 5: Generate masks/flags module

Extract keyword/flag constants from the manifest and generate the module.

### Step 6: Generate provider macro invocation

Wrap everything in `etw_provider!` with the correct provider attributes.

### Step 7: Register in mod.rs

Add `pub mod <file_name>;` to `src/providers/mod.rs`.

## Concrete Example: Mapping XML to Code

Given this XML template:
```xml
<template tid="CreateArgs">
    <data name="Irp" inType="win:Pointer" />
    <data name="ThreadId" inType="win:Pointer" />
    <data name="FileObject" inType="win:Pointer" />
    <data name="CreateOptions" inType="win:UInt32" />
    <data name="CreateAttributes" inType="win:UInt32" />
    <data name="ShareAccess" inType="win:UInt32" />
    <data name="FileName" inType="win:UnicodeString" />
</template>
```

And these events using it:
```xml
<event value="12" symbol="Create" version="0" template="CreateArgs" keywords="KERNEL_FILE_KEYWORD_FILEIO KERNEL_FILE_KEYWORD_CREATE" />
<event value="30" symbol="CreateNewFile" version="0" template="CreateArgs" keywords="KERNEL_FILE_KEYWORD_CREATE_NEW_FILE" />
```

Generated code:
```rust
#[etw_event(name = "CreateV0", id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
#[etw_event(name = "CreateNewFileV0", id = 30, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_CREATE_NEW_FILE)]
pub struct CreateArgsV0 {
    #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
    pub irp: usize,
    #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
    pub thread_id: usize,
    #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
    pub file_object: usize,
    #[etw_prop(name = "CreateOptions")]
    pub create_options: u32,
    #[etw_prop(name = "CreateAttributes")]
    pub create_attributes: u32,
    #[etw_prop(name = "ShareAccess")]
    pub share_access: u32,
    #[etw_prop(name = "FileName")]
    pub file_name: String,
}
```

## Checklist

- [ ] File starts with `#![allow(dead_code)]`
- [ ] `use crate::etw::etw_provider;` is imported
- [ ] Masks/flags module is defined before `etw_provider!`
- [ ] Provider `kind` matches session type (kernel vs user)
- [ ] Provider `name` and `guid` match the manifest exactly
- [ ] Each struct has fields in exact manifest template order
- [ ] Each `#[etw_prop]` name matches the manifest field name exactly
- [ ] Pointer fields use `parse_as = ferrisetw::parser::Pointer` and type `usize`
- [ ] FILETIME fields use `parse_as = ferrisetw::native::time::FileTime` and type `time::OffsetDateTime`
- [ ] GUID fields use type `windows::core::GUID`
- [ ] String fields (UnicodeString/AnsiString) use type `String` with no `parse_as`
- [ ] Each `#[etw_event]` has `name`, `id`, `version`, and enable attribute (`keyword_mask` or `enable_flag`/`group_mask`)
- [ ] Event names follow `<PascalName>V<N>` pattern
- [ ] Field names are snake_case
- [ ] Comment blocks precede each struct
- [ ] Module is registered in `src/providers/mod.rs`
