# Generating ETW Provider Code from Manifest XML

## 1. Goal

Eliminate the manual `etw_provider! { ... }` block in provider files (e.g. `kernel_file.rs`) by generating it directly from the provider's manifest XML (`Microsoft-Windows-Kernel-File.xml`). The generated output feeds into the existing `etw_provider!` macro, which remains the codegen backend.

## 2. Architecture

```
Manifest XML                    Generated .rs                    etw_provider! macro
  (event defs,       ───►       etw_provider! { ... }    ───►    Rust code
   templates,                                              (structs, enum,
   keywords)                                                try_parse, etc.)
```

Two-phases:

- **Phase 1** (new): Parse XML → generate `etw_provider! { ... }` source text.
- **Phase 2** (existing): `etw_provider!` macro → structs, enum, `try_parse`, `build_provider`.

## 3. Input Types → Rust Types Mapping

The manifest's `<data name="..." inType="win:XXX"/>` must map to Rust field types that ferrisetw's `Parser::try_parse<T>()` supports.

| win:InType         | Rust type                 | Notes                                                       |
|---------------------|---------------------------|-------------------------------------------------------------|
| `win:Pointer`       | `u64`                     | Managed by `find_property_size` with `pointer_size()`       |
| `win:UnicodeString` | `String`                  | UTF-16 → Rust String                                        |
| `win:AnsiString`    | `String`                  |                                                             |
| `win:Int8`          | `i8`                      |                                                             |
| `win:UInt8`         | `u8`                      |                                                             |
| `win:Int16`         | `i16`                     |                                                             |
| `win:UInt16`        | `u16`                     |                                                             |
| `win:Int32`         | `i32`                     |                                                             |
| `win:UInt32`        | `u32`                     |                                                             |
| `win:Int64`         | `i64`                     |                                                             |
| `win:UInt64`        | `u64`                     |                                                             |
| `win:Float`         | `f32`                     |                                                             |
| `win:Double`        | `f64`                     |                                                             |
| `win:Boolean`       | `bool`                    | 32-bit value, 0=false                                       |
| `win:GUID`          | `windows::core::GUID`     | 16 bytes                                                    |
| `win:FILETIME`      | `ferrisetw::parser::FileTime` | 8 bytes. Or `SystemTime` via conversion                 |
| `win:SYSTEMTIME`    | `SystemTime`              | 16 bytes                                                    |
| `win:SID`           | `String`                  | Converted via `ConvertSidToStringSid`                        |
| `win:Binary`        | `Vec<u8>`                 | Variable-size; size from `length` attr or sibling field     |
| `win:HexInt32`      | `u32`                     | Same underlying type as `win:UInt32`                         |
| `win:HexInt64`      | `u64`                     | Same underlying type as `win:UInt64`                         |

### Fields with dynamic length

Some fields reference another field's value for their length:

```xml
<data name="Verb" inType="win:AnsiString" length="_VerbLength"/>
```

The `length` attribute references a preceding `_VerbLength` (UInt16/UInt32) field. The generated code should note this but not change the field type — ferrisetw handles it at runtime via `tdh::property_size`.

## 4. Keyword / Mask Parsing

The manifest defines keywords and per-event keyword assignments:

```xml
<keyword name="KERNEL_FILE_KEYWORD_FILEIO" mask="0x20" />
<keyword name="KERNEL_FILE_KEYWORD_READ" mask="0x100" />

<event value="15" keywords="KERNEL_FILE_KEYWORD_FILEIO KERNEL_FILE_KEYWORD_READ" .../>
```

The tool must:

1. Parse `<keyword>` elements → map `name → mask`.
2. For each `<event>`, split the `keywords` attribute, look up masks, OR them → `mask = 0x120`.
3. Emit `mask = 0x120` in the `#[etw_event]` attribute.

### Combined mask for `.any()`

The tool computes the bitwise OR of all *non-skipped* events' masks and generates `.any(COMBINED)`. This is already handled by `etw_provider!` when all structs have `mask`.

## 5. Template / Property Parsing

The manifest defines templates with data fields:

```xml
<template tid="CreateArgs">
  <data name="Irp"           inType="win:Pointer" />
  <data name="ThreadId"      inType="win:Pointer" />
  <data name="FileObject"    inType="win:Pointer" />
  <data name="CreateOptions" inType="win:UInt32" />
  <data name="CreateAttributes" inType="win:UInt32" />
  <data name="ShareAccess"   inType="win:UInt32" />
  <data name="FileName"      inType="win:UnicodeString" />
</template>
```

