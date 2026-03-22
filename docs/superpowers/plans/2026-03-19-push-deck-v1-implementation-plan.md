# Push Deck V1 Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working macOS-only Push Deck app that can detect one Push 3, light pads, launch or focus apps, send bounded keyboard shortcuts to the frontmost app, and edit a fixed 8x8 layout through a Tauri UI.

**Architecture:** The app uses a Tauri shell with a Rust core. The Rust side owns config persistence, runtime state, Push device I/O, and action execution. The frontend owns editor UI state and talks to Rust only through explicit commands and events.

**Tech Stack:** Tauri 2, Rust, TypeScript, Vite, macOS app integration APIs, MIDI I/O crate, serde-based config persistence

---

## File Structure

The implementation should converge on this layout:

- `src-tauri/Cargo.toml`
  Rust workspace entrypoint and dependencies.
- `src-tauri/src/main.rs`
  Tauri bootstrap and app startup wiring.
- `src-tauri/src/lib.rs`
  Public Rust module wiring for Tauri commands and runtime initialization.
- `src-tauri/src/app_state.rs`
  Canonical runtime state model, capability flags, and event payload types.
- `src-tauri/src/config/mod.rs`
  Config load/save orchestration.
- `src-tauri/src/config/schema.rs`
  Config, profile, pad binding, and shortcut types.
- `src-tauri/src/config/store.rs`
  Disk persistence, schema version handling, atomic save, and recovery flow.
- `src-tauri/src/device/mod.rs`
  Push device service interface and wiring.
- `src-tauri/src/device/discovery.rs`
  Single-device discovery and binding policy.
- `src-tauri/src/device/push3.rs`
  MIDI input/output handling and LED rendering.
- `src-tauri/src/device/colors.rs`
  `PadColorId` to device color mapping.
- `src-tauri/src/actions/mod.rs`
  Action execution dispatch.
- `src-tauri/src/actions/launch_or_focus.rs`
  App launch/focus implementation.
- `src-tauri/src/actions/send_shortcut.rs`
  Shortcut execution and permission-aware handling.
- `src-tauri/src/macos/mod.rs`
  Shared macOS helpers and capability detection.
- `src-tauri/src/display/mod.rs`
  `DisplayAdapter` trait and `NoopDisplayAdapter`.
- `src-tauri/src/commands.rs`
  Tauri command handlers for UI interactions.
- `src-tauri/src/events.rs`
  Event emission helpers from Rust to frontend.
- `src/main.ts`
  Frontend entrypoint.
- `src/App.tsx`
  App shell layout and top-level data loading.
- `src/features/editor/EditorPage.tsx`
  Main editor screen composition.
- `src/features/editor/GridView.tsx`
  8x8 pad grid.
- `src/features/editor/DetailPanel.tsx`
  Selected-pad editor.
- `src/features/editor/AppPicker.tsx`
  App selection UI.
- `src/features/editor/ShortcutEditor.tsx`
  Bounded shortcut input UI.
- `src/features/status/StatusBar.tsx`
  Device state, save state, capability state surface.
- `src/lib/api.ts`
  Typed Tauri command/event wrapper.
- `src/lib/types.ts`
  Frontend copies of config/runtime payloads if needed.
- `tests/`
  Rust integration tests and frontend unit tests, organized by subsystem.

Plan tasks below assume new files can be created as needed under those boundaries, but workers must not collapse responsibilities into larger catch-all files without updating the plan.

The following files are serialized integration points and should be treated as controller-owned unless the active task explicitly claims them:

- `src-tauri/src/lib.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/config/schema.rs`
- `src-tauri/src/actions/mod.rs`
- `src-tauri/src/macos/mod.rs`
- `src/lib/types.ts`
- `src/App.tsx`

Do not run parallel worker tasks that all need to edit these files at the same time.

## Chunk 1: Bootstrap And Contracts

### Task 1: Scaffold the Tauri and frontend workspace

**Files:**
- Create: `package.json`
- Create: `pnpm-lock.yaml` or equivalent lockfile
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `index.html`
- Create: `src/main.ts`
- Create: `src/App.tsx`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

- [ ] **Step 1: Initialize the frontend package manifest**

Create a minimal Tauri + Vite + TypeScript package manifest with scripts for `dev`, `build`, `lint`, and `test`.

- [ ] **Step 2: Initialize the Rust package manifest**

Create the Tauri Rust crate with the minimum dependencies needed to compile an empty shell.

- [ ] **Step 3: Wire the minimal frontend entry**

