# Push Deck

Push Deck is a macOS desktop app that turns an Ableton Push 3 into a fixed 8x8 control surface, similar in spirit to a Stream Deck.

## Current Status

This is an early release-oriented preview, not a finished product.

What is already implemented:

- A Tauri-based desktop app with a React editor UI.
- Push 3 device discovery and runtime status reporting.
- A single 8x8 layout editor with per-pad labels, colors, and actions.
- Two action types: launch/focus an app by bundle ID, or send a shortcut to the frontmost app.
- Configuration persistence, validation, and recovery mode for unreadable config files.
- Pad action testing from the editor.

What is not implemented yet is listed in [Current Limitations](#current-limitations).

## Requirements

To use the app today, you need:

- macOS.
- An Ableton Push 3 connected over MIDI/CoreMIDI.
- Accessibility permission for shortcut injection.
- A Rust toolchain and Node.js/npm for development.

The app stores its config at `~/Library/Application Support/push-deck/config.json`.

## Quick Start

For development:

1. Install dependencies with `npm install`.
2. Start the desktop app with `make dev`.
3. Run checks with `make lint` and `make test`.
4. Build release artifacts with `make build`.

Useful narrower commands:

- `npm run dev` starts the Vite frontend only.
- `npm run dev:app` starts the Tauri app.
- `npm run build` builds the frontend bundle.
- `npm run lint` type-checks the frontend.
- `npm test` runs the frontend tests.
- `cargo check --manifest-path src-tauri/Cargo.toml` checks the Rust app.
- `cargo test --manifest-path src-tauri/Cargo.toml` runs the Rust tests.

## Current Capabilities

- Edit a single active 8x8 Push layout.
- Assign each pad a label, a color, and one action.
- Bind a pad to launch or focus an app by bundle ID.
- Bind a pad to send a keyboard shortcut.
- Save changes and reload them on restart.
- Restore defaults from recovery mode if the config file becomes unreadable.
- See device connection and runtime state in the UI.

## Current Limitations

- macOS only.
- One profile only.
- Fixed 8x8 layout only.
- No page switching or mode switching.
- No long press, double tap, chaining, or macro system.
- No Push screen icons or labels yet.
- No advanced coexistence tuning for Ableton Live.
- Shortcut actions depend on Accessibility permission.

## Spec

The implementation spec lives at [docs/superpowers/specs/2026-03-18-push-deck-design.md](/Users/yasudanaoki/Downloads/push-deck/.worktrees/task-19-readme-release/docs/superpowers/specs/2026-03-18-push-deck-design.md).
