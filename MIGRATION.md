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
3. A struct with a **single** `#[etw_event(...)]` should also specify `name` explicitly
   (the macro allows omitting it, defaulting to the struct name, but provider source code
   should always provide it).

**Important:** The `name` on each `#[etw_event(...)]` preserves the existing event struct
names. Migration does NOT rename any events — it only wraps them in a template struct.

## Background: Kernel vs User Tracing

ETW has two fundamentally different tracing models, and this affects how you think about
templates in the `etw_provider!` macro.

### Kernel Tracing (MOF Classes)

Kernel tracing uses **MOF (Managed Object Format) classes**. Each class defines an "event
type" with a set of fields. Different event IDs can share the same class (and therefore
the same fields).

Manifest files: `etw_manifests/fileio-mof/` — one `.md` file per class.

The version of a class comes from its parent. Parent classes have `EventVersion(x)` in
their MOF definition:
```
[Guid("{90cbdc39-...}"), EventVersion(0)]
class FileIo_V0 : MSNT_SystemTrace { };
```
Child classes inherit the version:
```
[EventType{0, 32}, EventTypeName{"Name", "FileCreate"}]
class FileIo_V1_Name : FileIo_V1 { ... }
```
Here `FileIo_V1_Name` extends `FileIo_V1` (which has `EventVersion(1)`), so
it's version 1. Meanwhile `FileIo_V0_Name : FileIo_V0` is version 0.

For example, in the FileIO kernel provider:
- Event type 0 (`Name`), 32 (`FileCreate`), 35 (`FileDelete`), 36 (`FileRundown`) all use
  the `FileIo_Name` class — they have the same fields (`FileObject`, `FileName`).
- Event type 65 (`Cleanup`), 66 (`Close`), 73 (`Flush`) all use the `FileIo_SimpleOp`
  class — same fields (`IrpPtr`, `TTID`, `FileObject`, `FileKey`).

**Template name for kernel tracing:** derived from the MOF class name with version suffix.
E.g., `FileIo_Name` V0 → `FileIoNameV0`, `FileIo_Name` V1 → `FileIoNameV1`. These are
different MOF classes even when the fields are the same. **Templates are generally NOT
shared between versions** — `FileIo_V0_Name` and `FileIo_V1_Name` are separate classes
that happen to have the same fields; they still get separate template structs.

### User Tracing (XML Manifest Templates)

User tracing uses **XML manifests** with `<templates>`. Each template has a `tid` attribute
and defines a set of data fields. Different events reference templates by name.

Manifest files: `etw_manifests/Microsoft-Windows-Kernel-File.xml`,
`etw_manifests/Microsoft-Windows-Kernel-Process.xml`

The version comes from the `version` attribute on each `<event>` tag (NOT from the
template). Templates are generally NOT shared between versions — always verify that two
events with different versions actually use templates with identical fields before grouping
them.

For example, in the `Microsoft-Windows-Kernel-File.xml` manifest:
- Events `NameCreate` (id=10, version=0) and `NameDelete` (id=11, version=0) both use
  template `NameCreateArgs` — same version, same template. Group them.
- Events `Cleanup` (id=13, version=0) and `Close` (id=14, version=0) both use template
  `CleanupArgs` — same version, same template. Group them.
- Events `Cleanup` (id=13, version=1) uses template `CleanupArgs_V1` — different version,
  different template (different fields). Separate template struct.

**Template name for user tracing:** derived from the XML template `tid` attribute with
version suffix. E.g., `NameCreateArgs` → `NameCreateArgs`, `CleanupArgs` → `CleanupArgsV0`,
`CreateArgs_V1` → `CreateArgsV1`.

### Key Difference

| Aspect | Kernel (MOF) | User (XML Manifest) |
|--------|--------------|---------------------|
| Template source | MOF class name | XML `<template tid="...">` |
| Multiple event IDs per template | Yes (e.g., 0, 32, 35, 36 all → `FileIo_Name`) | Yes (e.g., 10, 11 both → `NameCreateArgs`) |
| Different fields per version | Yes (V0, V1, V2 classes differ) | Yes (separate templates per version) |
| Template naming convention | `{Provider}_{Class}{Version}` | `{TemplateName}{Version}` |

---

## Migration Steps

### Step 1: Identify Template Groups

Look at the existing provider file and identify groups of events that share the same
fields. These are your templates.

**For kernel tracing:** Group events by MOF class. The manifest files in
`etw_manifests/fileio-mof/` (one `.md` file per class) show which event types share a
class. The class name is in the `## Syntax` block, e.g.:
```
[EventType{0, 32}, EventTypeName{"Name", "FileCreate"}]
class FileIo_V1_Name : FileIo_V1
```

**For user tracing:** Group events by XML template `tid`. The manifest XML files in
`etw_manifests/` (e.g. `Microsoft-Windows-Kernel-File.xml`) show the `<templates>` and
`<events>` sections. Each `<event>` tag has a `template` attribute referencing a `<template tid="...">`.

### Step 2: Determine Versions

#### Kernel Tracing (MOF Classes)

Versions come from the parent class hierarchy. Each parent class has an `EventVersion(x)`
attribute. Child classes inherit the version from their parent.

File paths: `etw_manifests/fileio-mof/`

1. Find the parent class file (e.g. `fileio-v0.md`, `fileio-v1.md`). The `EventVersion`
   is in the `## Syntax` block:
   ```
   [Guid("{90cbdc39-...}"), EventVersion(0)]
   class FileIo_V0 : MSNT_SystemTrace
   ```
   This means `FileIo_V0` is version 0.

2. Find the child class files that extend from it. The parent is specified in the
   `class ... : ParentClass` line:
   ```
   class FileIo_V0_Name : FileIo_V0
   ```
   This means `FileIo_V0_Name` is version 0 (inherited from `FileIo_V0`).

