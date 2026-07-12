# ETW Provider Code Generation Instructions

This document describes the process for generating Rust provider code from ETW instrumentation manifest XML files.

## Prerequisites

1. An ETW instrumentation manifest XML file (e.g., `Microsoft-Windows-Kernel-File.xml`)
2. The `ferrisetw` library with `time_rs` feature enabled
3. The `etw-macros` proc macro crate (in `etw-macros/`)
4. The `fileiolog` crate's `etw.rs` module for `EtwEventParse` and `EtwPropConvert` traits

## Step-by-Step Generation Process

### 1. Extract Provider Metadata

From the XML manifest, extract:

- **Provider name**: `<provider name="...">` attribute
- **Provider GUID**: `<provider guid="...">` attribute (strip braces, format as `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`)

### 2. Extract Event Definitions

From `<events>` section:

- **Event value**: `<event value="...">` attribute → becomes `id` in `#[etw_event(id = ...)]`
- **Event version**: `<event version="...">` attribute → becomes `version` in `#[etw_event(version = ...)]`
- **Template reference**: `<event template="...">` attribute → used to look up field definitions

### 3. Extract Template Fields

From `<templates>` section, for each template referenced by events:

- **Field name**: `<data name="...">` attribute → becomes `#[etw_prop(name = "...")]`
- **Input type**: `<data inType="...">` attribute → determines Rust type and parse strategy

### 4. Map ETW inType to Rust Types

| ETW inType | ferrisetw parser type | Rust field type | Notes |
|---|---|---|---|
| `win:Pointer` | `ferrisetw::parser::Pointer` | `usize` | Requires `parse_as = ferrisetw::parser::Pointer` with `EtwPropConvert` |
| `win:UInt8` | `u8` | `u8` | Direct parse |
| `win:Int8` | `i8` | `i8` | Direct parse |
| `win:UInt16` | `u16` | `u16` | Direct parse |
| `win:Int16` | `i16` | `i16` | Direct parse |
| `win:UInt32` | `u32` | `u32` | Direct parse |
| `win:Int32` | `i32` | `i32` | Direct parse |
| `win:UInt64` | `u64` | `u64` | Direct parse |
| `win:Int64` | `i64` | `i64` | Direct parse |
| `win:Float` | `f32` | `f32` | Direct parse |
| `win:Double` | `f64` | `f64` | Direct parse |
| `win:Boolean` | `bool` | `bool` | Direct parse |
| `win:UnicodeString` | `String` | `String` | Direct parse |
| `win:AnsiString` | `String` | `String` | Direct parse |
| `win:GUID` | `windows::core::GUID` | `windows::core::GUID` | Direct parse (ferrisetw handles `win:GUID` natively) |
| `win:FILETIME` | `ferrisetw::native::time::FileTime` | `time::OffsetDateTime` | Requires `parse_as = ferrisetw::native::time::FileTime` with `EtwPropConvert` impl |
| `win:SYSTEMTIME` | `ferrisetw::native::time::SystemTime` | `time::OffsetDateTime` | Requires `parse_as = ferrisetw::native::time::SystemTime` with `EtwPropConvert` impl |
| `win:SID` | `String` | `String` | Direct parse (ferrisetw converts SID to SDDL string natively) |
| `win:Binary` | `Vec<u8>` | `Vec<u8>` | Direct parse |
| `win:HexInt32` | `u32` | `u32` | Direct parse (hex representation) |
| `win:HexInt64` | `u64` | `u64` | Direct parse (hex representation) |

### 5. Field Conversion Rules

#### Direct Parse (no `parse_as` needed)
For types that ferrisetw can parse directly into the desired Rust type:
```rust
#[etw_prop(name = "FieldName")]
pub field_name: u32,
```

#### Pointer Parse (requires `parse_as`)
For `win:Pointer` inType:
```rust
#[etw_prop(name = "FieldName", parse_as = ferrisetw::parser::Pointer)]
pub field_name: usize,
```
This works because `EtwPropConvert<ferrisetw::parser::Pointer> for usize` is implemented in `etw.rs`.

