# Web Playground Editor 再開発計画

## 背景

Web Playground の editor は `web/src/editor/` と `web/src/language/neplg2/neplg2-provider.ts` に機能が集中しており、描画、入力、状態管理、言語解析、Problems 表示、Hover 表示、補完 UI が強く結合している。

今回の作業では不具合の都度修正を重ねるのではなく、責務分割を見直して editor を一から再設計し、CLI だけで再現できるテスト基盤を先に整える。

## 現状確認

### 現在の editor 実装

- `CanvasEditor` がテキスト状態、カーソル、選択、undo/redo、fold、language provider 連携、Problems 更新まで持っている。
- `EditorInputHandler` が DOM Event を直接読み、ショートカット処理、Hover 遅延、補完操作、編集操作をその場で `CanvasEditor` に反映している。
- `EditorRenderer` が描画時に token/diagnostic/selection/occurrence/fold placeholder の判断まで担っている。
- `EditorDOMUI` が Problems 表示と補完 UI を直接 editor 内部状態から構築している。

### 現在の言語機能実装

- `NEPLg2LanguageProvider` は `window.wasmBindings` に直接依存し、Web 前提の実装になっている。
- highlight, diagnostics, hover, definition, occurrences, completions, indentation, comment toggle, bracket match が 1 ファイルに混在している。
- Hover と Problems の文面は ad-hoc な文字列連結で構築されており、表示仕様が独立していない。
- word boundary, indentation, comment toggle は解析結果ではなく provider 内のローカル規則で処理されている。

### 既存の Rust / WASM 側資産

- `nepl-web/src/lib.rs` には `analyze_lex`, `analyze_parse`, `analyze_name_resolution`, `analyze_semantics`, `analyze_semantics_with_vfs` があり、token / diagnostics / name_resolution / token_resolution / token_semantics まで取得できる。
- `nodesrc/compiler_loader.js` により Trunk の成果物を Node.js から直接ロードできるため、ブラウザを使わず CLI で editor 用解析テストを実行できる。
- 一方で editor 側は Rust から返る payload をそのまま UI 直結で消費しており、再利用しやすい中間表現がない。

## 根本原因

- UI 層と editor state が分離されておらず、不具合が「描画の問題」なのか「入力処理の問題」なのか「解析結果の解釈ミス」なのかを切り分けにくい。
- editor の仕様が DOM event 起点で実装されており、CLI テスト可能な command/state モデルがない。
- 言語解析結果の正規化層がなく、hover/problems/highlight の仕様が provider 内の個別実装に埋もれている。
- 文字オフセット、行列位置、byte offset の変換責務が分散しており、hover/highlight/definition の位置ずれを誘発しやすい。

## 指示適合性の確認

この計画は次の repository 指示に適合する必要がある。

- 不具合は場当たり対応ではなく、責務の分離不足と仕様未固定を根本原因として解消する。
- `plan.md` は変更せず、差分や提案は `note.n.md` に記録する。
- `todo.md` には未実装タスクだけを残す。
- `doc/` の文書は他文書と整合する書き方にそろえる。
- テストは browser 依存に逃がさず、CLI だけで再現可能な形にする。
- 検証手順は `trunk build` の後に `nodesrc/cli.js` ベースで実行し、output の JSON を確認できる形にする。

今回の再設計では特に最後の項目が重要であり、専用 runner を作るだけでは不十分で、`nodesrc/cli.js` から呼べる正式な導線に載せる必要がある。

## 再設計方針

### 目標

- editor の状態遷移を pure な command reducer として定義し、DOM なしで操作テストできるようにする。
- 言語解析結果を UI 非依存の中間モデルへ正規化し、highlight / problems / hover / definition / completion を同じ仕様書から生成する。
- browser adapter は canvas 描画と DOM popup 表示に限定し、仕様判断は core 側へ寄せる。
- CLI から `trunk build` 後の成果物を読み込んで、editor の主要機能を JSON snapshot として検証できるようにする。

### 進め方の制約

repository の運用指示を守るため、実装の進め方にも制約を置く。

- 既存 editor を一度に捨てる flag day 方式にはしない。
- まず core を横に追加し、browser adapter を段階的に差し替える。
- 各段階で既存 UI を壊さない最小差分を優先し、不必要な周辺変更を避ける。
- 1 回の変更単位は「状態管理」「解析正規化」「入力」「描画」「CLI テスト導線」など責務ごとに分ける。
- 各変更単位ごとにテストを通し、必要ならその時点で commit できる粒度に保つ。

この制約により、再設計でありつつも、実際の実装は小さな段階に分割して進める。

### 層分割

#### 1. Editor Core

`web/src/editor-core/` を新設し、少なくとも次の責務へ分割する。

- `text-buffer.ts`
  - テキスト本体、行インデックス、offset 変換、差分適用を管理する。
