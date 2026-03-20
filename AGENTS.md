# AGENTS.md

## Purpose

This repository is intended to be developed with Codex as a multi-agent system, not as a single long-running assistant thread.

The goal is to maximize parallel throughput without creating overlapping edits, scope drift, or review debt.

## Default Operating Model

- One controller agent owns the spec, plan, task ordering, integration, and final verification.
- One worker agent owns one implementation task at a time.
- One spec reviewer checks that a completed task matches the spec and plan.
- One code reviewer checks code quality, tests, regressions, and maintainability.
- One verifier runs the full repo-level verification at integration checkpoints.

Do not skip the controller role. Workers should not self-assign adjacent work.

## Source Of Truth

Implementation must follow these documents in order:

1. `docs/superpowers/specs/2026-03-18-push-deck-design.md`
2. The active file under `docs/superpowers/plans/`
3. This `AGENTS.md`

If these conflict, the spec wins over the plan, and the plan wins over ad hoc worker decisions.

## Task Ownership

Each task must name an explicit file ownership boundary before implementation starts.

Good boundaries for this repository:

- Rust device core
- Layout store and config schema
- Action engine
- macOS integration
- Tauri UI
- Runtime state/event bridge
- Packaging and app shell

Avoid assigning multiple workers to the same file set in parallel.

The following files are controller-owned integration points unless a specific plan task says otherwise:

- `src-tauri/src/lib.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/config/schema.rs`
- `src-tauri/src/actions/mod.rs`
- `src-tauri/src/macos/mod.rs`
- `src/lib/types.ts`
- `src/App.tsx`

Workers may read these files freely, but should not modify them in parallel across multiple tasks.

## Branching

- Main integration branch: `main`
- Feature branch prefix: `codex/`
- Preferred pattern: one worktree or branch per active worker task

Examples:

- `codex/device-core-bootstrap`
- `codex/layout-store-schema`
- `codex/tauri-editor-shell`

Do not start implementation directly on `main` unless the user explicitly asks for it.

## Worker Rules

- A worker receives one bounded task.
- A worker should edit only the files named in that task, plus tightly-related tests/docs.
- A worker must not broaden scope without sending the concern back to the controller.
- A worker should leave unrelated failures untouched unless the task explicitly includes them.
- A worker should run the narrowest relevant verification first, then broader verification if needed.

## Review Gates

Every completed task goes through two checks before integration:

1. Spec review
2. Code quality review

The controller should not integrate task output until both checks pass or the remaining feedback is explicitly accepted as non-blocking.

## Parallelization Rules

Parallelize only across truly independent domains.

Safe examples:

- Device discovery work and Tauri layout mock work
- Layout schema work and macOS action execution work
- UI component work and packaging docs

Unsafe examples:

- Two workers changing the same runtime state file
- Two workers both redefining action payload types
- Two workers both changing Tauri-to-Rust command/event contracts

When in doubt, sequence the tasks.

## Required Repo Structure

Keep these folders as stable anchors for agent work:

- `docs/superpowers/specs/`
- `docs/superpowers/plans/`
- `src-tauri/`
- `src/`
- `tests/`

When new directories are introduced, the plan should mention why.

## Verification Contract

Before a task is considered complete, the task owner or verifier must record:

- What command was run
- What scope it covered
- Whether it passed
- Any remaining unverified risk

Prefer stable entrypoints such as:

- `make test`
- `cargo test`
- `pnpm test`
- `pnpm lint`

If these commands do not exist yet, add them early in the project.

## Commit Style

Prefer small, scoped commits.

Examples:

- `feat: bootstrap tauri and rust workspace`
- `feat: add layout profile schema and persistence`
- `feat: implement launch-or-focus app action`
- `test: add device service connection coverage`

Do not bundle unrelated subsystems into one commit.

## Escalation Rules

Escalate back to the controller when:

- The spec seems incomplete or contradictory
- A task requires crossing into another worker's owned files
- A boundary in the plan no longer makes sense
- A test failure suggests a wider architectural issue
- macOS or Push 3 behavior differs from the assumptions in the spec

## Initial Execution Order

Until a newer plan replaces this guidance, use this order:

1. Bootstrap Tauri/Rust workspace
2. Establish config schema and layout store
3. Establish runtime state and command/event boundary
4. Add Push device discovery and LED plumbing
5. Add app launch/focus action
6. Add shortcut action with permission handling
7. Build Tauri editor UI
8. Integrate and verify end-to-end
