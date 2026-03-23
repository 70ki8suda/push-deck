# Push Deck CoreMIDI Input Runtime Events Plan

Date: 2026-03-22

## Goal

Subscribe to the Push 3 User Port through CoreMIDI during app startup and emit runtime pad events from real incoming note-on and note-off messages.

## Scope

- Add a device input module that can:
  - find the Push 3 User Port source,
  - connect a CoreMIDI input port,
  - decode incoming MIDI 1.0 note data into Push 3 pad events,
  - emit runtime events through the shared Tauri event channel.
- Extend the runtime event model to represent pad release events in addition to pad press events.
- Start the input subscription during app bootstrap after runtime discovery succeeds.
- Add focused tests for:
  - runtime event serialization,
  - MIDI word decoding into pad press/release events,
  - startup behavior when no User Port source is available.

## File Ownership

- Modify: `src-tauri/src/device/mod.rs`
- Add: `src-tauri/src/device/input.rs`
- Modify: `src-tauri/src/events.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tests/runtime_contracts.rs`
- Add or modify focused Rust tests for the new input module
- Modify: `src/lib/types.ts`

## Non-Goals

- No outbound live LED transmission yet.
- No action dispatch from hardware pad presses yet.
- No hot-plug reconnect loop.
- No UI redesign.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test runtime_contracts`
- `cargo test --manifest-path src-tauri/Cargo.toml --test runtime_integration`
- focused Rust tests for the new input module

## Commit

- `feat: subscribe to push3 user port runtime events`