- `editor-state.ts`
  - カーソル、選択、scroll、overwrite、fold、completion 状態、hover 対象、undo/redo を保持する。
- `editor-command.ts`
  - insert / delete / move / select / indent / outdent / comment / fold / accept-completion などの command を定義する。
- `editor-reducer.ts`
  - command を受けて state を更新する pure reducer を実装する。
- `keymap.ts`
  - DOM keyboard event を editor command 列へ変換する規則を分離する。
- `view-model.ts`
  - 描画や DOM UI が必要とする cursor line, selection range, visible lines, fold placeholder, problems panel items を生成する。

#### 2. Language Analysis Core

`web/src/language/neplg2/core/` を新設し、次の責務へ分割する。

- `analysis-source.ts`
  - WASM / Node loader から `analyze_semantics(_with_vfs)` を呼び出す。
- `analysis-normalizer.ts`
  - token, diagnostics, token_resolution, token_semantics, definitions, references を editor 共通形式へ正規化する。
- `highlight-model.ts`
  - semantic highlight と lexical fallback の生成規則をまとめる。
- `problem-model.ts`
  - diagnostics の並び順、severity、範囲、problem list item 生成を統一する。
- `hover-model.ts`
  - hover の表示要素を structured data として生成し、最終的な文字列整形を別関数に分離する。
- `navigation-model.ts`
  - definition, occurrences, symbol lookup, completion candidates を担当する。
- `editing-rules.ts`
  - indentation, word boundary, toggle comment, bracket match を解析結果と文法規則に基づいて実装する。

#### 3. Browser Adapter

既存の `web/src/editor/` は adapter 層として縮小する。

- `editor.ts`
  - stateful facade のみを持つ。
- `editor-input-handler.ts`
  - DOM event を `keymap.ts` と pointer command へ変換するだけにする。
- `editor-renderer.ts`
  - `view-model.ts` が返した描画データを canvas へ描くことだけに集中する。
- `editor-dom-ui.ts`
  - popup / completion / problems の DOM 表示を担当するが、表示内容の決定は core から受け取る。

## Hover / Problems / Highlight の再設計

### Highlight

- lexical token 色付けと semantic highlight を分離する。
- semantic 情報がある token は semantic 優先、ない token は lexical fallback とする。
- byte offset と UTF-16 index の変換を normalizer で一元管理する。
- renderer は完成済み token decoration 配列を描くだけにする。

### Problems

- diagnostics を editor 表示用の `ProblemItem` へ正規化し、ソート規則を固定する。
- `ProblemItem` は message, severity, range, source stage, related definition を持てるようにする。
- Problems panel と underline 表示は同一データから生成する。

### Hover

- hover はプレーン文字列ではなく `HoverModel` として構築する。
- `HoverModel` の候補:
  - token 表記
  - 推論型
  - 定義種別と定義名
  - 引数位置
  - 定義候補一覧
  - diagnostic summary
- browser 表示用 formatter と CLI snapshot 用 serializer を分ける。

## CLI 完結テスト方針

### 基本方針

- browser の DOM event を直接叩くテストではなく、editor core の command/state を Node.js から実行する。
- 解析系テストは `trunk build` 後の `nepl-web` WASM 成果物を `nodesrc/compiler_loader.js` で読み込んで実行する。
- 出力は JSON に統一し、`nodesrc/cli.js` からの既存テスト文化と整合する形にする。

### 追加する CLI テスト基盤

- `nodesrc/cli.js`
  - playground editor テストを呼び出す entry point を追加し、JSON を `-o` で出力できるようにする。
  - 日常運用ではこの入口を標準にし、既存の `nodesrc` ワークフローとそろえる。
- `nodesrc/playground_editor_test_runner.js`
  - `nodesrc/cli.js` から呼ばれる下位 runner として、Trunk 成果物をロードし、fixture と command script を読み、state snapshot を JSON 出力する。
- `tests/playground_editor/`
  - editor の fixture, command script, expected snapshot を置く。
- `web/src/editor-core/testing/`
  - reducer 実行や snapshot 生成を補助する test-only utility を置く。

### fixture 形式

CLI だけで運用しやすくするため、fixture 形式も固定する。

- `tests/playground_editor/<case>/source.nepl`
  - 主対象の入力ソース
- `tests/playground_editor/<case>/vfs.json`
  - 複数ファイルや import が必要な場合の仮想ファイル群
- `tests/playground_editor/<case>/commands.json`
  - key input, shortcut, pointer, hover, definition 要求などの操作列
- `tests/playground_editor/<case>/expected.json`
  - 比較対象の snapshot

`commands.json` は DOM event そのものではなく、editor core command と検証要求だけを表す。

### テスト対象

#### 1. Key Input / Shortcut

