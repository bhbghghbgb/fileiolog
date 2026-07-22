# Migration Guide: etw_provider! Template Struct Syntax

## Overview

The `etw_provider!` macro now supports **template structs** — a single struct definition
that represents a shared set of fields (a "template") from which multiple event structs
are generated. Each event gets its own `#[etw_event(...)]` attribute specifying the
event ID, version, and other metadata.

The key changes:
1. Each expanded struct now has a `TEMPLATE_NAME` constant and a custom `Debug` impl
   that shows `EventName(TemplateName) { ... }`.
2. A struct can have **multiple** `#[etw_event(...)]` attributes (each must specify `name`).
3. A struct with a **single** `#[etw_event(...)]` does not need `name` — it defaults to the
   struct name.

## Background: Kernel vs User Tracing

ETW has two fundamentally different tracing models, and this affects how you think about
templates in the `etw_provider!` macro.

### Kernel Tracing (MOF Classes)

Kernel tracing uses **MOF (Managed Object Format) classes**. Each class defines an "event
type" with a set of fields. Different event IDs can share the same class (and therefore
the same fields).

For example, in the FileIO kernel provider:
- Event type 0 (`Name`), 32 (`FileCreate`), 35 (`FileDelete`), 36 (`FileRundown`) all use
  the `FileIo_Name` class — they have the same fields (`FileObject`, `FileName`).
- Event type 65 (`Cleanup`), 66 (`Close`), 73 (`Flush`) all use the `FileIo_SimpleOp`
  class — same fields (`IrpPtr`, `TTID`, `FileObject`, `FileKey`).

**Template name for kernel tracing:** derived from the MOF class name (e.g., `FileIo_Name`
→ `FileIoNameV0`, `FileIo_SimpleOp` → `FileIoSimpleOpV0`).

### User Tracing (XML Manifest Templates)

User tracing uses **XML manifests** with `<templates>`. Each template has a `tid` attribute
and defines a set of data fields. Different events reference templates by name.

For example, in the `Microsoft-Windows-Kernel-File` manifest:
- Events `NameCreate` (id=10) and `NameDelete` (id=11) both use template `NameCreateArgs`.
- Events `Cleanup` (id=13), `Close` (id=14), `Flush` (id=21) all use template
  `CleanupArgs` (v0) or `CleanupArgs_V1` (v1).
- Events `SetInformation` (id=17), `SetDelete` (id=18), `Rename` (id=19),
  `QueryInformation` (id=22), `FSCTL` (id=23) all use template `SetInformationArgs`
  (v0) or `SetInformationArgs_V1` (v1).

**Template name for user tracing:** derived from the XML template `tid` attribute
(e.g., `NameCreateArgs` → `NameCreateArgs`, `CleanupArgs` → `CleanupArgsV0`,
`CreateArgs_V1` → `CreateArgsV1`).

### Key Difference

| Aspect | Kernel (MOF) | User (XML Manifest) |
|--------|--------------|---------------------|
| Template source | MOF class name | XML `<template tid="...">` |
| Multiple event IDs per template | Yes (e.g., 0, 32, 35, 36 all → `FileIo_Name`) | Yes (e.g., 10, 11 both → `NameCreateArgs`) |
| Different fields per version | Yes (V0, V1, V2 classes differ) | Yes (separate templates per version) |
| Template naming convention | `{Provider}_{Class}{Version}` | `{TemplateName}{Version}` (V1 suffix omitted if no V0) |

---

## Migration Steps

### Step 1: Identify Template Groups

Look at the existing provider file and identify groups of events that share the same
fields. These are your templates.

**For kernel tracing:** Group events by MOF class. The manifest file
(`etw_manifests/fileio-mof/`) shows which event types share a class.

**For user tracing:** Group events by XML template `tid`. The manifest XML
(`etw_manifests/Microsoft-Windows-Kernel-File.xml`) shows the `<templates>` section.

### Step 2: Determine Template Struct Name

The template struct name is derived from:
- **Kernel:** The MOF class name, with version suffix. E.g., `FileIo_Name` → `FileIoNameV0`,
  `FileIo_Name` (V1) → `FileIoNameV1`.
- **User:** The XML template `tid`, with version suffix. E.g., `NameCreateArgs` → `NameCreateArgs`,
  `CleanupArgs_V1` → `CleanupArgsV1`.

### Step 3: Determine Event Struct Names

