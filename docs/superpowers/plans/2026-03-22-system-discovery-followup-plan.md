# Push Deck System Discovery Follow-up Plan

Date: 2026-03-22

## Goal

Close the highest-risk gap from `verification-notes.md` by adding a macOS startup-time discovery source for Push 3 that can identify representative USB entries from `system_profiler SPUSBDataType -json`, while preserving safe startup behavior when discovery is unavailable.

## Scope

- Add a `SystemDiscoverySource` implementation in the Rust device layer.
- Parse nested `system_profiler` USB JSON for representative Push 3 entries.
- Dedupe duplicate device matches by stable endpoint identity.
- Re-export the source from the device module.
- Update startup orchestration to fall back to `waiting_for_device` when system discovery fails.
- Add focused parser tests and startup fallback tests.

## File Ownership

- Modify: `src-tauri/src/device/discovery.rs`
- Modify: `src-tauri/src/device/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tests/device_discovery.rs`
- Modify: `src-tauri/tests/runtime_integration.rs`

## Non-Goals

- No manual re-scan UI.
- No MIDI I/O changes.
- No live LED rendering changes.
- No app-level runtime contract changes beyond startup fallback.

## Verification

- `cargo test --test device_discovery`
- `cargo test --test runtime_integration`

## Commit

- `feat: add system discovery startup fallback`
