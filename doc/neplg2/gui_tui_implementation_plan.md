# NEPLg2 GUI/TUI 標準ライブラリ実装計画

作成日: 2026-06-01

## 目的

`doc/neplg2/gui_standard_library_spec.md` に基づき、GUI と TUI を共通 UI substrate として実装する。既存 TUI は保守対象として残すのではなく、terminal backend として段階的に再設計・再実装する。

この計画は `plan.md` を変更せず、NEPLg2.1 の現行 `stdlib/` 上で進める。

## 現状

- 明示的な GUI 標準ライブラリは存在し、`core/gui`、`alloc/gui`、`std/gui`、`platforms/gui/terminal` の初期 checkpoint まで進んでいる。
- 現在の実装は bounded data contract を優先した段階であり、recursive tree、real backend present、Web Playground bridge、native/mobile backend はまだ未実装である。
- 既存の近い資産は `features/tui` と `platforms/wasix/tui` である。
- `platforms/wasix/tui` は TTY ABI、ANSI 出力、text width、box line、line buffer、diff present を持つが、raw terminal helper と UI concept が同じ層に混ざっている。
- Web Playground は browser 上の editor / terminal を持つが、NEPL stdlib の GUI backend としては未接続である。

## 根本課題

既存 TUI は terminal 向け helper としては有用だが、GUI と共通化できる抽象境界が不足している。

```text
現状:
    features/tui
        -> platforms/wasix/tui
            -> ansi/text/box/buffer/tty

問題:
    - ActionId / GuiEvent / GuiEffect がない
    - terminal raw input と application event が分離されていない
    - text-cell surface と GUI render command が共通 model を持たない
    - line buffer diff present が backend detail ではなく public helper になっている
    - accessibility / lifecycle / capability の型境界がない
```

解決後の形は次である。

```text
Application:
    Model + GuiEvent -> Update Model
    Model -> ViewTree

Common substrate:
    ViewTree -> LayoutTree -> DrawCommand stream

Backends:
    Web canvas / DOM-backed surface
    Native window surface
    Mobile host view
    Embedded DrawTarget
    Terminal TextGrid surface
```

## Phase 1: core/gui no_alloc 基盤

追加する module:

```text
stdlib/core/gui.nepl
stdlib/core/gui/prelude.nepl
stdlib/core/gui/geometry.nepl
stdlib/core/gui/color.nepl
stdlib/core/gui/pixel.nepl
stdlib/core/gui/text_measure.nepl
stdlib/core/gui/draw_target.nepl
stdlib/core/gui/render_target.nepl
stdlib/core/gui/dirty_region.nepl
stdlib/core/gui/dirty_region_set.nepl
stdlib/core/gui/capability.nepl
stdlib/core/gui/error.nepl
stdlib/core/gui/event.nepl
stdlib/core/gui/render_command.nepl
```