For each event in the template group, the event struct name is derived from:
- The event's descriptive name (from the manifest's `EventTypeName` or `symbol`)
- Plus the version suffix

E.g., for kernel FileIO:
- Template `FileIoNameV0` contains events `NameV0` (id=0), `FileCreateV1` (id=32),
  `FileDeleteV2` (id=35), `FileRundownV2` (id=36)
- Template `FileIoCreateV2` contains event `CreateV2` (id=64)

---

## Before & After Examples

### Example 1: Kernel Tracing — Multiple Events Sharing a Template

#### Before (old syntax)
```rust
etw_provider! {
    #[etw_provider(kind = "kernel", guid = "90cbdc39-4a3e-11d1-84f4-0000f80464e3")]
    pub enum KernelTraceFileIoEvent {
        // Event type 0: Name (FileIo_Name class, V0)
        #[etw_event(id = 0, version = 0, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct NameV0 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // Event type 0: Name (FileIo_Name class, V1)
        #[etw_event(id = 0, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct NameV1 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // Event type 32: FileCreate (FileIo_Name class, V1)
        #[etw_event(id = 32, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileCreateV1 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }
    }
}
```

#### After (new syntax)
```rust
etw_provider! {
    #[etw_provider(kind = "kernel", guid = "90cbdc39-4a3e-11d1-84f4-0000f80464e3")]
    pub enum KernelTraceFileIoEvent {
        // Template: FileIo_Name MOF class, V0 fields
        // Multiple events share these fields → use multiple #[etw_event] with explicit names
        #[etw_event(name = "NameV0", id = 0, version = 0, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV1", id = 32, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV0 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // Template: FileIo_Name MOF class, V1 fields
        #[etw_event(name = "NameV1", id = 0, version = 1, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        #[etw_event(name = "FileCreateV2", id = 32, version = 2, enable_flag = flags::EVENT_TRACE_FLAG_DISK_FILE_IO)]
        pub struct FileIoNameV1 {
            #[etw_prop(name = "FileObject", parse_as = ferrisetw::parser::Pointer)]
            pub file_object: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }
    }
}
```

#### What expanded code looks like

For the `FileIoNameV0` template, the macro generates:

```rust
#[derive(Clone, EtwEvent)]
pub struct NameV0 {
    pub file_object: usize,
    pub file_name: String,
}

impl NameV0 {
    pub const TEMPLATE_NAME: &str = "FileIoNameV0";
}

impl Debug for NameV0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NameV0(FileIoNameV0)")
            .field("file_object", &self.file_object)
            .field("file_name", &self.file_name)
            .finish()
    }
}

#[derive(Clone, EtwEvent)]
pub struct FileCreateV1 {
    pub file_object: usize,
    pub file_name: String,
}

impl FileCreateV1 {
    pub const TEMPLATE_NAME: &str = "FileIoNameV0";
}

impl Debug for FileCreateV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileCreateV1(FileIoNameV0)")
            .field("file_object", &self.file_object)
            .field("file_name", &self.file_name)
            .finish()
    }
}
```

---

### Example 2: User Tracing — Multiple Events Sharing a Template

#### Before (old syntax)
```rust
etw_provider! {
    #[etw_provider(kind = "user", name = "Microsoft-Windows-Kernel-File", guid = "EDD08927-...")]
    pub enum UserTraceKernelFileEvent {
        // Event ID 10: NameCreate (template: NameCreateArgs)
        #[etw_event(id = 10, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameCreateV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // Event ID 11: NameDelete (template: NameCreateArgs — same fields!)
        #[etw_event(id = 11, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameDeleteV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }
    }
}
```

#### After (new syntax)
```rust
etw_provider! {
    #[etw_provider(kind = "user", name = "Microsoft-Windows-Kernel-File", guid = "EDD08927-...")]
    pub enum UserTraceKernelFileEvent {
        // Template: NameCreateArgs from XML manifest
        // Two events share this template → use multiple #[etw_event] with explicit names
        #[etw_event(name = "NameCreateV0", id = 10, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        #[etw_event(name = "NameDeleteV0", id = 11, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILENAME)]
        pub struct NameCreateArgs {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }
    }
}
```

---

### Example 3: Single Event (No Template Sharing)

When only one event uses a template, you can omit `name` and it defaults to the struct name.

#### Before (old syntax)
```rust
#[etw_event(id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
pub struct CreateV0 {
    #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
    pub irp: usize,
    #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
    pub thread_id: usize,
    // ...
}
```