Render a placeholder app shell that proves the Tauri window can boot.

- [ ] **Step 4: Wire the minimal Rust/Tauri entry**

Boot a Tauri app that opens the frontend and exposes no commands yet.

- [ ] **Step 5: Add a narrow bootstrap verification command**

Prefer one stable command such as `pnpm tauri build --debug` or equivalent bootstrap verification that the rest of the plan can reuse.

- [ ] **Step 6: Run bootstrap verification**

Run the narrowest command that proves the workspace compiles.

- [ ] **Step 7: Commit**

Commit message: `feat: bootstrap tauri workspace`

### Task 2: Define canonical shared types and runtime contracts

**Files:**
- Create: `src-tauri/src/app_state.rs`
- Create: `src-tauri/src/events.rs`
- Create: `src-tauri/src/display/mod.rs`
- Create: `src/lib/types.ts`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing tests for state and event serialization**

Add Rust tests that assert the runtime state payloads and event payloads serialize into the expected shape.

- [ ] **Step 2: Implement the Rust runtime state model**

Define app states, capability flags, event payloads, and the `DisplayAdapter` / `NoopDisplayAdapter` boundary from the spec.

- [ ] **Step 3: Mirror the necessary payload types on the frontend**

Add frontend types only for the payloads the UI will consume directly.

- [ ] **Step 4: Run the new tests**

Run the Rust test target for this module only.

- [ ] **Step 5: Commit**

Commit message: `feat: add runtime state and event contracts`

## Chunk 2: Config And Persistence

### Task 3: Implement config schema and defaults

**Files:**
- Create: `src-tauri/src/config/schema.rs`
- Create: `src-tauri/src/config/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/config_schema.rs`

- [ ] **Step 1: Write failing tests for schema normalization**

Cover:
- default profile generation
- 64-pad normalization
- `unassigned` pads
- invalid `padId` rejection
- invalid shortcut modifier/key rejection

- [ ] **Step 2: Implement schema types**

Add Rust types for config, profiles, pad bindings, colors, action payloads, and bounded shortcut values.

- [ ] **Step 3: Implement normalization helpers**

Add helpers that generate the default config and normalize partial input into a 64-pad profile.

- [ ] **Step 4: Run schema tests**

Run only the new config schema tests.

- [ ] **Step 5: Commit**

Commit message: `feat: add config schema and defaults`

### Task 4: Implement config store, atomic save, and recovery flow

**Files:**
- Create: `src-tauri/src/config/store.rs`
- Modify: `src-tauri/src/config/mod.rs`
- Modify: `src-tauri/src/app_state.rs`
- Test: `src-tauri/tests/config_store.rs`

- [ ] **Step 1: Write failing tests for persistence behavior**

Cover:
- first launch missing config file creates `Default`
- valid config load succeeds
- broken JSON enters recovery mode
- atomic save preserves prior file on failure
- backup naming on corrupt config

- [ ] **Step 2: Implement storage path resolution**

Target `~/Library/Application Support/push-deck/config.json` on macOS.

- [ ] **Step 3: Implement load/save/recovery behavior**

Follow the spec exactly, including broken-file backup behavior and recovery-required state.

- [ ] **Step 4: Expose the recovery state to the rest of the app**

Return typed outcomes the runtime can surface without frontend-specific branching in the store layer.

- [ ] **Step 5: Run config store tests**

Run only the persistence tests.

- [ ] **Step 6: Commit**

Commit message: `feat: add config store and recovery flow`

## Chunk 3: Device Layer

### Task 5: Implement device discovery policy and service interface

**Files:**
- Create: `src-tauri/src/device/mod.rs`
- Create: `src-tauri/src/device/discovery.rs`
- Modify: `src-tauri/src/app_state.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/device_discovery.rs`

- [ ] **Step 1: Write failing tests for discovery policy**

Cover:
- no device found
- one Push 3 found
- multiple candidate devices binds first match
- discovery state is emitted correctly

- [ ] **Step 2: Implement the device service trait and discovery policy**

Model one active Push 3 device at a time and expose a typed runtime-facing interface.

- [ ] **Step 3: Emit runtime state updates**

Wire discovery outcomes into the canonical app state and capability/event layer.

- [ ] **Step 4: Run discovery tests**

Run the device discovery tests only.

- [ ] **Step 5: Commit**

Commit message: `feat: add push device discovery service`

### Task 6: Implement Push 3 pad input and LED output plumbing