- 文字入力、改行、Backspace、Delete
- Arrow / Home / End / PageUp / PageDown
- Shift 付き選択拡張
- Ctrl/Cmd+A, Z, Y, /
- Tab / Shift+Tab
- F12 definition jump
- Insert による overwrite mode 切替
- completion 表示中の ArrowUp / ArrowDown / Enter / Tab / Escape

pointer 系も CLI で再現できる必要があるため、次も対象に含める。

- click による cursor 移動
- drag による選択
- gutter click による fold toggle
- hover request に対する hover model 生成

#### 2. Editor State

- cursor / selection / preferred column
- undo / redo stack
- folded line と visible line の対応
- completion selection state
- hover target state
- overwrite mode と通常入力の差

#### 3. Highlight / Problems / Hover

- keyword / string / number / comment / function / variable の分類
- definition 解決済み token の function highlight
- diagnostics の range と severity
- hover に含まれる型情報、定義情報、候補情報
- definition jump の target range
- occurrences の収集
- folding range の抽出

#### 4. Multi-file / VFS

- `#import` を含む複数ファイル fixture で `analyze_semantics_with_vfs` を使う。
- definition / hover / problems が file_path を跨いで正しく生成されるか確認する。

### 期待する JSON snapshot

各ケースは少なくとも次を出力できるようにする。

- `text`
- `cursor`
- `selection`
- `foldedLines`
- `completion`
- `diagnostics`
- `problems`
- `hover`
- `definition`
- `occurrences`
- `decorations`
- `visibleLines`

### 検証コマンドの標準化

playground editor 再開発では、検証手順そのものも固定する。

1. `trunk build`
2. `node nodesrc/cli.js ... -o json=<path>`
3. 生成された output JSON を確認し、snapshot 差分と失敗ケースを読む

想定する最終形の例:

```bash
trunk build
node nodesrc/cli.js -i tests/playground_editor -o json=tmp/playground-editor-tests.json
```

また、開発中の局所再現用には `nodesrc/playground_editor_test_runner.js` を直接使えてよいが、CI と完了確認は `nodesrc/cli.js` 経由へ統一する。

## 実装フェーズ

### Phase 0: 仕様固定

- editor core の状態モデルと command 一覧を決める。
- hover / problems / highlight の JSON schema を決める。
- CLI test runner の入出力形式を決める。
- `nodesrc/cli.js` へどう統合するかをこの段階で決める。
- fixture ディレクトリ構成と `commands.json` / `expected.json` の schema を決める。

### Phase 1: Pure Core 先行

- text buffer, state, reducer, keymap を実装する。
- DOM 非依存の unit test を Node.js から回せるようにする。
- この段階では既存 browser adapter へはまだ最小限の接続しかしない。

### Phase 2: Language Model 再構築

- `NEPLg2LanguageProvider` から解析正規化ロジックを分離する。
- hover / problems / highlight / completion / navigation を純粋関数化する。

### Phase 3: Browser Adapter 差し替え

- 既存 canvas editor を新 core に接続する。
- DOM / canvas は view-model 消費に限定する。
- 旧実装を即削除せず、差し替え完了まで比較可能な状態を維持する。

### Phase 4: CLI Snapshot 拡充

- fixture を増やし、複数ファイル・日本語・全角・byte offset 境界を含むケースを追加する。
- `trunk build` + CLI テスト + JSON 検証を標準手順にする。

### Phase 5: 完了判定と運用化

- `nodesrc/cli.js` 経由の JSON 出力が CI とローカルの両方で再現できる状態にする。
- `doc/testing.md` と `doc/web_playground.md` に playground editor の正式な検証手順を追記する。
- 実装修正ごとに、必要なら README / doc を更新する運用へそろえる。

### 各フェーズの検証と commit

各フェーズの完了時には、少なくとも次を満たす。

1. `trunk build` が通る
2. `node nodesrc/cli.js ... -o json=<path>` で対象ケースの JSON が更新または一致する
3. 必要な document 更新が済んでいる
4. その時点で独立に commit 可能な粒度になっている

## 非目標

- 初回の再開発では Monaco など他 editor への置換は行わない。
- terminal / explorer / tabs の再設計は対象外とし、editor との境界整理に留める。
- visual design の刷新は副次対応とし、まず仕様とテストの再建を優先する。

## 完了条件

- editor 操作の主要経路が CLI から command script で再現できる。
- highlight / problems / hover / definition / completion の仕様が JSON snapshot として固定される。
- browser adapter が core なしでは振る舞いを決めない構成になる。
- 既知の editor 不具合修正が「個別対処」ではなく、新 core のテスト追加で再発防止できる状態になる。
- 実装完了後の標準確認手順が `trunk build` と `nodesrc/cli.js` による JSON 検証として文書化されている。
