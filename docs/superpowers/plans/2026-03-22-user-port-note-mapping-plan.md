# Push Deck User Port Note Mapping Plan

Date: 2026-03-22

## Goal

Replace the placeholder Push 3 pad transport mapping with the real note-based mapping observed from the Push 3 User Port on macOS, so pad decode and LED rendering share the same hardware note coordinates.

## Scope

- Update the Push 3 transport model from synthetic row-major indices to note-based transport values.
- Encode the observed User Port pad grid, with `r0c0` at the top-left and `r7c7` at the bottom-right.
- Decode inbound `note on` and `note off` style pad messages into logical pad ids.
- Render outbound LED commands against the same note mapping.
- Add focused tests for the measured corner notes and the full 8x8 round-trip mapping.

## Observed Hardware Reference

The Push 3 User Port reported these measured notes during live capture on 2026-03-22:

- top-left pad: `0x5C`
- center-region pad sample: `0x47`
- bottom-right pad: `0x2B`

The working note formula is:

- `note = 0x24 + (7 - row) * 8 + column`

## File Ownership

- Modify: `src-tauri/src/device/push3.rs`
- Modify: `src-tauri/tests/push3_leds.rs`

## Non-Goals

- No CoreMIDI input subscription loop.
- No live LED transmission yet.
- No runtime event plumbing changes.
- No config or UI changes.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test push3_leds`

## Commit

- `feat: align push3 transport mapping with user port notes`