**Files:**
- Create: `src-tauri/src/device/push3.rs`
- Create: `src-tauri/src/device/colors.rs`
- Modify: `src-tauri/src/device/mod.rs`
- Test: `src-tauri/tests/push3_leds.rs`

- [ ] **Step 1: Write failing tests for color mapping and pad event mapping**

Cover:
- every `PadColorId` maps to a device color value
- `r0c0` through `r7c7` map consistently to device coordinates
- inbound pad messages resolve to the correct `padId`

- [ ] **Step 2: Implement the discrete LED palette mapping**

Keep the stored color model and the device output model aligned through one mapping module.

- [ ] **Step 3: Implement pad input decoding and LED output rendering**

Add the minimum device code needed to decode pad presses and repaint the full grid.

- [ ] **Step 4: Run device LED tests**

Run only the pad/LED tests.

- [ ] **Step 5: Commit**

Commit message: `feat: add push input and led rendering`

## Chunk 4: Action Execution

### Task 7: Implement launch-or-focus app action

**Files:**
- Create: `src-tauri/src/actions/mod.rs`
- Create: `src-tauri/src/actions/launch_or_focus.rs`
- Create: `src-tauri/src/macos/mod.rs`
- Modify: `src-tauri/src/config/schema.rs`
- Test: `src-tauri/tests/launch_or_focus.rs`

- [ ] **Step 1: Write failing tests for action dispatch and failure shaping**

Cover:
- valid action dispatch
- unresolved bundle id
- app-not-found error surface

- [ ] **Step 2: Implement the action dispatcher skeleton**

Dispatch from action payloads to action-specific modules without leaking Tauri or device concerns into the action layer.

- [ ] **Step 3: Implement launch-or-focus behavior**

Use bundle-id-based behavior and surface typed failures for logging/UI.

- [ ] **Step 4: Run launch-or-focus tests**

Run the action tests for this module only.

- [ ] **Step 5: Commit**

Commit message: `feat: add launch or focus action`

### Task 8: Implement bounded shortcut action and capability handling

**Files:**
- Create: `src-tauri/src/actions/send_shortcut.rs`
- Modify: `src-tauri/src/actions/mod.rs`
- Modify: `src-tauri/src/macos/mod.rs`
- Modify: `src-tauri/src/app_state.rs`
- Test: `src-tauri/tests/send_shortcut.rs`

- [ ] **Step 1: Write failing tests for shortcut execution and permission behavior**

Cover:
- valid shortcut dispatch
- missing accessibility permission
- missing frontmost target
- invalid shortcut payload rejected before execution

- [ ] **Step 2: Implement permission-aware shortcut execution**

Treat accessibility as a capability flag, not a global app state.

- [ ] **Step 3: Emit capability updates**

Expose shortcut availability to the frontend and runtime logs.

- [ ] **Step 4: Run shortcut tests**

Run the action tests for this module only.

- [ ] **Step 5: Commit**

Commit message: `feat: add shortcut action handling`

## Chunk 5: Tauri Command Layer And Frontend

### Task 9: Implement Rust Tauri command handlers

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Write failing tests for command handlers**

Cover:
- load current config
- update pad binding
- trigger test action
- restore default config in recovery mode

- [ ] **Step 2: Implement Rust command handlers**

Wire commands into config store, runtime state, and action execution without embedding UI logic into Rust.

- [ ] **Step 3: Run command tests**

Run only the command-layer tests.

- [ ] **Step 4: Commit**

Commit message: `feat: add tauri rust commands`

### Task 10: Implement typed frontend API wrapper

**Files:**
- Create: `src/lib/api.ts`
- Modify: `src/lib/types.ts`
- Test: `src/lib/api.test.ts`

- [ ] **Step 1: Write failing tests for frontend API helpers**

Cover:
- typed invoke wrapper shapes
- event subscription cleanup
- recovery-state payload handling

- [ ] **Step 2: Implement the typed API wrapper**

Wrap `invoke` and event subscription calls in one small module with no UI rendering concerns.

- [ ] **Step 3: Run API tests**

Run only the frontend API tests.

- [ ] **Step 4: Commit**

Commit message: `feat: add typed frontend api`

### Task 11: Build the editor shell and status surfaces

**Files:**
- Create: `src/features/editor/EditorPage.tsx`
- Create: `src/features/editor/GridView.tsx`
- Create: `src/features/editor/DetailPanel.tsx`
- Create: `src/features/editor/RecoveryPanel.tsx`
- Create: `src/features/status/StatusBar.tsx`
- Modify: `src/App.tsx`
- Test: `src/features/editor/*.test.tsx`

