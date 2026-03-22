# push-deck

Ableton Push 3 を macOS 上で Stream Deck のように使うためのデスクトップアプリ構想です。

現在は設計フェーズで、詳細な spec は `docs/superpowers/specs/2026-03-18-push-deck-design.md` にあります。

## Goal

- Push 3 の Pad を固定 8x8 レイアウトのランチャーとして使う
- アプリ起動または前面化
- 現在フォーカス中アプリへのキーボードショートカット送出
- Push 本体では Pad LED によるフィードバック

## Planned Stack

- Tauri
- Rust
- macOS native integration
- MIDI-based Push 3 control

## Bootstrap

- `npm install`
- `npm run build`
- `npm run lint`
- `cargo check --manifest-path src-tauri/Cargo.toml`

## Developer Workflow

- `make dev` starts the Tauri app in development mode.
- `make lint` runs the frontend typecheck and a Rust compile check.
- `make test` runs the frontend test suite and the Rust test suite.
- `make build` runs the frontend build and Rust build.
- `make check` runs `make lint` and `make test`.

The `npm` scripts are still available if you want to work on a narrower layer:

- `npm run dev` starts the Vite frontend only.
- `npm run dev:app` starts the Tauri app.
- `npm run build` builds the frontend bundle.
- `npm run lint` checks the frontend types.
- `npm test` runs the frontend tests.
- `cargo check --manifest-path src-tauri/Cargo.toml` checks the Rust app.
- `cargo test --manifest-path src-tauri/Cargo.toml` runs the Rust test targets.

The latest controller verification summary lives in `docs/superpowers/plans/verification-notes.md`.
