# Open URL Action Design

Date: 2026-03-25

## Summary

Push Deck に新しい汎用 action `open_url` を追加する。  
この action は macOS の規定ブラウザで URL を開く挙動を基本としつつ、Arc が利用可能な場合だけ追加オプションとして `profile` と `pinned` タブ制御を扱えるようにする。

今回の目的は、たとえば「ボタンを押したら Arc の `Private` profile で pinned タブの Twitter を開く」のような操作を、既存の `launch_or_focus_app` や `send_shortcut` とは独立した action として設定できるようにすること。

この文書は [2026-03-18-push-deck-design.md](/Users/yasudanaoki/Desktop/push-deck/docs/superpowers/specs/2026-03-18-push-deck-design.md) の addendum として扱う。既存 spec の責務分離は維持し、action model を拡張する。

## Product Goals

- Push の pad から URL を直接開けるようにする
- 通常ブラウザ操作は macOS の規定ブラウザに委ねる
- Arc 利用時だけ、profile 指定と pinned タブ再利用を追加できるようにする
- 既存 action の責務を壊さずに、将来のブラウザ拡張余地を残す

## Non-Goals

- V1.1 時点で Chrome / Edge / Brave の profile 制御を実装すること
- すべてのブラウザで pinned タブ再利用を統一的に扱うこと
- 複数 URL を順に開くマクロ action
- URL ごとの閲覧状態同期や既存タブタイトル条件による高度検索

## Chosen Approach

既存 `launch_or_focus_app` を拡張せず、新しい action `open_url` を追加する。

### Why this approach

- `launch_or_focus_app` は「アプリ起動/前面化」の責務に留めたほうが明確
- `open_url` は「URL を開く」という別のユースケースとして独立したほうが UI も schema も自然
- 通常ブラウザ動作と Arc 固有動作を一つの action 内で分岐できる
- 将来 `browser` ごとの差分実装を足しやすい

## Action Model

`PadBinding.action` に以下の variant を追加する。

```ts
type PadAction =
  | { type: "unassigned" }
  | {
      type: "launch_or_focus_app";
      bundleId: string;
      appName: string;
    }
  | {
      type: "send_shortcut";
      key: string;
      modifiers: string[];
    }
  | {
      type: "open_url";
      url: string;
      browser: "default" | "arc";
      tabBehavior: "normal" | "pinned";
      arcProfileId?: string | null;
    };
```

### Semantics

- `browser: "default"`
  - macOS の規定ブラウザで URL を開く
  - `tabBehavior` は保存上は持てるが、実行時は `normal` と同義に扱う
  - `arcProfileId` は無視する
- `browser: "arc"` かつ `tabBehavior: "normal"`
  - Arc で URL を開く
- `browser: "arc"` かつ `tabBehavior: "pinned"`
  - 指定 profile の pinned タブ一覧から URL 一致を探索する
  - 一致する pinned タブがあればそのタブへ寄せる
  - 一致しなければその profile に新しい pinned タブとして作成する

### Validation Rules

- `url` は空文字を禁止する
- `url` は `http` または `https` の絶対 URL のみ許可する
- `browser: "default"` のとき `arcProfileId` は `null` 扱いへ正規化する
- `browser: "default"` のとき `tabBehavior: "pinned"` は保存時に `normal` へ正規化する
- `browser: "arc"` かつ `tabBehavior: "pinned"` のときだけ `arcProfileId` を保持できる
- `browser: "arc"` かつ `tabBehavior: "pinned"` で `arcProfileId` が欠けている場合は保存エラーにする

## UI Model

Detail Panel に `open_url` 用の editor を追加する。

### Fields

- Action type
  - `Open URL`
- URL
  - テキスト入力
- Browser
  - `Default Browser`
  - `Arc`
- Tab behavior
  - `Open normally`
  - `Reuse or create pinned tab`
- Arc profile
  - 候補選択

### Interaction Rules

- `browser = default` のとき `tabBehavior` は表示してもよいが `Open normally` 固定にする
- `browser = default` のとき `Arc profile` は非表示または disabled にする
- `browser = arc` のときだけ `tabBehavior` を編集可能にする
- `browser = arc` かつ `tabBehavior = pinned` のときだけ `Arc profile` 選択を有効にする
- Arc capability が unavailable のときは
  - `browser = arc` を選べないようにする
  - 既存設定に Arc action がある場合は read-only warning を出してもよい
  - 実行時は通常 URL オープンにフォールバックする

### Label Defaults

- `label` が空なら URL の hostname を既定ラベルとして使う
- Arc + pinned の場合もラベル生成規則は同じにする

## Runtime Boundary