最初の checkpoint では trait-based drawing の実 backend へ深く踏み込まず、型・constructor・軽量 helper・mock target・doctest を先に固定する。ただし、embedded を最低制約にするため、`core/gui` が `alloc` / `std` / `platforms` へ依存しないことは Phase 1 で検査する。検査は focused doctest だけにせず、`nodesrc/test_stdlib_gui_layering_policy.js` で core/platform 依存方向と terminal TextGrid 型の再定義禁止も固定する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_core.n.md --no-tree -o tmp/gui-core-phase1.json -j 1 --dist web/dist --assert-io
node nodesrc/run_source_policy_regressions.js --warn-only
node nodesrc/issues.js check --dir issues
git diff --check
```

## Phase 2: TUI 共通化の型橋渡し

追加または変更する module:

```text
stdlib/features/gui.nepl
stdlib/features/tui.nepl
stdlib/platforms/gui/terminal.nepl
stdlib/platforms/gui/terminal/capability.nepl
stdlib/platforms/gui/terminal/text_grid.nepl
stdlib/platforms/gui/terminal/frame.nepl
```

目的:

- `features/gui` を新しい UI substrate の public facade にする。
- `features/tui` は compatibility facade として残し、内部で terminal backend へ寄せる。
- terminal は `SurfaceKind::TextGrid` capability を持つ backend として定義する。
- terminal 固有の cols / rows は `TerminalProfile` に置き、共通 capability と text-cell command は `core/gui` の型を再利用する。custom capability を受ける helper は `Result` を返し、`SurfaceKind::TextGrid` 以外を拒否する。
- 既存 `line_top` / `line_box` / `buffer_present_diff` などを一気に消さず、terminal backend の compatibility helper として段階的に移す。
- この Phase は terminal backend の型境界を先に作るための橋渡しであり、最初の complete backend は Web Playground のままとする。

2026-06-01 checkpoint では、terminal backend は `TerminalProfile`、`TextGridCapability`、`TerminalFrame` と core `TextCellRun` の橋渡しまでを実装済みである。`TextGridRenderTarget` や real `GuiHost.present` はまだ実装していない。旧 `features/tui` / `platforms/wasix/tui` の raw line buffer diff は compatibility path に残し、共通 substrate の public diff contract にはしない。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/features_tui.n.md -i tests/stdlib/gui_core.n.md --no-tree -o tmp/gui-tui-bridge.json -j 1 --dist web/dist --assert-io
node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js
node nodesrc/issues.js check --dir issues
git diff --check
```

## Phase 3: alloc/gui application model

追加する module:

```text
stdlib/alloc/gui.nepl
stdlib/alloc/gui/app.nepl
stdlib/alloc/gui/app/types.nepl
stdlib/alloc/gui/widget.nepl
stdlib/alloc/gui/widget/types.nepl
stdlib/alloc/gui/tree.nepl
stdlib/alloc/gui/tree/types.nepl
stdlib/alloc/gui/layout.nepl
stdlib/alloc/gui/layout/types.nepl
stdlib/alloc/gui/theme.nepl
stdlib/alloc/gui/theme/types.nepl
stdlib/alloc/gui/accessibility.nepl
stdlib/alloc/gui/focus.nepl
stdlib/alloc/gui/focus/types.nepl
stdlib/alloc/gui/routing.nepl
stdlib/alloc/gui/routing/types.nepl
stdlib/alloc/gui/routing/focus.nepl
stdlib/alloc/gui/diff.nepl
stdlib/alloc/gui/diff/types.nepl
stdlib/alloc/gui/text.nepl
stdlib/alloc/gui/text/types.nepl
stdlib/alloc/gui/test.nepl
```

最初は `WidgetId`、`ActionId`、`ViewNode`、`GuiEffect`、`Update`、`LayoutContext`、widget descriptor、retained tree、focus traversal、event routing、diff / invalidation、text buffer、theme、semantic tree、mock replay helper を小さく入れる。Closure callback、DOM、terminal raw code、OS handle は入れない。

2026-06-01 checkpoint では、`Update.effects` の境界を `GuiEffectBatch` に変更し、`alloc/gui/layout` の `TextMeasurer` 注入、`alloc/gui/widget` の button / label descriptor、`alloc/gui/tree` の bounded retained tree、`alloc/gui/theme` の typed palette / metrics、`alloc/gui/accessibility` の semantic tree 初期 slice まで実装した。`GuiEffectBatch` は現時点では capacity 2 の bounded data であり、`alloc` collection 側の owner contract が安定した段階で `Vec GuiEffect` へ置き換える。

Focus traversal は `alloc/gui/focus` の platform 非依存 data contract として扱う。順方向 / 逆方向、wrap の有無、現在 focus が tree に存在しない場合の結果、disabled widget の除外を `Option` / enum で表し、host focus や accessibility focus の反映は `std/gui` 以降へ渡す。

