# ETW Rundown Support for fileiolog

## 1. What Is ETW Rundown?

When a trace session starts, some providers emit **DCStart/DCEnd** ("Dara Collect" start/end) events that describe the *current state* of the system — e.g. existing file handles, active processes, loaded modules, etc. These are known as **rundown events**.

The mechanism to trigger them is calling `EnableTraceEx2` with `EVENT_CONTROL_CODE_CAPTURE_STATE` (value `2`) for each provider you want rundown from. The provider then emits its current state as a burst of events, followed by the normal event stream.

**Critical timing requirement** (from krabsetw `etw.hpp:375-378`):
> "During the testing of the (slower) C++/CLI implementation it became evident that EnableTraceEx2(EVENT_CONTROL_CODE_CAPTURE_STATE) must be called very shortly before ProcessTrace() in order for the rundown events to be generated."

The call must happen **after** `StartTraceW` / `OpenTraceW` but **immediately before** `ProcessTrace`.

---

## 2. How krabsetw Does It

**File:** `krabs/krabs/ut.hpp:204-224`

```cpp
inline void ut::enable_rundown(
    const krabs::trace<krabs::details::ut>& trace)
{
    if (trace.registrationHandle_ == INVALID_PROCESSTRACE_HANDLE)
        return;

    for (auto& provider : trace.providers_) {
        if (!provider.get().rundown_enabled_)
            continue;

        ULONG status = EnableTraceEx2(trace.registrationHandle_,
            &provider.get().guid_,
            EVENT_CONTROL_CODE_CAPTURE_STATE,
            0,        // Level
            0,        // AnyKeyword
            0,        // AllKeyword
            0,        // Timeout (0 = infinite)
            NULL);    // Parameters
        error_check_common_conditions(status);
    }
}
```

Called from `process_trace()` in `krabs/krabs/etw.hpp:379`:
```cpp
template <typename T>
void trace_manager<T>::process_trace()
{
    // ...
    T::trace_type::enable_rundown(trace_);   // <-- HERE
    ULONG status = ProcessTrace(&trace_.sessionHandle_, 1, NULL, NULL);
    // ...
}
```