#### After (new syntax — name omitted, defaults to struct name)
```rust
#[etw_event(id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
pub struct CreateV0 {
    #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
    pub irp: usize,
    #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
    pub thread_id: usize,
    // ...
}
```

The expanded struct is `CreateV0` with `TEMPLATE_NAME = "CreateV0"`.

---

## Naming Conventions

### Kernel Tracing (MOF Classes)

Template struct name = `{Provider}_{Class}{Version}` with underscores removed and
PascalCase applied.

| MOF Class | Version | Template Struct Name |
|-----------|---------|---------------------|
| `FileIo_Name` | V0 | `FileIoNameV0` |
| `FileIo_Name` | V1 | `FileIoNameV1` |
| `FileIo_Create` | V2 | `FileIoCreateV2` |
| `FileIo_SimpleOp` | V0 | `FileIoSimpleOpV0` |
| `FileIo_SimpleOp` | V1 | `FileIoSimpleOpV1` |
| `FileIo_ReadWrite` | V2 | `FileIoReadWriteV2` |
| `FileIo_Info` | V2 | `FileIoInfoV2` |
| `FileIo_DirEnum` | V2 | `FileIoDirEnumV2` |
| `FileIo_OpEnd` | V2 | `FileIoOpEndV2` |

Event struct name = descriptive name + version suffix.

| Event Type ID | MOF Class | Event Struct Name |
|---------------|-----------|-------------------|
| 0 | `FileIo_Name` | `NameV0` |
| 32 | `FileIo_Name` | `FileCreateV1` |
| 35 | `FileIo_Name` | `FileDeleteV2` |
| 64 | `FileIo_Create` | `CreateV2` |
| 65 | `FileIo_SimpleOp` | `CleanupV2` |
| 66 | `FileIo_SimpleOp` | `CloseV2` |

### User Tracing (XML Manifest Templates)

Template struct name = XML `tid` with underscores removed, PascalCase, version suffix.

| XML Template `tid` | Template Struct Name |
|--------------------|---------------------|
| `NameCreateArgs` | `NameCreateArgs` |
| `CreateArgs` | `CreateArgsV0` |
| `CreateArgs_V1` | `CreateArgsV1` |
| `CleanupArgs` | `CleanupArgsV0` |
| `CleanupArgs_V1` | `CleanupArgsV1` |
| `ReadArgs` | `ReadArgsV0` |
| `ReadArgs_V1` | `ReadArgsV1` |
| `SetInformationArgs` | `SetInformationArgsV0` |
| `SetInformationArgs_V1` | `SetInformationArgsV1` |

Event struct name = task/symbol name + version suffix.

| Event ID | XML Template | Event Struct Name |
|----------|--------------|-------------------|
| 10 | `NameCreateArgs` | `NameCreateV0` |
| 11 | `NameCreateArgs` | `NameDeleteV0` |
| 12 | `CreateArgs` | `CreateV0` |
| 12 | `CreateArgs_V1` | `CreateV1` |
| 13 | `CleanupArgs` | `CleanupV0` |
| 13 | `CleanupArgs_V1` | `CleanupV1` |

---

## Rules

1. **Single `etw_event` attr:** `name` is optional. Defaults to the struct name.
2. **Multiple `etw_event` attrs:** All must specify `name = "..."`. The macro rejects
   multiple `etw_event` attrs where any lacks `name`.
3. **Duplicate names:** The macro rejects two events with the same `name` in the same
   provider.
4. **No `etw_event` attr:** Error — every struct must have at least one `#[etw_event(...)]`.
5. **`TEMPLATE_NAME` constant:** Every expanded struct gets `pub const TEMPLATE_NAME: &str = "...";`
   containing the template struct name.
6. **Debug format:** `EventName(TemplateName) { field1: value1, ... }` instead of
   `EventName { field1: value1, ... }`.

---

## Checklist for Migrating Each Provider File

1. Read the corresponding manifest (MOF or XML) to identify which events share templates.
2. Group events by template (same fields = same template).
3. For each group:
   a. Determine the template struct name (see naming conventions above).
   b. Create a single struct with the shared fields.
   c. Add `#[etw_event(name = "...", ...)]` for each event, with explicit `name`.
4. For events with unique fields (no sharing), keep as single struct with optional `name`.
5. Verify all events compile and the Debug output includes the template name.
6. Update any code that references the old struct names (they should remain the same
   unless you change the `name` values).
