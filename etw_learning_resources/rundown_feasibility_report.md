# Adding Rundown Support to `fileiolog` — Feasibility Report

## 1. Problem Summary

You need to call `EnableTraceEx2` with `EVENT_CONTROL_CODE_CAPTURE_STATE` to request rundown events from ETW providers. The `ferrisetw` crate has no method to do this today, and the required handle is private.

---

## 2. Handle Types in `ferrisetw`

| Handle | Type | Source | Required For | Public? |
|---|---|---|---|---|
| `control_handle` | `CONTROLTRACE_HANDLE` | Returned by `StartTraceW` | `EnableTraceEx2`, `ControlTraceW` | **No** — private field of `UserTrace` / `KernelTrace` |
| `trace_handle` | `PROCESSTRACE_HANDLE` | Returned by `OpenTraceW` | `ProcessTrace`, `CloseTrace` | **Yes** — via `TraceTrait::trace_handle()` |

Both handles are created during `TraceBuilder::start()` (line 537–562 in `src/trace.rs`). The `enable_provider()` function in `src/native/evntrace.rs` (line 234–272) uses `control_handle` internally, but it hardcodes `EVENT_CONTROL_CODE_ENABLE_PROVIDER` — there is no `EVENT_CONTROL_CODE_CAPTURE_STATE` path.

---

## 3. What `ferrisetw` Is Missing

1. **No `control_handle()` public accessor** on `UserTrace` / `KernelTrace`.
2. **No `EVENT_CONTROL_CODE_CAPTURE_STATE` reference** anywhere in the codebase (imported or used).
3. **No `rundown_enabled` field** on the `Provider` struct — krabsetw has this (see `ut.hpp` line 46: `bool rundown_enabled_ = false`).
4. **`enable_provider()` is `pub(crate)`** — not callable from consumer code.
5. **`enable_rundown()` free function** missing entirely — krabsetw has `ut::enable_rundown()` (ut.hpp line 204) that iterates providers with `rundown_enabled_` and calls `EnableTraceEx2` with `EVENT_CONTROL_CODE_CAPTURE_STATE`.

---

## 4. New Finding: Recover Session Handle via `ControlTraceW(QUERY)` (No Fork Needed)

You can use `ControlTraceW` with `EVENT_TRACE_CONTROL_QUERY` and the session **name** to retrieve the session handle, then use it with `EnableTraceEx2`.

### How It Works

`ControlTraceW` with `TraceHandle = 0` + `InstanceName = session_name` + `ControlCode = EVENT_TRACE_CONTROL_QUERY` fills in an `EVENT_TRACE_PROPERTIES` buffer. The `WNODE_HEADER.HistoricalContext` field contains the session handle on output.

From MSDN:
> **WNODE_HEADER.HistoricalContext** — On output, the handle to the event tracing session. You can use this handle with the `EnableTraceEx2` function.

### Type Layout (confirmed in `windows` crate 0.57)

```rust
pub union WNODE_HEADER_0 {                      // #[repr(C)] union
    pub HistoricalContext: u64,
    pub Anonymous: WNODE_HEADER_0_0,             // { Version: u32, Linkage: u32 }
}

pub struct EVENT_TRACE_PROPERTIES {
    pub Wnode: WNODE_HEADER,                     // Wnode.Anonymous1 is WNODE_HEADER_0
    pub BufferSize: u32,
    // ... other fields ...
    pub LoggerNameOffset: u32,
    // ...
}

pub struct CONTROLTRACE_HANDLE { pub Value: u64; }
```

### Key Constants (all available in the `windows` crate)

- `EVENT_TRACE_CONTROL_QUERY` — `EVENT_TRACE_CONTROL` newtype, usable directly
- `EVENT_CONTROL_CODE_CAPTURE_STATE` — `ENABLECALLBACK_ENABLED_STATE` newtype, use `.0` for raw `u32`
- `WNODE_FLAG_TRACED_GUID` — `u32`

### Verified: the flow is doable end-to-end

