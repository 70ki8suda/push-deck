# User Mode Fast Resume Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Detect Push 3 `User Mode` transitions directly from CoreMIDI and use them to re-acquire Push Deck input/LED control faster than the current runtime refresh path.

**Architecture:** Add a dedicated device mode watcher that decodes the observed `User Mode` button CC and mode-confirming SysEx messages from the Push 3 Live/User ports. Integrate that watcher into the Tauri runtime so `UserModeButtonPressed` triggers a fast resume path that re-subscribes to the user input port and re-syncs LEDs, while preserving the existing discovery-based refresh path as fallback.

**Tech Stack:** Rust, Tauri 2, CoreMIDI, serde, Rust integration tests

---

## Chunk 1: Mode Signal Decoding

### Task 1: Add a dedicated Push mode decoder and tests

**Files:**
- Create: `src-tauri/src/device/mode.rs`
- Modify: `src-tauri/src/device/mod.rs`
- Test: `src-tauri/tests/device_input.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for:
- `B0 3B 7F` => `UserModeButtonPressed`
- `B0 3B 00` => `UserModeButtonReleased`
- `F0 00 21 1D 01 01 0A 01 F7` => `UserModeEntered`
- `F0 00 21 1D 01 01 0A 00 F7` => `UserModeExited`
- unrelated bytes => `None`

- [ ] **Step 2: Run the focused test target and verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test device_input`
Expected: FAIL because the mode decoder types/functions do not exist yet.

- [ ] **Step 3: Implement the minimal mode decoder**

Add `PushModeEvent` plus helpers that decode the observed CC and SysEx payloads without touching pad note decoding.

- [ ] **Step 4: Re-run the focused test target**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test device_input`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/device/mode.rs src-tauri/src/device/mod.rs src-tauri/tests/device_input.rs
git commit -m "feat: add push user mode signal decoder"
```

## Chunk 2: Fast Resume Runtime Wiring

### Task 2: Wire a mode watcher and fast resume path into the runtime

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/device/input.rs`
- Modify: `src-tauri/src/device/mod.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/tests/runtime_integration.rs`

- [ ] **Step 1: Write the failing runtime tests**

Add tests for:
- a `UserModeButtonPressed` event triggers a fast resume attempt
- a successful fast resume re-syncs LEDs
- a failed fast resume falls back cleanly instead of panicking

- [ ] **Step 2: Run the focused runtime test target and verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test runtime_integration`
Expected: FAIL because fast resume hooks do not exist yet.

- [ ] **Step 3: Add runtime helpers for fast resume**

Implement:
- a reusable helper to replace the stored Push input subscription
- a reusable helper to sync LEDs from current config
- a fast resume entrypoint that re-subscribes to the user port and re-syncs LEDs

- [ ] **Step 4: Add the mode watcher subscription**

Subscribe to the Push 3 Live/User ports at startup, decode mode events, and call the fast resume path on `UserModeButtonPressed`. Keep existing startup discovery and runtime refresh logic intact.

- [ ] **Step 5: Re-run the focused runtime test target**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test runtime_integration`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/device/input.rs src-tauri/src/device/mod.rs src-tauri/src/commands.rs src-tauri/tests/runtime_integration.rs
git commit -m "feat: fast resume on push user mode"
```

## Chunk 3: Full Verification

### Task 3: Verify the integrated behavior and leave the repo in a clean state

**Files:**
- Modify: `docs/superpowers/plans/verification-notes.md` (only if verification notes are being recorded there already)

- [ ] **Step 1: Run the narrow regression targets**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test device_input`
- `cargo test --manifest-path src-tauri/Cargo.toml --test runtime_integration`

Expected: PASS.

- [ ] **Step 2: Run the broader Rust suite for affected areas**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, or capture any unrelated existing failures explicitly.

- [ ] **Step 3: Record remaining risk**

Note that manual hardware verification is still required for the real Live -> User Mode latency improvement, even if automated tests pass.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/plans/verification-notes.md
git commit -m "test: record fast resume verification"
```
