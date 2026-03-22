# Push Deck CoreMIDI Discovery Follow-up Plan

Date: 2026-03-22

## Goal

Close the next highest-risk hardware gap by adding a macOS `CoreMIDI` discovery source for Push 3 ports, so runtime startup can identify the actual MIDI endpoints that later tasks will use for pad input and LED output.

## Scope

- Add a `CoreMidiDiscoverySource` implementation in the Rust device layer.
- Enumerate macOS MIDI sources and destinations through `CoreMIDI`.
- Match Push 3 ports by stable display/name heuristics, preferring the `User Port`.
- Return a representative Push 3 endpoint descriptor without regressing the current startup contract.
- Update startup orchestration to try `CoreMIDI` discovery before falling back to `system_profiler` and then `waiting_for_device`.
- Add focused tests around port matching and startup fallback order.

## File Ownership

- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/device/discovery.rs`
- Modify: `src-tauri/src/device/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tests/device_discovery.rs`
- Modify: `src-tauri/tests/runtime_integration.rs`

## Non-Goals

- No live pad input subscription yet.
- No live LED rendering yet.
- No UI changes.
- No hot-plug monitoring loop.
- No app-level runtime contract changes beyond improved device resolution.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test device_discovery`
- `cargo test --manifest-path src-tauri/Cargo.toml --test runtime_integration`

## Commit

- `feat: add coremidi discovery startup fallback`
