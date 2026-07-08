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

## 4. Options for Adding Rundown (Ranked)

### Option A — Fork `ferrisetw` and add `request_rundown()` to `UserTrace` (Recommended)

Add a public method on `UserTrace` that delegates to `EnableTraceEx2` with `EVENT_CONTROL_CODE_CAPTURE_STATE`:

```rust
// In src/trace.rs, impl UserTrace block
pub fn request_rundown(&self, provider_guid: &GUID) -> Result<(), TraceError> {
    unsafe {
        Etw::EnableTraceEx2(
            self.control_handle,
            provider_guid as *const GUID,
            EVENT_CONTROL_CODE_CAPTURE_STATE.0,
            0,  // level
            0,  // match_any_keyword
            0,  // match_all_keyword
            0,  // timeout
            None,  // enable_parameters (can include filters if needed)
        )
    }
    .ok()
    .map_err(|e| TraceError::EtwNativeError(
        crate::native::EvntraceNativeError::IoError(
            std::io::Error::from_raw_os_error(e.code().0)
        )
    ))?;
    Ok(())
}
```

**Changes required in `src/native/evntrace.rs`:**
- Add import: `use windows::Win32::System::Diagnostics::Etw::{..., EVENT_CONTROL_CODE_CAPTURE_STATE};` (the constant exists in the `windows` crate but is not imported)

**Also consider the krabsetw pattern:** For both Option A and B, add `rundown_enabled` field to `Provider`/`ProviderBuilder` and call `EVENT_CONTROL_CODE_CAPTURE_STATE` automatically after `enable_provider` in the `start()` flow (like `ut::enable_rundown()` does in krabsetw). This would match the C++ API pattern.

**Use from your code:**
```rust
// After starting the trace
trace.request_rundown(&"3f68e5c2-3f68-4e5c-a3f6-8e5ca368e5c2".into());
```

---

### Option B — Fork `ferrisetw` to expose `control_handle` only (Minimal change)

Add a single public accessor to `UserTrace`:

```rust
// In src/trace.rs, impl UserTrace block
pub fn control_handle(&self) -> ControlHandle {
    self.control_handle
}
```

Then in your code, call `EnableTraceEx2` directly via the `windows` crate:

```rust
use windows::Win32::System::Diagnostics::Etw::{
    EnableTraceEx2, EVENT_CONTROL_CODE_CAPTURE_STATE, CONTROLTRACE_HANDLE,
};

let handle = trace.control_handle();
unsafe {
    EnableTraceEx2(
        handle,
        &guid as *const _,
        EVENT_CONTROL_CODE_CAPTURE_STATE.0,
        0, 0, 0, 0, None,
    );
}
```

