# User Mode Fast Resume Design

Date: 2026-03-25

## Summary

Ableton Push 3 を Live の楽器として使っている状態から `User Mode` を押して Push Deck に戻るまでのギャップタイムを短縮する。  
既存の Push Deck は `Push 3 User Port` の再検出と runtime refresh に依存しており、`User Mode` 復帰後の input/LED 再獲得が遅い。

今回の設計では `User Mode` ボタン押下と mode 遷移を CoreMIDI 生イベントから直接検出し、その瞬間に Push Deck 側の再購読と LED 再送を開始する。

この文書は [2026-03-18-push-deck-design.md](/Users/yasudanaoki/Desktop/push-deck/docs/superpowers/specs/2026-03-18-push-deck-design.md) の addendum として扱う。

## Product Goals

- Live 演奏中から `User Mode` 押下後、Push Deck の pad 入力復帰を速くする
- `User Mode` 復帰時の LED 再点灯を速くする
- 既存の discovery/runtime refresh を壊さず、最速復帰経路を追加する

## Non-Goals

- Push 3 のすべての mode/state を網羅的に decode すること
- Live 使用中も Push Deck が常時 input/LED を保持し続けること
- UI 上で高度な mode debug 画面を追加すること

## Observed Device Signals

実機観測では `User Mode` 押下時に以下の信号が再現した。

- `B0 3B 7F`
- `B0 3B 00`
- `F0 00 21 1D 01 01 0A 01 F7`

別タイミングでは逆向きの state signal として以下も観測した。

- `F0 00 21 1D 01 01 0A 00 F7`

この設計では次の意味で扱う。

- `B0 3B 7F`
  - `UserModeButtonPressed`
- `B0 3B 00`
  - `UserModeButtonReleased`
- `F0 00 21 1D 01 01 0A 01 F7`
  - `UserModeEntered`
- `F0 00 21 1D 01 01 0A 00 F7`
  - `UserModeExited`

## Chosen Approach

`pad input` とは別に `Push mode watcher` を追加し、`Live Port` / `User Port` の生メッセージから `User Mode` 遷移だけを検出する。

### Why this approach

- 既存の pad decoder に mode 固有ロジックを混ぜずに済む
- `User Mode` の高速復帰だけを独立責務として実装できる
- 将来ほかの mode signal を足すときも境界が明確

## Architecture

### Device Mode Watcher

`src-tauri/src/device/mode.rs` を追加し、以下を担当する。

- `Ableton Push 3 Live Port`
- `Ableton Push 3 User Port`

この 2 つの CoreMIDI source を購読し、raw MIDI bytes から mode transition event を抽出する。

最小インターフェースは以下を想定する。

```rust
pub enum PushModeEvent {
    UserModeButtonPressed,
    UserModeButtonReleased,
    UserModeEntered,
    UserModeExited,
}
```

### Runtime Integration

mode watcher が `UserModeButtonPressed` を出したら、Push Deck は即時に fast resume シーケンスを始める。

1. `User Port` input 購読を張り直す
2. 成功したら現在 config を LED backend に再送する
3. `UserModeEntered` が来たら復帰成功として確定する

### Existing Runtime Fallback

既存の `refresh_runtime` と discovery ベースの復帰経路は残す。

- fast resume に失敗した場合
- `UserModeEntered` が来ない場合
- CoreMIDI mode watcher 自体が張れない場合

このいずれでも、最終的には既存 runtime refresh にフォールバックする。

## Detection Rules

### CC-based trigger

- status `0xB0`
- controller `0x3B`
- value `0x7F`

これを `UserModeButtonPressed` として扱う。

### Release signal

- status `0xB0`
- controller `0x3B`
- value `0x00`

これは release として記録してよいが、復帰開始の主トリガーにはしない。

### SysEx-based state confirmation

- `F0 00 21 1D 01 01 0A 01 F7`
  - `UserModeEntered`
- `F0 00 21 1D 01 01 0A 00 F7`
  - `UserModeExited`

fast resume は `button pressed` で先行開始し、`entered` で確定させる。

## Fast Resume State Machine

状態は単純に保つ。

```text
Idle
  -> UserModeButtonPressed
Resuming
  -> UserModeEntered => Ready
  -> retry exhausted => FallbackRefresh
Ready
  -> UserModeExited => Idle
FallbackRefresh
  -> runtime refresh success => Ready or WaitingForDevice
```

### Timing rules

- `UserModeButtonPressed` 後すぐに再購読を試みる
- 再購読失敗時は短い backoff で数回だけ再試行する
- `UserModeEntered` が短時間内に来ない場合は fast path を打ち切る

初回は最小限として、backoff と retry 回数は小さく固定値でよい。

## Responsibilities

### `device/input.rs`

- pad note の decode を引き続き担当する
- mode signal の責務は持たない

### `device/mode.rs`

- raw bytes から mode event を検出する
- Live/User Port の mode watcher subscription を保持する

### `lib.rs`

- startup 時に mode watcher を購読する
- mode event を受けて fast resume シーケンスを起動する

### `commands.rs`

- 既存 `sync_push3_leds` 相当のロジックを fast resume から再利用できるようにする
- 必要なら `User Port` 再購読ヘルパーを host/command 境界に追加する

## Error Handling

- mode watcher subscription 失敗
  - 起動は継続する
  - 既存 runtime refresh のみで動作する
- `UserModeButtonPressed` 後の `User Port` 再購読失敗
  - 短い retry を行う
  - だめなら runtime refresh へフォールバックする
- `UserModeEntered` が来ない
  - fast resume を打ち切って fallback する
- LED resync 失敗
  - input 復帰は維持しつつ warning を残す

## Testing Strategy

### Device mode tests

- `B0 3B 7F` が `UserModeButtonPressed` に decode される
- `B0 3B 00` が `UserModeButtonReleased` に decode される
- `F0 00 21 1D 01 01 0A 01 F7` が `UserModeEntered` に decode される
- `F0 00 21 1D 01 01 0A 00 F7` が `UserModeExited` に decode される
- 既知以外の bytes は無視される

### Runtime integration tests

- `UserModeButtonPressed` で fast resume が開始される
- `UserModeEntered` で LED resync が走る
- retry 枯渇時に runtime refresh fallback が呼ばれる
- `UserModeExited` で state が idle に戻る

### Manual verification

- Live 演奏中に Push Deck が inactive の状態を作る
- `User Mode` を押す
- 最初の pad press が Push Deck で反応するまでの時間を比較する
- LED 再点灯の体感時間を比較する

## Open Risks

- `CC 0x3B` が将来 firmware で別意味に変わる可能性がある
- `SysEx 0A 01/00` の意味付けは現時点では観測ベース
- Live と Push Deck のポート競合が強い環境では、再購読タイミングだけでは改善幅が足りない可能性がある

初回実装では観測済み signal のみを扱い、複雑なモード抽象化は行わない。
