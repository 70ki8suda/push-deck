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
