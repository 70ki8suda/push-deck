# Push Deck Design

Date: 2026-03-18

## Summary

Ableton Push 3 を macOS 上で Stream Deck のように使うデスクトップアプリを作る。  
実装方針は `Tauri app + Rust native core + MIDI device control`。初版では Push 3 の Pad 入力と LED フィードバックを使い、以下を実現する。

- アプリ起動または前面化
- 現在フォーカス中のアプリへのキーボードショートカット送出
- 固定 8x8 レイアウトの GUI 編集

将来は Push 本体画面へのアイコンやラベル表示を追加したくなる前提で、画面描画系は別アダプタとして拡張できる構造にする。

## Product Goals

- Stream Deck を持っていなくても、机上の Push 3 を日常操作デバイスとして使えるようにする
- 固定レイアウトで迷わず使えることを優先する
- Push 本体上で役割がわかる最低限のフィードバックとして Pad LED 色を使う
- 設定は GUI で行い、運用を軽くする

## Non-Goals For V1

- Push 本体画面へのアイコン表示やラベル表示
- ページ切り替え、モード切り替え
- 長押し、ダブルタップ、複数トリガー
- 複雑なマクロ、条件分岐、自動化 DSL
- Ableton Live 利用時との高度な共存最適化

## Chosen Approach

検討した案の中では、`Tauri app + Rust native core + MIDI control` を採用する。

### Why this approach

- Tauri で設定 UI と macOS 向け配布をまとめやすい
- Rust 側にデバイス制御とアクション実行を寄せると責務分離が明確になる
- 初版で必要な `Pad 入力 + LED 制御 + macOS 操作` を最短で満たせる
- 将来の Push 画面対応を `Display Adapter` として追加しやすい

## Architecture

システムは以下のレイヤーに分ける。

### 1. Tauri Shell

- 設定 UI
- 初回セットアップと権限案内
- ログ表示
- 常駐制御

### 2. Rust Device Service

- Push 3 の接続監視
- MIDI input の購読
- Pad 押下イベント生成
- LED 状態の保持と再送
- 再接続時の状態復元

### 3. Action Engine

- Pad 押下イベントからアクション実行
- `LaunchOrFocusApp`
- `SendShortcut`

### 4. Layout Store

- 8x8 固定レイアウト
- Pad ごとの色、ラベル、アクション定義
- JSON ベースの永続化

### 5. macOS Integration

- `NSWorkspace` によるアプリ起動と前面化
- `CGEvent` または同等 API によるキーボードイベント送出
- Accessibility 権限チェック

### 6. Future Display Adapter

- Push 本体画面へのアイコン描画
- ラベル表示
- 状態表示

V1 ではインターフェースだけ定義し、実装は行わない。

最小インターフェースは以下とする。

```ts
type DisplayFrame = {
  target: "main" | "top-strip";
  payload: unknown;
};

interface DisplayAdapter {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  render(frame: DisplayFrame): Promise<void>;
  clear(): Promise<void>;
}
```

V1 では `NoopDisplayAdapter` を使い、core から表示更新イベントを受けても何もしない。

### Layer Contracts

各レイヤーの責務と境界は以下で固定する。

- `Tauri Shell`
  - UI state を保持する
  - ユーザー操作を command/event 経由で Rust core に送る
  - 永続化ファイルを直接触らない
- `Rust Device Service`
  - Push デバイスとの I/O を専有する
  - Pad 押下や接続変化を domain event として発行する
  - UI レイヤーを知らない
- `Action Engine`
  - `PadBinding.action` を受けて macOS 操作へ変換する
  - デバイス I/O や永続化には触らない
- `Layout Store`
  - プロファイルの読込、保存、検証を担当する
  - JSON を扱う唯一の層とする

state flow は次で統一する。

1. UI 操作は Tauri command として Rust core へ渡す
2. Rust core は `Layout Store` に保存する
3. 保存成功後に Rust core が最新プロファイルを `Device Service` へ適用する
4. `Device Service` の接続状態や Pad 押下は event として UI に返す
5. `Action Engine` は Pad 押下 event を受けて実行する