**Downside:** Exposes a low-level handle that could be misused (stopping the trace behind the library's back, dangling handles, etc.).

---

### Option C — No fork: use `trace_handle` + `ControlTraceW` by session name (Will NOT work)

`ControlTraceW` accepts a session name (by passing a zero handle), but `ControlTraceW` only supports `EVENT_TRACE_CONTROL_QUERY`, `EVENT_TRACE_CONTROL_STOP`, and `EVENT_TRACE_CONTROL_FLUSH` — **not** `EVENT_CONTROL_CODE_CAPTURE_STATE`. You must use `EnableTraceEx2`, which requires the `CONTROLTRACE_HANDLE`.

The `freebasic` example you found uses a manual `hTrace` (`TRACEHANDLE`) from `StartTraceW`, which is the same as `control_handle` in ferrisetw — confirming you need this specific handle.

---

### Option D — No fork: add rundown to `TraceBuilder::start()` before returning (Moderate change, requires rebuilding ferrisetw)

Modify `TraceBuilder::start()` to call rundown on providers that opt in, right after the `enable_provider` loop (before `open_trace`). This is the approach krabsetw uses in `ut::enable_rundown()` — called between `enable_providers()` and `ProcessTrace()`.

This still requires modifying `ferrisetw` source (you'd rebuild it as a local path dep).

---

## 5. Implementation Plan (Recommended: Option A)

### 5.1 Fork or patch `ferrisetw` locally

In your `Cargo.toml`, switch to a local path:

```toml
[dependencies]
ferrisetw = { path = "etw_learning_resources/ferrisetw" }
```

### 5.2 Add `EVENT_CONTROL_CODE_CAPTURE_STATE` import

**File:** `src/native/evntrace.rs`
- Change line 22 from:
  ```rust
  EVENT_CONTROL_CODE_ENABLE_PROVIDER, EVENT_TRACE_CONTROL_QUERY,
  ```
  To:
  ```rust
  EVENT_CONTROL_CODE_ENABLE_PROVIDER, EVENT_CONTROL_CODE_CAPTURE_STATE, EVENT_TRACE_CONTROL_QUERY,
  ```

### 5.3 Add `request_rundown()` to `UserTrace`

**File:** `src/trace.rs` — add to the `impl UserTrace` block (after line 293):

```rust
/// Request rundown events from a specific provider.
///
/// This sends `EVENT_CONTROL_CODE_CAPTURE_STATE` to the provider,
/// causing it to emit state-capture (rundown) events.
/// Should be called after the trace is started and processing events.
pub fn request_rundown(&self, provider_guid: &GUID) -> TraceResult<()> {
    use crate::native::evntrace::filter_invalid_control_handle;
    use windows::Win32::System::Diagnostics::Etw::EVENT_CONTROL_CODE_CAPTURE_STATE;

    let handle = match filter_invalid_control_handle(self.control_handle) {
        Some(h) => h,
        None => return Err(TraceError::EtwNativeError(
            crate::native::EvntraceNativeError::InvalidHandle,
        )),
    };

    let res = unsafe {
        Etw::EnableTraceEx2(
            handle,
            provider_guid as *const GUID,
            EVENT_CONTROL_CODE_CAPTURE_STATE.0,
            0, // level - using 0 for rundown (matches krabsetw)
            0, // match_any_keyword
            0, // match_all_keyword
            0, // timeout
            None,
        )
    }
    .ok();

    res.map_err(|err| {
        TraceError::EtwNativeError(crate::native::EvntraceNativeError::IoError(
            std::io::Error::from_raw_os_error(err.code().0),
        ))
    })
}
```

**Note:** `filter_invalid_control_handle` is `pub(crate)` in `native/evntrace.rs`, so it's accessible within the crate. If you prefer to inline the check, just validate `control_handle.Value != 0`.

### 5.4 Call it from `manager.rs`

After `start_and_process()` returns a live trace:

```rust
pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
where
    F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
{
    log::info!("Creating new ETW session: '{}'...", self.session_name);

    let trace = self
        .register_providers(shared_callback)
        .named(self.session_name.clone())
        .start_and_process()?;

    // Request rundown for providers that need it
    // (e.g., Microsoft-Windows-Kernel-Process for ProcessRundown events)
    trace.request_rundown(&GUID::from("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"))?;

    log::info!("ETW Trace session '{}' is now active.", self.session_name);
    Ok(EtwTraceSession {
        session_name: self.session_name,
        trace: Some(trace),
    })
}
```

### 5.5 (Optional) Add `rundown_enabled` to `ProviderBuilder`

For a cleaner API matching krabsetw, add a `rundown_enabled(bool)` method to `ProviderBuilder`, store it in `Provider`, then call `request_rundown` automatically for all providers with that flag set. This is the `ut::enable_rundown()` pattern from krabsetw.

---

## 6. Krabsetw Reference (from `ut.hpp`)

### `filter_settings` struct (line 37–47)
```cpp
struct filter_settings {
    std::set<unsigned short> provider_filter_event_ids_;
    filter_flags filter_flags_{};
    bool rundown_enabled_ = false;    // <-- per-provider rundown flag
};
```

### `enable_rundown()` (line 203–220)
```cpp
static void enable_rundown(const krabs::trace<krabs::details::ut>& trace) {
    if (trace.registrationHandle_ == INVALID_PROCESSTRACE_HANDLE)
        return;

    for (auto& provider : trace.providers_) {
        if (!provider.get().rundown_enabled_)
            continue;

        ULONG status = EnableTraceEx2(trace.registrationHandle_,
            &provider.get().guid_,
            EVENT_CONTROL_CODE_CAPTURE_STATE,
            0,  // level
            0,  // match_any_keyword
            0,  // match_all_keyword
            0,  // timeout
            NULL);
        error_check_common_conditions(status);
    }
}
```

Called from `trace::start()` right before `ProcessTrace`.

---

## 7. Verification

After adding the rundown call:
- For `Microsoft-Windows-Kernel-Process` (GUID `22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716`), you should receive events like `ProcessRundown` (event ID 15) in addition to `ProcessStart` (event ID 1).
- For `.NET` runtime, you'd enable `Microsoft-Windows-DotNETRuntimeRundown` as a *separate provider* (this doesn't need `EVENT_CONTROL_CODE_CAPTURE_STATE`; it's a standalone provider that emits rundown events when enabled with the `StartRundownKeyword`).

---

## 8. Window of Opportunity for Rundown

According to krabsetw comments in `etw.hpp` line 377:

> `EnableTraceEx2(EVENT_CONTROL_CODE_CAPTURE_STATE)` must be called very shortly after the trace starts processing with `ProcessTrace`, or the rundown events might get lost.

In practice, calling it after `start_and_process()` (which spawns the `ProcessTrace` thread) is usually fine, but for maximum reliability you should call it before `ProcessTrace` begins. This means you'd need to use `start()` instead of `start_and_process()`, capture both the `UserTrace` and `TraceHandle`, spawn the processing thread **after** the rundown call:

```rust
let (mut trace, trace_handle) = builder.start()?;
trace.request_rundown(&guid)?;
std::thread::spawn(move || UserTrace::process_from_handle(trace_handle));
```

If you go with Option A (fork with `request_rundown`), you can also restructure the builder to accept `rundown` providers and call `enable_rundown()` between `enable_providers()` and `open_trace()` — matching the krabsetw timing exactly.
