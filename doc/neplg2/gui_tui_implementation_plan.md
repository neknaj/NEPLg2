# NEPLg2 GUI/TUI 標準ライブラリ実装計画

作成日: 2026-06-01

## 目的

`doc/neplg2/gui_standard_library_spec.md` に基づき、GUI と TUI を共通 UI substrate として実装する。既存 TUI は保守対象として残すのではなく、terminal backend として段階的に再設計・再実装する。

この計画は `plan.md` を変更せず、NEPLg2.1 の現行 `stdlib/` 上で進める。

## 現状

- 明示的な GUI 標準ライブラリはまだ存在しない。
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
stdlib/platforms/gui/terminal/text_cell.nepl
stdlib/platforms/gui/terminal/render_target.nepl
```

目的:

- `features/gui` を新しい UI substrate の public facade にする。
- `features/tui` は compatibility facade として残し、内部で terminal backend へ寄せる。
- terminal は `SurfaceKind::TextGrid` capability を持つ backend として定義する。
- terminal 固有の cols / rows は `TerminalProfile` に置き、共通 capability と text-cell command は `core/gui` の型を再利用する。custom capability を受ける helper は `Result` を返し、`SurfaceKind::TextGrid` 以外を拒否する。
- 既存 `line_top` / `line_box` / `buffer_present_diff` などを一気に消さず、terminal backend の compatibility helper として段階的に移す。
- この Phase は terminal backend の型境界を先に作るための橋渡しであり、最初の complete backend は Web Playground のままとする。

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
stdlib/alloc/gui/test.nepl
```

最初は `WidgetId`、`ActionId`、`ViewNode`、`GuiEffect`、`Update`、`LayoutContext`、widget descriptor、retained tree、theme、semantic tree、mock replay helper を小さく入れる。Closure callback、DOM、terminal raw code、OS handle は入れない。

2026-06-01 checkpoint では、`Update.effects` の境界を `GuiEffectBatch` に変更し、`alloc/gui/layout` の `TextMeasurer` 注入、`alloc/gui/widget` の button / label descriptor、`alloc/gui/tree` の bounded retained tree、`alloc/gui/theme` の typed palette / metrics、`alloc/gui/accessibility` の semantic tree 初期 slice まで実装した。`GuiEffectBatch` は現時点では capacity 2 の bounded data であり、`alloc` collection 側の owner contract が安定した段階で `Vec GuiEffect` へ置き換える。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_app.n.md --no-tree -o tmp/gui-app-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_layout.n.md --no-tree -o tmp/gui-layout-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_widget.n.md --no-tree -o tmp/gui-widget-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_tree.n.md --no-tree -o tmp/gui-tree-phase3.json -j 1 --dist web/dist --assert-io
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
stdlib/std/gui/ime.nepl
stdlib/std/gui/accessibility_host.nepl
stdlib/std/gui/error_display.nepl
```

目的:

- `GuiHost`、`WindowId`、`SurfaceId`、`TimerId`、`HostTextMeasurer`、`ImeBridge`、`AccessibilityHost` の標準境界を定義する。
- application は `GuiHost` を直接呼ばず、`GuiEffect` を runtime が解釈する。
- capability unsupported を `GuiError::Unsupported` として返す。
- `TextMeasurer` contract は `core/gui` に置き、ここでは host font / browser / terminal / native text stack へ接続する実装を扱う。

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
nodesrc/tui_regression.js
```

目的:

- `platforms/wasix/tui` の raw storage / ANSI / TTY helper を terminal backend implementation detail に押し下げる。
- `buffer_new` / `buffer_present_diff` の raw handle API を `TextGridRenderTarget` / `TerminalFrame` / `GuiHost.present` へ置き換える。
- `features/tui` の利用者向け path は壊さず、内部を新 substrate に差し替える。

互換維持:

- `features_tui.n.md` は当面維持する。
- 新規 test は `tests/stdlib/gui_terminal.n.md` に追加し、旧 TUI helper と新 TextGrid backend の対応を固定する。

## Phase 7: Embedded backend

目的:

- Phase 1 で固定した `core/gui` no_alloc contract を real-style backend で再検査する。
- `MockDrawTarget` と fixed-capacity dirty region を用意する。
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

tests/stdlib/gui_theme.n.md
    typed color role lookup
    metric validation
    Option FontId

tests/stdlib/gui_accessibility.n.md
    Semantic node state
    bounded semantic tree insertion

tests/stdlib/gui_terminal.n.md
    TextGrid capability
    terminal event mapping model
    legacy TUI facade bridge
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