Each `<data>` maps to:

```rust
#[etw_prop(name = "Irp")]
pub irp: u64,
```

The `#[etw_prop(name = "...")]` value is the `name` attribute (as it appears in the ETW schema). The Rust field name is a `snake_case` version of that name. Converting from PascalCase to snake_case (e.g. `CreateOptions` → `create_options`) via `heck` crate.

### Template reuse

Multiple events share the same template (e.g. Cleanup v0, Close v0, Flush v0 all use `CleanupArgs`). The tool must deduplicate — define the struct once, reference it in multiple enum variants. But currently `etw_provider!` defines one struct per variant. Two approaches:

**A. One struct per variant (current pattern)**: Generate a separate struct per event even if templates match. Simple, but duplicates code. Already works.

**B. Shared structs**: Generate the struct once, reference from multiple variants. More complex XML → code logic but less generated code.

Recommend **A** initially for simplicity and consistency with the existing pattern.

## 6. Implementation Options

### Option A: `build.rs` code generator

A build script that reads the manifest XML and writes a `.rs` file.

```
manifest.xml ──► build.rs ──► $OUT_DIR/provider_gen.rs
                               └── mod provider_gen { etw_provider! { ... } }
```

**Pros**:
- No XML parsing at proc-macro time (proc-macros can't easily depend on XML crates).
- Full access to Rust ecosystem (`quick-xml`, `serde`, `heck`).
- Generated file is human-readable and commitable.

**Cons**:
- Requires running `build.rs` (always triggers recompilation when XML changes).
- Generated file path is `$OUT_DIR` which is less discoverable.

### Option B: CLI tool `etw-codegen`

A separate binary in the workspace:

```
cargo run -p etw-codegen -- -i manifest.xml -o src/providers/generated.rs
```

**Pros**:
- Explicit user control over when to regenerate.
- Generated file is committed alongside source (reviewable, diffable in PRs).

**Cons**:
- Additional step in workflow (manual or in CI).

### Option C: Proc-macro that embeds the XML

Extend the existing `etw_provider!` macro (or create a sibling) to accept XML content:

```rust
etw_provider_from_xml! {
    include_str!("../../etw_manifests/Microsoft-Windows-Kernel-File.xml")
}
```

**Pros**:
- Zero external tooling; everything happens at compile time.
- Single macro invocation replaces both the XML-to-code step and codegen.

**Cons**:
- Proc-macro must parse XML (limited dependency ecosystem, more complex).
- `include_str!` embeds the XML → triggers recompile on any XML change (desirable).
- Proc-macro errors are harder to debug than a separate tool.

## 7. Recommended Approach

**Option C (proc-macro)** for small-to-medium manifests, with the following structure:

```
etw-macros/src/
  lib.rs              — existing derive(EtwEvent) and etw_provider!
  xml.rs              — new: parse manifest XML into an intermediate representation
  xml_to_provider.rs  — new: convert IR → EtwProviderInput, then call expand()
```

A new proc-macro entry point:

```rust
#[proc_macro]
pub fn etw_provider_from_xml(input: TokenStream) -> TokenStream {
    // input is a string literal (the XML content, via include_str!)
    // parse XML → build EtwProviderInput → call expand() → return TokenStream
}
```

Usage:

```rust
use crate::etw::etw_provider_from_xml;

etw_provider_from_xml! {
    include_str!("../../etw_manifests/Microsoft-Windows-Kernel-File.xml")
}
```

### Why proc-macro over build.rs?

1. **Single macro call** is the simplest user experience — no build script, no separate tool, no output file to manage.
2. **No committed generated file** — the XML is the source of truth.
3. **Already in the proc-macro crate** — no new dependency burden on the main crate.
4. **`include_str!` at the call site** lets the user decide which manifest to use and keeps the XML path relative to the provider file.

### Why not proc-macro?

1. **XML parsing in proc-macros** is not well-supported. `quick-xml` works (it's `no_std` compatible with `alloc`), but error messages are poor compared to a CLI tool.
2. **Proc-macro compile times** increase if XML parsing is complex.
3. **Testing XML parsing** from proc-macros requires `trybuild`-style integration tests.

### Fallback: build.rs

If proc-macro XML parsing proves too painful, a `build.rs` in `src/providers/` that calls the existing `etw_provider!` macro indirectly is also viable. The build script generates:

```rust
// $OUT_DIR/kernel_file_gen.rs
etw_provider! {
    #[etw_provider(name = "Microsoft-Windows-Kernel-File", guid = "...")]
    pub enum KernelFileEvent {
        #[etw_event(id = 10, version = 0, mask = 0x10)]
        pub struct NameCreateV0 {
            #[etw_prop(name = "FileKey")]
            pub file_key: u64,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }
        ...
    }
}
```

which is included via `include!`:

```rust
// src/providers/kernel_file.rs
include!(concat!(env!("OUT_DIR"), "/kernel_file_gen.rs"));
```

## 8. XML Parsing Details

### 8a. Required XML crates (`quick-xml`)

No schema validation needed — just read:

- `<provider name="..." guid="...">` → provider metadata
- `<keyword name="..." mask="0x..."/>` → keyword map
- `<task name="..." value="N"/>` → event ID lookup (optional, `value` attr on `<event>` is primary)
- `<event value="N" version="M" keywords="..." template="T"/>` → event def
- `<template tid="T"> <data name="..." inType="win:XXX"/> </template>` → field defs

### 8b. Edge cases

| Case | Handling |
|------|----------|
| No `version` attribute | `version = 0` (common in v0-only events) |
| Multiple keywords in one event | Split by space, OR masks |
| Template shared by multiple events | Generate one struct per event (no dedup) |
| `length` attribute on a data field | Note in comment, no Rust type change |
| No keywords attribute | Leave `mask` unset (partial mask → no `.any()`) |
| `count` attribute (array fields) | `Vec<T>` or `&[T]` slice |


## 9. Implementation Plan (Proc-macro approach)

### Step 1: Add XML parsing dependency

```toml
# etw-macros/Cargo.toml
[dependencies]
quick-xml = { version = "0.36", default-features = false, features = ["alloc"] }
```

### Step 2: Create `etw-macros/src/xml.rs`

Parse `Microsoft-Windows-Kernel-File.xml` into an intermediate representation:

```rust
struct Manifest {
    provider_name: String,
    provider_guid: String,
    keywords: HashMap<String, u64>,
    events: Vec<ManifestEvent>,
}

struct ManifestEvent {
    id: u16,
    version: Option<u8>,
    keywords: Vec<String>,   // -> mask = OR of keyword masks
    template_name: String,
    fields: Vec<ManifestField>,
}

struct ManifestField {
    name: String,        // e.g. "FileKey"
    in_type: String,     // e.g. "win:Pointer"
    length: Option<String>, // e.g. Some("_VerbLength")
}
```

### Step 3: Create `etw-macros/src/xml_to_provider.rs`

Convert `Manifest` → `EtwProviderInput` (the existing AST that `expand()` consumes).

Key logic:

- `win:Pointer` → `u64`
- `win:UnicodeString` → `String`
- `win:UInt32` → `u32`
- etc.
- PascalCase field name → `snake_case` via `heck` crate.

### Step 4: Register `etw_provider_from_xml!` proc-macro

```rust
#[proc_macro]
pub fn etw_provider_from_xml(input: TokenStream) -> TokenStream {
    let xml_str = parse_macro_input!(input as LitStr).value();
    let manifest = match parse_manifest(&xml_str) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error().into(),
    };
    let provider_input = manifest_to_provider_input(&manifest);
    match provider_input.expand() {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
```

## 10. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| XML schema varies across providers | Only target `Microsoft-Windows-Kernel-File` initially; test others later |
| Proc-macro adds sec to compile time for large manifests | Acceptable; manifests are small (<50 events) |
| `quick-xml` API changes | Pin version in Cargo.toml |
| MSVC `mc.exe` generates different manifests | Validate against actual `.xml` files from Windows SDK |
| Unmapped input types produce bad code | Emit compile_error for unknown types |

## 11. Future Extensions

- **CLI tool variant** for users who prefer build.rs or committed generated files.
- **`#[etw_skip]` by attribute** — specify which event IDs or names to skip in the XML source or as an extra argument.
- **Custom type overrides** — allow `#[etw_provider(type_overrides = "JsonData -> serde_json::Value")]`-like syntax.
- **Array fields** — handle `count` attribute on `<data>` for `Vec<T>` fields.
