# FFI Interface Boundary — INDI / ASCOM

**Owner:** TPT Edge Agent (Rust) · **Spec ref:** Phase 1, Phase 2 FFI bindings.

The Edge Agent binds to INDI (Linux, C) and ASCOM (Windows, C++/COM) drivers
via `unsafe` Rust FFI. This document defines the stable Rust-facing boundary
so the rest of the agent never touches `unsafe` directly.

## Safety Contract

1. **Single unsafe layer.** All `extern "C"` declarations live in `edge-agent/crates/ffi`.
   Callers use safe wrappers only.
2. **Ownership.** C-allocated handles are wrapped in `#[derive(Debug)]` RAII guards
   implementing `Drop` to free the resource. No manual free elsewhere.
3. **Validation.** Every pointer returned across the boundary is null-checked and
   length-checked before use. Strings are converted via `CStr::from_bytes_until_nul`.
4. **No untrusted input into C calls.** Command params are validated (range-checked
   RA/Dec, positive steps) **before** the FFI call.
5. **Concurrency.** Each device driver is accessed through a `Mutex`/`channel`;
   the C library is assumed non-reentrant unless documented otherwise.
6. **Error mapping.** C return codes map to `Result<_, DeviceError>` — never panic
   across the boundary.

## Logical Device Interface (Rust trait)

```rust
trait Device {
    fn connect(&mut self) -> Result<(), DeviceError>;
    fn disconnect(&mut self) -> Result<(), DeviceError>;
}

trait Mount: Device {
    fn slew(&self, ra: f64, dec: f64, epoch: Epoch) -> Result<(), DeviceError>;
    fn stop(&self) -> Result<(), DeviceError>;
    fn read_encoders(&self) -> Result<MountState, DeviceError>;
}

trait Focuser: Device {
    fn move_to(&self, position: u32) -> Result<(), DeviceError>;
    fn move_relative(&self, delta: i32) -> Result<(), DeviceError>;
    fn position(&self) -> Result<FocuserState, DeviceError>;
}

trait Weather: Device {
    fn sample(&self) -> Result<WeatherSample, DeviceError>;
}
```

`Epoch` is a `#[non_exhaustive] enum { J2000, JNow }`.

## C ABI Sketch (per backend)

```c
/* indi.h */
typedef struct indi_handle indi_handle_t;
indi_handle_t* indi_connect(const char* device);
void           indi_disconnect(indi_handle_t*);
int            indi_mount_slew(indi_handle_t*, double ra, double dec);
int            indi_mount_read(indi_handle_t*, double* ra, double* dec, int* status);
```

Rust wraps these in `edge-agent/crates/ffi/src/indi.rs` with `#[repr(C)]`
mirrors and `unsafe` blocks confined to that module.

## Telescope / ASCOM note

ASCOM is COM-based (Windows). On the Intel NUC path, the agent uses a thin
C++/WinRT shim exposing the same C ABI as INDI; the Rust `ffi` crate selects
the backend at compile time via `cfg(feature = "ascom")`.

## Open Questions

- Driver discovery / hotplug events.
- Timeout & watchdog for unresponsive drivers.