既存アーキテクチャの責務は次で拡張する。

### Action Engine

- `src-tauri/src/actions/open_url.rs` を新設する
- `open_url` action の正規化済み payload を受け取る
- browser ごとの実行戦略を macOS integration 層へ委譲する
- Arc 固有失敗時のフォールバック方針をここで統一する

### macOS Integration

- `src-tauri/src/macos/mod.rs` 直下または近傍に browser integration を追加する
- 共通責務
  - 規定ブラウザで URL を開く
  - Arc アプリの存在確認
  - Arc capability 検出
- Arc 固有責務
  - profile 候補の列挙
  - 指定 profile の pinned タブ探索
  - 一致タブへのフォーカス
  - pinned タブの新規作成

### Frontend API

- Arc profile 候補取得 command を追加する
- capability payload に Arc browser integration 状態を追加する
- editor は capability を見て Arc UI の enabled/disabled を切り替える

## Arc Integration Strategy

Arc 連携は `設定読取 + スクリプト実行` の組み合わせで扱う。

### Profile enumeration

- 第一手段として macOS 上の Arc 設定ファイルから profile 一覧を読む
- UI が必要とするのは profile の stable identifier と表示名
- 実装上は以下のどちらでもよい
  - 設定ファイルから stable key を取得できるならその key を `arcProfileId` に使う
  - stable key が取れない場合は profile 名を `arcProfileId` として扱う

### Pinned tab control

- Arc の pinned タブ探索とフォーカス/作成は AppleScript などのスクリプト経由で行う
- 比較キーは URL の完全一致を基本とする
- URL 正規化は最低限に留める
  - 末尾 slash の吸収や query 除去は行わない
  - 保存した URL と実タブ URL が完全一致した場合のみ既存 pinned とみなす

### Capability fallback

以下のどれかが満たせない場合、Arc 固有機能は unavailable とする。

- Arc 本体が見つからない
- Arc profile の列挙に失敗する
- Arc pinned タブ操作スクリプトが失敗する

unavailable の場合の挙動は次とする。

- UI では Arc 固有項目を disabled にする
- 既存 `open_url(browser=arc, tabBehavior=pinned)` は保存済みなら維持してよい
- 実行時は失敗で止めず、規定ブラウザでの通常 URL オープンにフォールバックする

## Data Model Additions

Frontend / Rust 共通で以下の補助型を持つ。

```ts
type BrowserTarget = "default" | "arc";
type UrlTabBehavior = "normal" | "pinned";

type ArcProfileOption = {
  id: string;
  label: string;
};
```

runtime capability には Arc 用の状態を追加する。

```ts
type ArcCapabilityState = "available" | "unavailable";
```

必要なら unavailable の理由表示用に detail を拡張できるが、初回実装では必須ではない。

## Persistence Rules

- `open_url` action は既存 config JSON に保存可能とする
- 既存 action と同様に schema validation 対象に含める
- `arcProfileId` は `browser=arc && tabBehavior=pinned` のときだけ保存する
- 既存 config に `open_url` が存在しない場合の migration は不要
- 既存 schema version は据え置きでもよいが、action variant の追加で migration 管理が必要なら version increment を許容する

## Error Handling

- URL validation failure
  - 保存失敗として UI に返す
- Arc profile 読取失敗
  - capability unavailable
  - UI は Arc 固有設定を disabled
- Arc pinned タブ探索失敗
  - 実行時 warning を残し、規定ブラウザ open へフォールバック
- Arc profile が保存時点では存在したが実行時に消えていた
  - Arc fallback と同じく規定ブラウザ open に降格する

## Testing Strategy

### Schema tests

- `open_url` の URL validation
- `browser=default` での `tabBehavior` 正規化
- `browser=arc && tabBehavior=pinned` で `arcProfileId` 必須

### Action tests

- 規定ブラウザ open が呼ばれる
- Arc normal open が呼ばれる
- Arc pinned で既存 tab に寄せる
- Arc pinned で新規作成へ進む
- Arc failure で規定ブラウザ open にフォールバックする

### Frontend tests

- editor で browser 切替に応じて field enabled state が変わる
- Arc capability unavailable 時に Arc UI が無効化される
- draft から `open_url` binding を正しく生成できる

## Open Implementation Risks

- Arc の profile 設定ファイル形式が安定していない可能性がある
- AppleScript から pinned タブへ十分にアクセスできるかは実機検証が必要
- URL 完全一致判定は単純だが、ユーザー期待とのズレが出る可能性がある

初回実装では判定ロジックを複雑化しない。実機挙動が確認できてから URL 正規化戦略を追加検討する。