1. Allocate a `vec![0u8; N]` big enough for `EVENT_TRACE_PROPERTIES + session_name + log_file_name`
2. Cast the buffer to `*mut EVENT_TRACE_PROPERTIES`
3. Set `Wnode.BufferSize = total_len`, `Wnode.Flags = WNODE_FLAG_TRACED_GUID`, `LoggerNameOffset = offsetof(EVENT_TRACE_PROPERTIES, after struct)`
4. Copy the wide session name into the buffer at `LoggerNameOffset`
5. Call `ControlTraceW(CONTROLTRACE_HANDLE { Value: 0 }, session_name_ptr, properties_ptr, EVENT_TRACE_CONTROL_QUERY)`
6. On success: `let handle = unsafe { (*properties).Wnode.Anonymous1.HistoricalContext }`
7. Construct `CONTROLTRACE_HANDLE { Value: handle }`
8. Call `EnableTraceEx2(handle, &provider_guid, EVENT_CONTROL_CODE_CAPTURE_STATE.0, 0, 0, 0, 0, None)`

The `widestring` crate is already a dependency of `ferrisetw`, and the `windows` crate is already linked, so no new dependencies.

---

## 5. Options for Adding Rundown (Ranked)

### Option E (NEW, BEST) — No fork. Call `ControlTraceW(QUERY)` + `EnableTraceEx2` directly

This requires **zero changes to ferrisetw**. You add a helper function to your own `manager.rs` that:

1. Calls `ControlTraceW` by session name with `EVENT_TRACE_CONTROL_QUERY`
2. Extracts `HistoricalContext` from the output `WNODE_HEADER`
3. Calls `EnableTraceEx2` with the recovered handle

**You already have the session name** (`EtwTraceManager.session_name`), and the `windows` crate is already linked transitively through `ferrisetw`. You just need to add `Win32_System_Diagnostics_Etw` to your own `Cargo.toml`'s `windows` dependency, or use `unsafe` FFI directly.

#### Full implementation code for `manager.rs`

New helper function:

```rust
use windows::Win32::System::Diagnostics::Etw::{
    self as Etw, ControlTraceW, EnableTraceEx2, EVENT_TRACE_CONTROL_QUERY,
    EVENT_CONTROL_CODE_CAPTURE_STATE, WNODE_FLAG_TRACED_GUID,
    EVENT_TRACE_PROPERTIES, CONTROLTRACE_HANDLE,
};
use windows::core::GUID;
use std::mem;

/// Retrieve the session handle by querying the trace by name,
/// then send EVENT_CONTROL_CODE_CAPTURE_STATE to request rundown events.
pub fn request_rundown(session_name: &str, provider_guid: &GUID) -> Result<(), Box<dyn std::error::Error>> {
    // Buffer: EVENT_TRACE_PROPERTIES + session name (wide) + log file name (wide)
    let buf_size = mem::size_of::<EVENT_TRACE_PROPERTIES>() + 512 * mem::size_of::<u16>();
    let mut buf = vec![0u8; buf_size];

    let properties = buf.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES;

    // Convert session name to UTF-16
    let wide_name: Vec<u16> = session_name.encode_utf16().chain(std::iter::once(0)).collect();
    let logger_name_offset = mem::size_of::<EVENT_TRACE_PROPERTIES>() as u32;

    unsafe {
        // Initialize the EVENT_TRACE_PROPERTIES header
        (*properties).Wnode.BufferSize = buf_size as u32;
        (*properties).Wnode.Flags = WNODE_FLAG_TRACED_GUID;
        (*properties).LoggerNameOffset = logger_name_offset;

        // Copy the session name after the struct
        let name_dst = buf.as_mut_ptr()
            .add(logger_name_offset as usize)
            as *mut u16;
        std::ptr::copy_nonoverlapping(wide_name.as_ptr(), name_dst, wide_name.len());

        // Query the session — this populates HistoricalContext with the session handle
        let result = ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            windows::core::PCWSTR::from_raw(name_dst),
            properties,
            EVENT_TRACE_CONTROL_QUERY,
        );

        if let Err(e) = result {
            return Err(format!("ControlTraceW(QUERY) failed: {}", e).into());
        }

        // Read the session handle from HistoricalContext
        let handle_value = (*properties).Wnode.Anonymous1.HistoricalContext;
        let session_handle = CONTROLTRACE_HANDLE { Value: handle_value };

        // Request rundown events
        let result = EnableTraceEx2(
            session_handle,
            provider_guid as *const GUID,
            EVENT_CONTROL_CODE_CAPTURE_STATE.0,
            0,  // level (0 = current, matches krabsetw)
            0,  // match_any_keyword
            0,  // match_all_keyword
            0,  // timeout
            None,
        );

        if let Err(e) = result {
            return Err(format!("EnableTraceEx2(CAPTURE_STATE) failed: {}", e).into());
        }
    }

    Ok(())
}
```