## Action Model

初版ではアクションを 2 種類に絞る。

### LaunchOrFocusApp

- 指定アプリが起動済みなら前面化
- 未起動なら起動
- アプリ識別は表示名ではなく `bundle id` を使う

### SendShortcut

- 現在フォーカス中のアプリにショートカットを送る
- 例: `Cmd+Shift+P`
- 特定アプリを前面化してから送る複合アクションは V1 では対象外

## Data Model

```ts
type PadBinding = {
  padId: string;   // r0c0 ... r7c7
  label: string;
  color: PadColorId;
  action:
    | {
        type: "unassigned";
      }
    | {
        type: "launch_or_focus_app";
        bundleId: string;
        appName: string;
      }
    | {
        type: "send_shortcut";
        key: string;
        modifiers: string[];
      };
};

type PadColorId =
  | "off"
  | "white"
  | "red"
  | "orange"
  | "yellow"
  | "green"
  | "cyan"
  | "blue"
  | "purple"
  | "pink";

type LayoutProfile = {
  id: string;
  name: string;
  pads: PadBinding[];
};

type AppSettings = {
  activeProfileId: string;
};
```

`launchAtLogin` は V1 の planning 範囲から外す。必要なら Tauri 実装時に別タスクで追加する。

将来の Push 画面表示に備えて、必要になれば以下を追加できるようにする。

- `iconPath`
- `screenAssetId`
- `displayLabel`
- `stateStyle`

### Persistence Rules

- 永続化形式は JSON とする
- 保存場所は `~/Library/Application Support/push-deck/config.json` とする
- schema version を持ち、V1 は `schemaVersion: 1` とする
- V1 ではプロファイルは 1 つだけ持つ
- 将来の複数プロファイル対応を見据えて内部的には `activeProfileId` を持つが、永続化ファイル上では常に `default` 固定とする
- 初回起動時は `Default` プロファイルを自動生成する
- `Default` プロファイルは 64 個の固定 `padId` を持つ
- すべての Pad は常に `PadBinding` を持つ。未設定 Pad は `action: { type: "unassigned" }` を使う
- `padId` は `r0c0` から `r7c7` までの予約済み ID のみを許可する
- `r0c0` は GUI 上で左上、`r0c7` は右上、`r7c0` は左下、`r7c7` は右下とする
- 物理 Push 3 上でも GUI と同じ向きで対応付ける
- 保存時に重複 `padId`、未知の `padId`、不正な action payload を検証し、エラーなら保存しない
- `activeProfileId` が欠落または無効なら `default` にフォールバックし、それも無ければ `Default` を再生成する
- 未設定 Pad は押しても no-op とする
- 保存はテンポラリファイルへの書き込み後に rename する atomic save とする
- JSON が壊れていて読めない場合は自動上書きせず、`config recovery required` 状態で起動する
- 壊れた JSON は退避コピーを作成し、その後に `Default` プロファイルを新規生成できる導線を UI で出す
- 保存失敗時は直前の正常ファイルを維持し、UI に保存失敗を明示する

### Persistence Schema

V1 の保存ファイルは 1 つとする。

```json
{
  "schemaVersion": 1,
  "settings": {
    "activeProfileId": "default"
  },
  "profiles": [
    {
      "id": "default",
      "name": "Default",
      "pads": [
        { "padId": "r0c0", "label": "", "color": "off", "action": { "type": "unassigned" } }
      ]
    }
  ]
}
```

- `profiles` 配列は V2 の複数プロファイル対応を見据えた形だが、V1 では要素数 1 固定
- 上の `pads` は短縮例であり、実際の保存時は 64 要素を持つ
- `pads` は保存時に常に 64 要素へ正規化する
- 読込時に欠けている Pad があれば `unassigned` で補完する
- `color` は常に `PadColorId` で保存し、hex は保存しない