#### FILETIME/SYSTEMTIME Parse (requires `parse_as` + `EtwPropConvert`)
For `win:FILETIME` or `win:SYSTEMTIME` inType, first add the conversion impl to `etw.rs`:
```rust
impl EtwPropConvert<ferrisetw::native::time::FileTime> for time::OffsetDateTime {
    fn convert(value: ferrisetw::native::time::FileTime) -> Self {
        value.as_date_time()
    }
}

impl EtwPropConvert<ferrisetw::native::time::SystemTime> for time::OffsetDateTime {
    fn convert(value: ferrisetw::native::time::SystemTime) -> Self {
        value.as_date_time()
    }
}
```

Then use:
```rust
#[etw_prop(name = "FieldName", parse_as = ferrisetw::native::time::FileTime)]
pub field_name: time::OffsetDateTime,
```

#### Custom Conversion (requires `parse_as` + `convert_with`)
For custom conversion logic:
```rust
#[etw_prop(name = "FieldName", parse_as = u64, convert_with = my_converter)]
pub field_name: MyType,
```

### 6. Struct and Enum Generation

#### Provider Enum
```rust
etw_provider! {
    #[etw_provider(name = "ProviderName", guid = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX")]
    pub enum ProviderEvent {
        #[etw_event(id = EVENT_ID, version = VERSION, mask = KEYWORD_MASK)]
        pub struct EventNameV0 {
            #[etw_prop(name = "FieldName")]
            pub field_name: FieldType,
            // ... more fields
        }
    }
}
```

#### Field Ordering
Fields must be in the same order as defined in the template XML.

#### Event Versioning
- Events with multiple versions get separate structs (e.g., `EventNameV0`, `EventNameV1`)
- Version is specified in `#[etw_event(version = N)]`
- Events without version attribute use wildcard matching

### 7. Keyword Masks

Extract keyword masks from `<keywords>` section and combine with OR for `mask`:
```xml
<keyword name="KEYWORD_A" mask="0x10" />
<keyword name="KEYWORD_B" mask="0x20" />
```

For events using multiple keywords:
```rust
#[etw_event(id = 1, version = 0, mask = 0x30)] // 0x10 | 0x20
```

If all events use keywords, compute combined mask for `build_provider`:
```rust
pub fn build_provider<F>(callback: F) -> ferrisetw::provider::Provider {
    // ...
    .any(COMBINED_MASK)
    // ...
}
```

### 8. Module Registration

After creating the provider file (e.g., `kernel_process.rs`), register it in `providers/mod.rs`:
```rust
pub mod kernel_file;
pub mod kernel_process;
```

## Example: Complete Event Generation

Given XML:
```xml
<event value="1" symbol="ProcessStart" version="0" task="ProcessStart" 
       opcode="win:Start" level="win:Informational" 
       keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStartArgs" />

<template tid="ProcessStartArgs">
    <data name="ProcessID" inType="win:UInt32" />
    <data name="CreateTime" inType="win:FILETIME" />
    <data name="ParentProcessID" inType="win:UInt32" />
    <data name="SessionID" inType="win:UInt32" />
    <data name="ImageName" inType="win:UnicodeString" />
</template>
```

Generated Rust:
```rust
#[etw_event(id = 1, version = 0, mask = 0x10)]
pub struct ProcessStartV0 {
    #[etw_prop(name = "ProcessID")]
    pub process_id: u32,
    #[etw_prop(name = "CreateTime", parse_as = ferrisetw::native::time::FileTime)]
    pub create_time: time::OffsetDateTime,
    #[etw_prop(name = "ParentProcessID")]
    pub parent_process_id: u32,
    #[etw_prop(name = "SessionID")]
    pub session_id: u32,
    #[etw_prop(name = "ImageName")]
    pub image_name: String,
}
```

## Notes

- The `mask` field in `#[etw_event]` is optional. If omitted, no keyword filtering is applied.
- Events with `opcode="win:Start"` or `opcode="win:Stop"` typically have no version suffix in the struct name, but follow the same pattern.
- The `build_provider` function is automatically generated when `guid` is provided.
- The `any()` filter is only added when ALL events have keyword masks defined.