Focus routing は traversal とは別に `alloc/gui/routing/focus` へ置く。`FocusRouteCommand::Next` / `Previous` は focus movement だけを返し、`Activate` は current focus id の widget action だけを `GuiEvent::Action` として返す。戻り値は `FocusRouteResult::Ignored` / `MoveFocus WidgetId` / `Emit GuiEvent` で分ける。Tab、Shift+Tab、Enter、Space の portable default mapping は `std/gui/keymap` が `KeyboardEvent` と `FocusKeyMap` から `Option FocusRouteCommand` へ変換する。Arrow key や modifier bit の std contract は `std/gui/keymap` に置くが、ANSI escape sequence、DOM keyboard event、OS virtual key は platform backend が `KeyboardEvent` へ正規化し、application は raw key sequence を直接扱わない。

Event routing は `alloc/gui/routing` の pure data contract として扱う。pointer routing は `LayoutTree` hit test で `WidgetId` を得て、`ViewTree` の widget data から `GuiEvent::Action` を導出する。現 checkpoint は bounded root + 2 child、half-open `GuiRect`、second child topmost、disabled widget suppression、focus command routing、std keymap、terminal 1 byte input normalization、`ESC [ Z` Shift+Tab normalization、`ESC [ A/B/C/D` arrow key normalization、`ESC [ 1 ; <modifier> A/B/C/D` xterm modifier arrow normalization までを固定する。pointer capture、gesture、Web / native / mobile raw keyboard normalization、terminal の追加 ANSI / CSI sequence、途中入力 buffering、recursive traversal は後続で実装する。TUI では keyboard / focus routing から同じ `FocusRouteCommand` / `GuiEvent::Action` を生成し、raw ANSI input を application が直接扱わないようにする。

Diff / invalidation は `alloc/gui/diff` に置く。ここでは dirty widget / tree / layout などの共通 data contract だけを持ち、terminal line buffer diff、DOM patch、framebuffer dirty rect compression は `platforms/gui/*` の実装詳細にする。

Text buffer と minimal layout cache data は `alloc/gui/text` が所有する。`core/gui::TextRunId` は buffer snapshot 内の安定参照 id とし、`std/gui` / platform は測定、IME、font loading を担当する。現 checkpoint では `TextLayout` が injected `TextMeasurer` の結果、byte length、char count、fallback cell count、max width を保持し、`CachedTextLayout` が buffer id、run id、font id、max width、byte length、char count から deterministic key を作る。`core/gui` に `String` 実体を持たせず、terminal backend に text buffer ownership を漏らさない。line break、text hash / revision based invalidation、complex shaping cache は後続で実装する。

`core/gui/dirty_region` は embedded / framebuffer 向け no_alloc redraw contract として扱う。現 checkpoint は `DirtyRegion::Empty` / `Rect` / `Full` と O(1) bounding rect merge までを固定する。