### LED Palette Mapping

V1 では GUI、保存、LED 送信のすべてで同じ離散パレットを使う。

- `off`
- `white`
- `red`
- `orange`
- `yellow`
- `green`
- `cyan`
- `blue`
- `purple`
- `pink`

GUI の color picker はこの 10 色だけを提示する。

### Config Recovery Flow

設定ファイルが存在しない初回起動時は recovery に入らず、`Default` を自動生成して通常起動する。

JSON が壊れている場合の挙動は次で固定する。

1. 起動時に `Layout Store` が JSON parse を試みる
2. parse に失敗したら元ファイルを `.broken-<timestamp>.json` として退避する
3. アプリは `config recovery required` 状態で起動する
4. この状態では既存プロファイル編集はできず、UI には `Restore default layout` だけを出す
5. ユーザーが復旧を選ぶと `Default` プロファイルを新規生成して保存する
6. 保存成功後に通常モードへ戻る
7. 保存失敗時は recovery 状態を維持し、再試行だけを許可する

## Runtime Flow

### Startup

1. アプリ起動
2. 設定読み込み
3. Push 3 接続待ち
4. 接続完了後に LED を全 Pad へ反映
5. UI と Device Service が同じプロファイルを参照

V1 のデバイス方針は `同時に 1 台の Push 3 のみサポート` とする。  
複数候補がある場合は最初に検出した 1 台へ bind し、UI に対象デバイス名を表示する。手動切替は V2 に送る。

### On Pad Press

1. Device Service が押下イベントを受信
2. 対応する `PadBinding` を解決
3. Action Engine がアクション実行
4. 必要なら一時的な LED フィードバックを反映

### On Config Change

1. GUI で Pad を選択
2. アクションや色を編集
3. 保存
4. Layout Store 更新
5. Device Service が即時 LED を再反映

### On Reconnect

1. Push 切断を検知
2. `waiting for device` 状態へ移行
3. 再接続時に現在プロファイルを再送

## GUI Design

V1 の編集 UI は `Grid + Detail Panel` を採用する。

### Main Layout

- 左側: 8x8 グリッド
- 右側: 選択中 Pad の詳細設定
- 上部: 接続状態、保存状態、権限状態

### Grid Behavior

- 各 Pad を色付きセルで表示
- 未設定 Pad はニュートラルカラー
- 選択中 Pad は強調表示
- V1 では GUI 上のクリックで編集対象を選ぶ
- Push 実機で Pad を押したときは編集選択には使わず、割り当て済み action の実行だけを行う

### Detail Panel Fields

- Pad label
- Action type
- App picker または shortcut editor
- Color picker
- Clear binding button
- Test action button

### App Picker

- `/Applications` などから候補を選ばせる
- 内部保存は bundle id

### Shortcut Editor

- 修飾キーの選択
- キー入力のキャプチャ
- 表示形式は `Cmd+Shift+P`

`SendShortcut` の値ルールは以下で固定する。

- `modifiers` は `["Cmd", "Shift", "Opt", "Ctrl"]` から選ぶ
- 保存時の順序は常に `Cmd`, `Shift`, `Opt`, `Ctrl` の順へ正規化する
- `key` は次の bounded subset のみ許可する
  - `A-Z`
  - `0-9`
  - `F1-F12`
  - `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`
  - `Space`, `Tab`, `Enter`, `Escape`, `Delete`
- 未対応キーは保存時に reject する

型は次の制約を前提に扱う。

```ts
type ShortcutModifier = "Cmd" | "Shift" | "Opt" | "Ctrl";
type ShortcutKey =
  | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
  | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z"
  | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
  | "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11" | "F12"
  | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight"
  | "Space" | "Tab" | "Enter" | "Escape" | "Delete";
```

### Test Action Button