**To add the `windows` crate dependency**, add to your `Cargo.toml`:

```toml
[dependencies]
windows = { version = "0.57", features = ["Win32_System_Diagnostics_Etw", "Win32_Foundation"] }
```

(Use the same version that ferrisetw pulls in, which is 0.57.)

Then call it from `EtwTraceManager::start()`:

```rust
pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
where
    F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
{
    let trace = self
        .register_providers(shared_callback)
        .named(self.session_name.clone())
        .start_and_process()?;

    // Request rundown after trace is processing
    if let Err(e) = request_rundown(
        &self.session_name,
        &"22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716".into(),  // Microsoft-Windows-Kernel-Process
    ) {
        log::warn!("Rundown request failed (may still work): {:?}", e);
    }

    Ok(EtwTraceSession { session_name: self.session_name, trace: Some(trace) })
}
```

---

### Option A — Fork `ferrisetw` and add `request_rundown()` to `UserTrace`

(Still valid but no longer necessary if Option E works.)

---

### Option C — Call `EnableTraceEx2` with zero handle

Will NOT work. `EnableTraceEx2` requires a valid `CONTROLTRACE_HANDLE` — there is no `EnableTraceEx2ByName` API.

---

## 6. Implementation Plan (Recommended: Option E)

### Step 1: Add `windows` crate to `Cargo.toml`

```toml
windows = { version = "0.57", features = ["Win32_System_Diagnostics_Etw", "Win32_Foundation"] }
```

### Step 2: Add the `request_rundown` helper to `manager.rs`

Use the code from Option E above.

### Step 3: Test with `Microsoft-Windows-Kernel-Process`

Call it after `start_and_process()`. You should see `ProcessRundown` (event ID 15) events in your callback.

### Step 4: Optional — Refine timing

For maximum reliability, use `start()` + `process_from_handle` instead of `start_and_process()` to call `request_rundown()` before `ProcessTrace` begins:

```rust
let (mut trace, trace_handle) = builder.start()?;
request_rundown(&session_name, &guid)?;
std::thread::spawn(move || UserTrace::process_from_handle(trace_handle));
```

---

## 7. Verification

- For `Microsoft-Windows-Kernel-Process` (GUID `22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716`), adding `any(0x10)` (WINEVENT_KEYWORD_PROCESS) and requesting rundown should produce events like:
  - Event ID 1: `ProcessStart`
  - Event ID 2: `ProcessStop`
  - Event ID 15: `ProcessRundown`
- For `.NET` runtime, enable the *separate* `Microsoft-Windows-DotNETRuntimeRundown` provider with `StartRundownKeyword` (0x40) — this doesn't need `EVENT_CONTROL_CODE_CAPTURE_STATE`.

---

## 8. Krabsetw Reference

In krabsetw, this is done via `ut::enable_rundown()` (ut.hpp:204) which iterates providers with `rundown_enabled_ == true` and calls `EnableTraceEx2(registrationHandle_, &guid, EVENT_CONTROL_CODE_CAPTURE_STATE, ...)`. The `registrationHandle_` is the same as ferrisetw's `control_handle` and the same value returned by `StartTraceW` and stored in `HistoricalContext`.

---

## 9. Theoretical Unsafe Memory Read (Not Recommended)

Even though `UserTrace` lacks `#[repr(C)]`, you could attempt pointer arithmetic to read the private `control_handle` field. This is **undefined behavior** — Rust's default layout allows field reordering, and both `control_handle` and `trace_handle` are identical `{ Value: u64 }` wrappers, making them indistinguishable by value. Do not rely on this.