`core/gui/dirty_region_set` は no_alloc fixed-capacity multiple rect contract として扱う。現 checkpoint は最大 2 rect を保持し、3 個目の追加は `Full` 状態へ昇格する。generic capacity、backend-specific damage compression、DOM patch、terminal line diff は `platforms/gui/*` の実装詳細へ置く。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_app.n.md --no-tree -o tmp/gui-app-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_layout.n.md --no-tree -o tmp/gui-layout-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_widget.n.md --no-tree -o tmp/gui-widget-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_tree.n.md --no-tree -o tmp/gui-tree-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_focus.n.md --no-tree -o tmp/gui-focus-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_routing.n.md --no-tree -o tmp/gui-routing-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_focus_routing.n.md --no-tree -o tmp/gui-focus-routing-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_diff.n.md --no-tree -o tmp/gui-diff-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_text.n.md --no-tree -o tmp/gui-text-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_theme.n.md --no-tree -o tmp/gui-theme-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_accessibility.n.md --no-tree -o tmp/gui-accessibility-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/issues.js check --dir issues
git diff --check
```

## Phase 4: std/gui host contract

追加する module:

```text
stdlib/std/gui.nepl
stdlib/std/gui/host.nepl
stdlib/std/gui/runtime.nepl
stdlib/std/gui/window.nepl
stdlib/std/gui/timer.nepl
stdlib/std/gui/text_measure.nepl
stdlib/std/gui/keymap.nepl
stdlib/std/gui/ime.nepl
stdlib/std/gui/accessibility_host.nepl
stdlib/std/gui/error_display.nepl
```

目的:

- `GuiHost`、`WindowId`、`SurfaceId`、`TimerId`、`HostTextMeasurer`、`ImeBridge`、`AccessibilityHost` の標準境界を定義する。
- application は `GuiHost` を直接呼ばず、`GuiEffect` を runtime が解釈する。
- capability unsupported を `GuiError::Unsupported` として返す。
- `TextMeasurer` contract は `core/gui` に置き、ここでは host font / browser / terminal / native text stack へ接続する実装を扱う。
- `FocusKeyMap` は `KeyboardEvent` を `FocusRouteCommand` へ O(1) で変換し、`alloc/gui/routing/focus` に platform raw key code を漏らさない。

## Phase 5: Web Playground backend

対象:

```text
stdlib/platforms/gui/web/**
web/src/gui/**
nodesrc/gui_*_test_runner.js
tests/gui_playground/**
```

目的:

- TypeScript host bridge が `GuiEvent`、`DrawCommand`、`TextMeasurer` を変換する。
- JS `null` / `undefined` は backend 境界で `Option` / `Result` に変換する。
- browser canvas / DOM-backed surface を `RenderTarget` として実装する。
- CLI で headless smoke test を走らせる。

## Phase 6: Terminal backend replacement

対象:

```text
stdlib/platforms/gui/terminal/**
stdlib/platforms/wasix/tui/**
tests/stdlib/features_tui.n.md
tests/stdlib/gui_terminal_input.n.md
nodesrc/tui_regression.js
```

目的:

- `platforms/wasix/tui` の raw storage / ANSI / TTY helper を terminal backend implementation detail に押し下げる。
- `buffer_new` / `buffer_present_diff` の raw handle API を `TextGridRenderTarget` / `TerminalFrame` / `GuiHost.present` へ置き換える。
- 現 checkpoint の `TerminalFrame` は単一 `TextCellRun` frame 境界である。`TextGridRenderTarget` は diff / present 実装時に追加し、terminal-specific line diff は `platforms/gui/terminal` に閉じる。
- `platforms/gui/terminal/input.nepl` は terminal raw byte、3 byte ESC sequence、bounded 6 byte CSI modifier sequence を `TerminalInputEvents` へ正規化する。`ESC [ Z` は Shift+Tab として key code 9、modifier bit 1 へ正規化し、`ESC [ A/B/C/D` と `ESC [ 1 ; <modifier> A/B/C/D` は std navigation key code と modifier bitset へ正規化するが、`FocusRouteCommand` や `ActionId` は作らず、`std/gui/keymap` と `alloc/gui/routing/focus` の責務を保つ。
- `features/tui` の利用者向け path は壊さず、内部を新 substrate に差し替える。

互換維持:

- `features_tui.n.md` は当面維持する。
- 新規 test は `tests/stdlib/gui_terminal.n.md` に追加し、旧 TUI helper と新 TextGrid backend の対応を固定する。
- input normalization は `tests/stdlib/gui_terminal_input.n.md` に分け、Tab / LF / CR / Space / printable ASCII / invalid byte / unsupported control byte / Shift+Tab ESC sequence / arrow key ESC sequence / xterm modifier arrow CSI sequence / unknown sequence / invalid modifier parameter を固定する。

## Phase 7: Embedded backend

目的:

- Phase 1 で固定した `core/gui` no_alloc contract を real-style backend で再検査する。
- `MockDrawTarget`、`DirtyRegion`、`DirtyRegionSet` を用意する。現 checkpoint の `DirtyRegionSet` は fixed capacity 2 の O(1) contract であり、generic capacity や backend-specific compression は後続で追加する。
- Optional `FlushTarget` を確認する。

## Phase 8: Native / Mobile backend

Native:

- window / surface / clipboard / timer / cursor を `std/gui` host contract に接続する。
- File dialog、menu、tray、drag-and-drop は extension module に分ける。

Mobile:

- `LifecycleEvent`、touch id、IME composition、safe area、orientation、accessibility bridge を backend として接続する。
- Keyboard event だけで text input を扱わない。

## Documentation Rules

- 仕様と現状実装を分けて書く。
- platform 差は `GuiCapabilities` と target notes で説明する。
- doc comment は日本語で書き、module 先頭に目的、実装、注意、計算量を置く。
- callback を避ける理由、TUI を backend として扱う理由、unsupported operation を `Result` にする理由は繰り返し明示する。

## Test Policy

最小 test 群:

```text
tests/stdlib/gui_core.n.md
    geometry arithmetic
    color constructors
    event/capability enum smoke
    text-grid surface kind

tests/stdlib/gui_app.n.md
    ActionId based update replay
    GuiEffect batch construction
    ViewNode snapshot

tests/stdlib/gui_layout.n.md
    TextMeasurer injection
    constraint clamp
    invalid constraint error

tests/stdlib/gui_widget.n.md
    Button action event generation
    disabled action suppression
    semantic node generation

tests/stdlib/gui_tree.n.md
    bounded ViewTree child insertion
    first focusable WidgetId
    LayoutTree child insertion

tests/stdlib/gui_focus.n.md
    next / previous focus traversal
    disabled widget exclusion
    missing current focus result

tests/stdlib/gui_routing.n.md
    LayoutTree hit test
    button action lowering
    disabled / outside suppression
    child z-order

tests/stdlib/gui_focus_routing.n.md
    focus route command movement
    focused widget action emission
    disabled / non-action / stale focus ignored

tests/stdlib/gui_keymap.n.md
    default Tab / Shift+Tab / Enter / Space mapping
    KeyUp and unknown key ignored
    custom FocusKeyMap contract

tests/stdlib/gui_diff.n.md
    widget data diff
    tree shape invalidation
    child widget invalidation id

tests/stdlib/gui_text.n.md
    TextBuffer storage
    checked insert / replace / delete
    TextRunId mapping boundary

tests/stdlib/gui_theme.n.md
    typed color role lookup
    metric validation
    Option FontId

tests/stdlib/gui_accessibility.n.md
    Semantic node state
    bounded semantic tree insertion

tests/stdlib/gui_terminal.n.md
    TextGrid capability
    terminal frame model
    legacy TUI facade bridge

tests/stdlib/gui_terminal_input.n.md
    terminal byte to KeyboardEvent / TextInputEvent normalization
    Space as both key and text
    ESC [ Z as Shift+Tab keyboard normalization
    invalid byte and unsupported control byte handling

tests/stdlib/gui_dirty_region.n.md
    Empty / Rect / Full merge
    bounding rect union
    invalid geometry error

tests/stdlib/gui_dirty_region_set.n.md
    fixed two rect storage
    overflow to Full
    invalid geometry error
```

Web backend が入った後:

```text
node nodesrc/cli.js -i tests/gui_playground --gui-playground-tests -o json=tmp/gui-playground.json
```

## Checkpoint Commit Rule

各 phase は小さく commit する。

1. docs only
2. `core/gui` types + tests
3. `features/gui` + terminal capability bridge
4. `alloc/gui` app model + tests
5. `alloc/gui` layout/widget/accessibility slice + tests
6. TUI backend replacement checkpoint

各 commit 前に、その checkpoint に対応する focused test、`node nodesrc/issues.js check --dir issues`、`git diff --check` を通す。
