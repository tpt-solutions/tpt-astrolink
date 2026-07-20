# Hardware-in-the-Loop (HIL) Test Plan — INDI / ASCOM FFI Layer

**Component:** `edge-agent/crates/ffi` · **Owner:** TPT Solutions
**Purpose:** Validate the safe Rust↔C boundary to INDI/ASCOM drivers
against real hardware (mount, focuser, weather station) before fleet rollout.

> The unit tests in `crates/ffi` mock the C library; this plan covers the
> *real* drivers, which CI cannot exercise.

## 1. Scope

| Capability | C entry point | Hardware under test |
|-------------|--------------|----------------------|
| Mount slew | `indi_mount_slew` | EQ6-R / iOptron CEM40 / Pegasus |
| Mount stop | `indi_mount_stop` | same |
| Mount read | `indi_mount_read` | same (returned RA/Dec) |
| Focuser move | `indi_focuser_move` | Moonlite / Pegasus FocusCube |
| Focuser read | `indi_focuser_read` | same (returned position) |
| Weather sample | `indi_weather_sample` | Bos / OpenWeather proxy |
| Connect / disconnect | `indi_connect` / `indi_disconnect` | all |

## 2. Fixture

- Raspberry Pi 5 + target board, or Intel NUC, running the signed
  agent image from `scripts/release.sh` + `scripts/install.sh`.
- INDI server (`indiserver`) with the real driver loaded, **or** the
  ASCOM Alpaca/USB driver on Windows NUC.
- A "safe" mount with clutches disengaged and **no optics / lens cap on**
  for the first slew pass.
- A logging proxy capturing every FFI call (arguments + return code)
  from `tpt-edge-agent` for later diff review.

## 3. Procedure

### 3.1 Connectivity & lifecycle
1. Boot the node; assert `indi_connect` returns `0` and the agent
   reports `online`.
2. Power-cycle the mount mid-session; assert the agent reconnects
   within the `ffi` retry budget and does **not** panic.
3. Assert `indi_disconnect` on shutdown leaves the mount park-safe.

### 3.2 Command fidelity
For each command type, drive it from the **Client UI** (full path:
UI → WS → Cloud Core → MQTT → Edge Agent → FFI) and verify:
- The exact argument values (RA/Dec/position) reach the C driver
  (per the logging proxy).
- The driver ACKs; the returned state matches the command
  (within.driver tolerance).
- A bogus command (NaN coords, negative focuser position) is
  **rejected at the FFI boundary** and never reaches hardware.

### 3.3 Telemetry accuracy
- Slew to a known RA/Dec; after settle, compare `indi_mount_read`
  to the commanded coordinates (arcmin tolerance).
- Step the focuser by N steps; assert `indi_focuser_read` delta == N.
- Weather sample parsed into the `WeatherSample` struct with sane ranges.

### 3.4 Safety & fault injection
- Kill `indiserver` while a slew is in flight → agent must
  surface an error and move to a safe state (no hung thread).
- Send a malformed payload larger than the FFI buffer → assert the
  boundary returns an error, **not** UB / segfault.
- Run for 24 h with a cyclic slew/focus script; check for
  memory growth (RSS) and FD leaks in the agent process.

## 4. Pass / Fail criteria

| Check | Gate |
|-------|------|
| All 7 entry points exercised on real hardware | required |
| Command fidelity 100% (no arg corruption) | required |
| Bogus/buffer-overflow inputs rejected at boundary | required |
| Reconnect within retry budget | required |
| 24 h soak: no leak, no segfault | required |
| Telemetry within tolerance | required |

## 5. Sign-off

Record: hardware SKU, driver/firmware versions, agent build SHA, and the
FFI call log. Attach to the MVP release checklist
(`docs/mvp-release-checklist.md`).