- `unassigned` では無効化する
- `LaunchOrFocusApp` はその場で実行を試す
- `SendShortcut` は Accessibility 権限があり、かつフォーカス先が存在する場合のみ試行する
- 成功時は UI に短い success 表示を出す
- 失敗時は理由を明示する
  - 権限不足
  - 対象アプリ未解決
  - 不正な shortcut 定義
  - 実行時エラー

## macOS Integration Notes

### Launch or Focus

- `bundle id` ベースで対象アプリを扱う
- 起動済みなら前面化、未起動なら起動
- 名前一致は避ける
- 実行時に `bundle id` が解決できない場合は action を失敗として扱い、UI ログに `app not found` を出す

### Shortcut Injection

- Accessibility 権限が必要
- 初回セットアップ時に案内 UI を出す
- 権限未付与時はショートカットアクションだけを無効扱いにする
- `LaunchOrFocusApp`、設定編集、Push 接続監視は継続利用できる
- frontmost app が存在しない場合は action をスキップし、UI ログに `no frontmost app` を出す

### Background Behavior

- V1 は `menu bar app + settings window` とする
- 常時メニューバーに常駐し、設定画面は別ウィンドウで開く
- 設定ウィンドウを閉じても Device Service は動作を継続する

## Error Handling

- Push 未接続: UI に `waiting for device`
- 権限不足: UI に `shortcut execution unavailable`
- 不正設定: 保存前バリデーションでブロック
- 実行失敗: ログ表示。将来は Pad 赤点滅なども検討可能

## State Model

V1 のアプリ状態は以下で固定する。

- `starting`
- `waiting_for_device`
- `ready`
- `config_recovery_required`
- `save_failed`

主な遷移は以下。

- 起動直後は `starting`
- 設定が正常で Push 未接続なら `waiting_for_device`
- 設定が正常で Push 接続済みなら `ready`
- config 読込失敗時は `config_recovery_required`
- 保存失敗時は一時的に `save_failed`。再試行成功で直前の安定状態へ戻る

Accessibility 権限不足は app state ではなく capability flag として扱う。  
具体的には UI 上の `shortcut capability: unavailable` として表示し、shortcut 系 action と test だけを無効化する。

## Technical Risks

### 1. Push 3 LED control details

V1 は任意 RGB ではなく、アプリ内で定義した離散カラー パレットを使う。  
理由は、Push 側で安定した色再現を行いやすく、UI でも扱いを単純にできるため。

### 2. App focusing behavior

macOS 上で期待通り前面化されるかは、起動方法や権限で差が出る可能性がある。

### 3. Shortcut injection permissions

Accessibility 権限が必須になり得るため、初回導線を適切に設計しないと UX が悪化する。

### 4. Future display support

Push 画面の取り扱いは V1 の外に出す。  
V1 の planning 対象には含めず、`Display Adapter` は空インターフェースだけを定義する。  
画面表示プロトコルやアイコン形式の検討は、別の V2 spec で扱う。

## MVP Scope

V1 で必ず入れるものは以下。

- Push 3 の接続検知
- 8x8 Pad 押下の取得
- Pad ごとの LED 色設定
- `LaunchOrFocusApp`
- `SendShortcut`
- GUI での Pad 編集
- レイアウトの保存と読み込み
- 起動時の再接続対応
- 権限不足や未接続の状態表示

## Suggested Build Order

1. Rust 側で Push 3 入出力の最小疎通を作る
2. Pad 押下検知と LED 反映のプロトタイプを作る
3. `LaunchOrFocusApp` を実装する
4. `SendShortcut` と権限チェックを実装する
5. レイアウト保存モデルを固める
6. Tauri GUI の `Grid + Detail Panel` を作る
7. UI 変更を Rust core に即時反映する
8. 常駐、起動時復元、エラーステートを仕上げる

## Deferred To V2

以下は V1 planning の対象外とし、この spec から派生させない。

- 複数プロファイル UI
- Push 実機から編集モードへ入る導線
- Push 画面表示でアイコンとラベルのどちらを優先するか