- [ ] **Step 1: Write failing component tests**

Cover:
- 8x8 grid rendering
- selected pad highlighting
- status bar state rendering
- disabled shortcut capability surface
- recovery mode hides normal editor actions and shows only restore flow

- [ ] **Step 2: Implement the editor shell**

Render the status bar, grid, and detail panel with mockable typed data.

- [ ] **Step 3: Add recovery-mode UI gating**

Render a locked-down recovery panel when the runtime enters `config_recovery_required`.

- [ ] **Step 4: Connect the shell to the typed API**

Load runtime/config data and refresh state on command/event updates.

- [ ] **Step 5: Run frontend component tests**

Run only the editor/status test target.

- [ ] **Step 6: Commit**

Commit message: `feat: add editor shell and status ui`

### Task 12: Build the detail panel editors

**Files:**
- Create: `src/features/editor/AppPicker.tsx`
- Create: `src/features/editor/ShortcutEditor.tsx`
- Modify: `src/features/editor/DetailPanel.tsx`
- Modify: `src/lib/types.ts`
- Test: `src/features/editor/DetailPanel.test.tsx`

- [ ] **Step 1: Write failing tests for detail panel editing**

Cover:
- clear binding
- app picker selection
- shortcut modifier normalization
- invalid shortcut input blocked
- test action button disabled for `unassigned`

- [ ] **Step 2: Implement the app picker and shortcut editor**

Keep the shortcut editor bounded to the spec's allowed key set.

- [ ] **Step 3: Persist edits through the command layer**

Update the selected pad and confirm the grid reflects the saved color and label.

- [ ] **Step 4: Run detail panel tests**

Run only the detail panel-related tests.

- [ ] **Step 5: Commit**

Commit message: `feat: add pad detail editors`

## Chunk 6: Integration, Packaging, And Verification

### Task 13: Integrate runtime flows, save-failed handling, and menubar behavior

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/App.tsx`
- Modify: `src/features/editor/EditorPage.tsx`
- Modify: `src/lib/api.ts`
- Modify: `src/lib/types.ts`
- Test: `src-tauri/tests/runtime_integration.rs`
- Test: `src/features/editor/AppShell.test.tsx`
- Test: `src/features/editor/EditorPage.test.tsx`
- Test: `src/lib/api.test.ts`

- [ ] **Step 1: Write failing integration tests for startup paths**

Cover:
- first launch
- waiting for device
- ready with capability unavailable
- config recovery required
- transient save failed state and recovery to prior stable state

- [ ] **Step 2: Implement startup orchestration**

Wire config load, device startup, capability detection, and event emission together.

- [ ] **Step 3: Implement transient save-failed handling**

Surface `save_failed` to the UI and return to the prior stable state after successful retry.

- [ ] **Step 4: Add menubar-oriented app shell behavior**

Keep the runtime alive when the settings window closes.

- [ ] **Step 5: Run runtime integration tests**

Run the Rust integration tests for startup/runtime behavior.

- [ ] **Step 6: Commit**

Commit message: `feat: integrate runtime startup and menubar shell`

### Task 14: Add top-level developer verification commands

**Files:**
- Create: `Makefile`
- Modify: `package.json`
- Modify: `README.md`

- [ ] **Step 1: Add stable developer entrypoints**

At minimum, provide `make test`, `make lint`, and `make dev` or equivalent stable commands.

- [ ] **Step 2: Document the commands**

Update the README with the expected local workflow for humans and agents.

- [ ] **Step 3: Run the verification entrypoints**

Run the implemented test/lint commands and confirm they work from repo root.

- [ ] **Step 4: Commit**

Commit message: `chore: add developer workflow commands`

### Task 15: Run end-to-end verification and integration review

**Files:**
- Modify: `README.md`
- Create: `docs/superpowers/plans/verification-notes.md`

- [ ] **Step 1: Run the full repo verification**

Run the top-level test and lint commands plus the narrowest available build command.

- [ ] **Step 2: Perform a manual verification pass**

Check:
- app boots
- config loads
- recovery state is visible
- one Push 3 can bind
- LED colors render
- launch/focus works
- shortcut capability surface works
- editor saves and reloads

- [ ] **Step 3: Record verification notes**

Write the command outputs and remaining known risks into `docs/superpowers/plans/verification-notes.md`.

- [ ] **Step 4: Commit**

Commit message: `chore: record v1 verification notes`