The lifecycle is:
1. `StartTrace` → gets `registrationHandle_` (a `TRACEHANDLE`, same as ferrisetw's `ControlHandle`)
2. `EnableTraceEx2(EVENT_CONTROL_CODE_ENABLE_PROVIDER)` for each provider
3. `OpenTrace` → gets `sessionHandle_`
4. **`EnableTraceEx2(EVENT_CONTROL_CODE_CAPTURE_STATE)` for each rundown-enabled provider**
5. `ProcessTrace` (blocking)

Steps 4+5 happen atomically — no other work or delay between them.

---

## 3. ferrisetw's Limitations

The bloated ferrisetw (cloned at `etw_learning_resources/ferrisetw/`) has **no rundown support at all**. Grepping for `rundown|Rundown|RUNDOWN|CAPTURE_STATE` returns zero results.

**What's private vs public:**

| Item | Visibility | Needed For |
|------|-----------|------------|
| `UserTrace.control_handle` (`ControlHandle`) | private field | `EnableTraceEx2` first argument |
| `enable_provider()` in `native::evntrace` | `pub(crate)` | pattern to call `EnableTraceEx2` |
| `EventTraceProperties` in `native::etw_types` | `pub(crate)` (struct is `pub` but module is `pub(crate)`) | constructing QUERY buffer |
| `Provider.guid()` | **public** | iterating providers for rundown |
| `UserTrace::trace_name()` (from `RealTimeTraceTrait`) | **public** | querying session by name |
| `stop_trace_by_name()` | **public** | demonstrates `ControlTraceW` by-name pattern |
| `control_trace_by_name()` | `pub(crate)` | cannot use directly |

The `control_handle` is the returned handle from `StartTraceW` stored in `UserTrace` at `src/trace.rs:210`. It's used for `ControlTraceW(STOP)` and would be needed for `EnableTraceEx2`.

---

## 4. Implementation Approaches

There are two viable approaches. We recommend **Approach B**.

### Approach A: Fork/Extend ferrisetw

**Steps:**
- Fork ferrisetw, add a `capture_state()` or `enable_rundown()` method on `UserTrace`
- Make `control_handle` accessible or add a method that wraps the `EnableTraceEx2(CAPTURE_STATE)` call
- Use a custom fork in `Cargo.toml`

**Downsides:**
- Maintenance burden — must track upstream changes
- The `TraceBuilder::start_and_process()` would still need modification to inject rundown before `ProcessTrace`

---

### Approach B: Call `EnableTraceEx2` Directly via `windows` Crate (Recommended)

We already depend on ferrisetw, which depends on `windows = "0.57.0"`. We add the same `windows` crate to our `Cargo.toml` and call `EnableTraceEx2` and `ControlTraceW` ourselves.

#### How to Get the Control Handle

The `control_handle` is private inside `UserTrace`. But we can retrieve it **from the session name** by calling `ControlTraceW` with `EVENT_TRACE_CONTROL_QUERY`.

When `ControlTraceW` returns successfully, the `Wnode.HistoricalContext` field of the `EVENT_TRACE_PROPERTIES` structure contains the session handle (the same as `control_handle` from `StartTraceW`).

**This is documented at:**
- `wnode-header.md`: *"HistoricalContext — On output, the handle to the event tracing session."*
- `ns-evntrace-event_trace_properties.md`: *"If ControlCode specifies EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_CONTROL_QUERY or EVENT_TRACE_CONTROL_FLUSH, you only need to set the Wnode.BufferSize, Wnode.Guid, LoggerNameOffset, and LogFileNameOffset members"*

#### Step-by-Step Plan

1. **Add `windows` dependency to `Cargo.toml`:**
   ```toml
   windows = { version = "0.57", features = [
       "Win32_Foundation",
       "Win32_System_Diagnostics_Etw",
   ] }
   ```
   (Must match ferrisetw's windows version exactly to avoid duplicate types)

2. **Create a helper module** (e.g. `src/rundown.rs`) with a function:
   ```rust
   pub fn request_rundown(
       session_name: &str,
       provider_guids: &[GUID],
   ) -> Result<(), std::io::Error>
   ```

   This function will:
   - Allocate a buffer large enough for `EVENT_TRACE_PROPERTIES` + wide session name (1024 chars max, or 200 to match ferrisetw)
   - Zero-initialize and populate:
     - `Wnode.BufferSize = size_of::<EVENT_TRACE_PROPERTIES>() + (max_name_chars + 1) * 2`
     - `Wnode.Flags = WNODE_FLAG_TRACED_GUID` (0x00020000)
     - `Wnode.Guid = GUID::zeroed()` (or any GUID; for named query it doesn't need to match)
     - `LoggerNameOffset = size_of::<EVENT_TRACE_PROPERTIES>() as u32`
     - `LogFileNameOffset = 0`
     - Copy session name into the buffer at `LoggerNameOffset`
   - Call `ControlTraceW(name_ptr, &mut props, CONTROL_QUERY)`
   - Extract `control_handle` from `props.Wnode.HistoricalContext`
   - For each GUID in `provider_guids`, call:
     ```
     EnableTraceEx2(control_handle, &guid, EVENT_CONTROL_CODE_CAPTURE_STATE,
                    0, 0, 0, 0, None)
     ```

3. **Modify `manager.rs`** to use the **two-step start** (`start()` then manual `process()`) instead of `start_and_process()`:

   ```rust
   pub fn start<F>(self, shared_callback: F) -> Result<EtwTraceSession, TraceError>
   where
       F: Fn(ProviderEvent) + Send + Sync + Clone + 'static,
   {
       // Build providers and collect GUIDs
       let (providers, provider_guids) = self.build_providers(shared_callback);
       
       let builder = UserTrace::new();
       let builder = providers.into_iter().fold(builder, |b, p| b.enable(p));
       let builder = builder.named(self.session_name.clone());
       
       // Step 1: start the trace (StartTraceW + EnableTraceEx2 + OpenTraceW)
       let (trace, trace_handle) = builder.start()?;
       
       // Step 2: request rundown (MUST be before ProcessTrace)
       request_rundown(&self.session_name, &provider_guids)?;
       
       // Step 3: spawn ProcessTrace thread
       std::thread::spawn(move || UserTrace::process_from_handle(trace_handle));
       
       Ok(EtwTraceSession {
           session_name: self.session_name,
           trace: Some(trace),
       })
   }
   ```

4. **Modify `register_providers`** to return both the providers and their GUIDs.

5. **(Optional) Add event IDs for rundown events** to your provider enum definitions (e.g. `kernel_file.rs`). For `Microsoft-Windows-Kernel-File`, common rundown events include DCStart/DCStart2 for file create rundown, etc. You'll need to discover these via trace capture or Microsoft documentation.

---

## 5. Open Questions / Issues

| Question | Status |
|----------|--------|
| Does `ControlTraceW(QUERY)` work with the session name right after `StartTraceW`? | ✅ Yes, same as `stop_trace_by_name` uses it |
| Is `Wnode.HistoricalContext` really the same as `ControlHandle`? | ✅ Yes — both are `ULONG64` / `u64`; MSDN confirms |
| Does `EnableTraceEx2(CAPTURE_STATE)` require admin? | Same as `ENABLE_PROVIDER` — yes, both require admin |
| Will all providers respond to `CAPTURE_STATE`? | ❌ **No.** A provider must explicitly implement rundown logic (DCStart/DCEnd events). `Microsoft-Windows-Kernel-File` *does* have rundown events (IDs 36-39 for DCStart, etc.), but many providers don't. The call is harmless for non-responding providers (returns success but produces no events). |
| What about kernel trace (NT Kernel Logger) rundown? | krabsetw uses a **different** mechanism: special GUID `{3b9c9951-3480-4220-9377-9c8e5184f5cd}` with `EVENT_CONTROL_CODE_ENABLE_PROVIDER` and rundown flags. Not applicable to user traces (`UserTrace`). |
| Does `EnableTraceEx2(CAPTURE_STATE)` need the same level/keywords as enable? | ❌ No. The krabsetw code passes `0` for all (level=TRACE_LEVEL_NONE, any=0, all=0), relying on the already-enabled provider to send its current state. |
| Timing: can we call between `start()` returning and `std::thread::spawn`? | ✅ Yes. `start()` does `StartTraceW → EnableTraceEx2(ENABLE) → OpenTraceW`. None of those block. We then call `CAPTURE_STATE` on the same thread, then spawn `ProcessTrace`. The critical constraint is that `CAPTURE_STATE` must be *shortly before* `ProcessTrace`, which is satisfied. |

---

## 6. Key Files Referenced

| File | Content |
|------|---------|
| `src/manager.rs` | The file to modify — our trace session manager |
| `etw_learning_resources/ferrisetw/src/trace.rs` | ferrisetw `UserTrace`, `TraceBuilder::start()` (line 507), `start_and_process()` (line 636) |
| `etw_learning_resources/ferrisetw/src/native/evntrace.rs` | `enable_provider()` (line 234), `control_trace()` (line 303), `control_trace_by_name()` (line 332), types |
| `etw_learning_resources/ferrisetw/src/native/etw_types.rs` | `EventTraceProperties` struct (line 193), `TRACE_NAME_MAX_CHARS` (200) |
| `etw_learning_resources/krabsetw/krabs/krabs/ut.hpp` | `ut::enable_rundown()` (line 204) — the C++ implementation to replicate |
| `etw_learning_resources/krabsetw/krabs/krabs/etw.hpp` | `process_trace()` (line 369) — shows timing: enable_rundown → ProcessTrace |
| `etw_learning_resources/krabsetw/krabs/krabs/provider.hpp` | `provider::enable_rundown_events()` (line 555) — sets flag |
| `etw_learning_resources/freebasic.net ... .md` | FreeBasic example showing raw `EnableTraceEx2(CAPTURE_STATE)` call at lines 686-695 |
| `src/providers/kernel_file.rs` | `build_provider()` — our provider builder, returns `Provider` |
| `src/provider_event.rs` | `ProviderEvent` enum — our unified event type |