3. The template struct name includes the version from the parent: `FileIo_V0_Name` →
   `FileIoNameV0`.

**Important:** `FileIo_V0_Name` and `FileIo_V1_Name` are different MOF classes even
though they have the same fields. They belong to different versions (V0 vs V1). Each gets
its own template struct. Templates are generally NOT shared between versions — this applies
to both MOF classes and XML templates. Always verify that two events with different versions
actually use templates with identical fields before grouping them.

#### User Tracing (XML Manifest Templates)

Versions come from the `version` attribute on each `<event>` tag.

File paths: `etw_manifests/Microsoft-Windows-Kernel-File.xml`,
`etw_manifests/Microsoft-Windows-Kernel-Process.xml`

1. Look at the `<events>` section. Each `<event>` has `version="..."`:
   ```xml
   <event value="12" symbol="Create" version="0" ... template="CreateArgs" />
   <event value="12" symbol="Create_V1" version="1" ... template="CreateArgs_V1" />
   ```

2. The version is on the event, NOT on the template. Different events with the same
   `version` value and same `template` reference can share a template struct.

3. **Templates are generally NOT shared between versions.** Always verify that two events
   with different `version` values actually reference templates with the same fields before
   grouping them. Check the `<templates>` section to compare field lists.

   Example: `CleanupArgs` (v0) has `Irp, ThreadId, FileObject, FileKey` while
   `CleanupArgs_V1` (v1) has `Irp, FileObject, FileKey, IssuingThreadId` — different
   fields, so they are separate template structs (`CleanupArgsV0` and `CleanupArgsV1`).

   Counter-example: events 13 (Cleanup), 14 (Close), 21 (Flush) all reference
   `CleanupArgs` with `version="0"` — same template, same fields. Group them.

### Step 3: Determine Template Struct Name

The template struct name is derived from the type/template name with version suffix:

- **Kernel:** The MOF class name with version from the parent class. E.g.:
  - `FileIo_V0_Name` (parent `FileIo_V0`, EventVersion(0)) → `FileIoNameV0`
  - `FileIo_V1_Name` (parent `FileIo_V1`, EventVersion(1)) → `FileIoNameV1`
  - `FileIo_SimpleOp` (check which parent it extends) → `FileIoSimpleOpV0` or `V1` etc.
- **User:** The XML template `tid` with version from the event's `version` attribute. E.g.:
  - `NameCreateArgs` with event `version="0"` → `NameCreateArgs` (no V suffix needed if
    there's only one version)
  - `CreateArgs` with event `version="0"` → `CreateArgsV0`
  - `CreateArgs_V1` with event `version="1"` → `CreateArgsV1`
  - `CleanupArgs` with event `version="0"` → `CleanupArgsV0`
  - `CleanupArgs_V1` with event `version="1"` → `CleanupArgsV1`

### Step 4: Determine Event Struct Names

**The event struct names in the existing provider code are already correct.** The migration
does NOT change them — it only wraps them in a template struct. The `name` value on each
`#[etw_event(...)]` should match the existing struct name exactly.

For reference, the event struct name is derived from:
- The event's descriptive name (from the manifest's `EventTypeName` for MOF or `symbol`
  for XML)
- Plus the version suffix (from the event's version)

Always specify `name` explicitly on every `#[etw_event(...)]` — do not rely on the
default.

E.g., for kernel FileIO V0:
- Template `FileIoNameV0` contains events `NameV0` (id=0), `FileCreateV1` (id=32),
  `FileDeleteV2` (id=35), `FileRundownV2` (id=36)
- Template `FileIoCreateV2` contains event `CreateV2` (id=64)

E.g., for user tracing Microsoft-Windows-Kernel-File:
- Template `NameCreateArgs` (version=0) contains events `NameCreateV0` (id=10),
  `NameDeleteV0` (id=11)
- Template `CreateArgsV0` (version=0) contains event `CreateV0` (id=12)
- Template `CreateArgsV1` (version=1) contains event `CreateV1` (id=12)

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

Even when a template has only one event, always specify `name` explicitly.

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

#### After (new syntax — always explicit name)
```rust
#[etw_event(name = "CreateV0", id = 12, version = 0, keyword_mask = masks::KERNEL_FILE_KEYWORD_FILEIO | masks::KERNEL_FILE_KEYWORD_CREATE)]
pub struct CreateArgsV0 {
    #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
    pub irp: usize,
    #[etw_prop(name = "ThreadId", parse_as = ferrisetw::parser::Pointer)]
    pub thread_id: usize,
    // ...
}
```

The template name is `CreateArgsV0` (from the MOF class), and the event name is `CreateV0`.
The expanded struct is `CreateV0` with `TEMPLATE_NAME = "CreateArgsV0"`.

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

1. **Always specify `name` on `etw_event`:** Provider source code should always provide
   `name = "..."` on every `#[etw_event(...)]` attribute. The macro allows omitting it
   (defaulting to the struct name), but this is not the intended usage for providers.
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
   Verify that events grouped together actually have the same fields.
2. Group events by template (same fields, same version = same template).
3. For each group:
   a. Determine the template struct name (see naming conventions above).
   b. Create a single struct with the shared fields (this is the template).
   c. Add `#[etw_event(name = "...", ...)]` for each event, with explicit `name`.
      The `name` value must match the existing event struct name exactly.
4. For events with unique fields (no sharing), still wrap in a template struct and
   always specify `name` explicitly.
5. Verify all events compile and the Debug output includes the template name.
6. **Event struct names do NOT change.** The `name` on `etw_event` preserves them. Only
   the template struct name is new.
