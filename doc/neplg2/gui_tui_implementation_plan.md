# NEPLg2 GUI/TUI 標準ライブラリ実装計画

作成日: 2026-06-01

## 目的

`doc/neplg2/gui_standard_library_spec.md` に基づき、GUI と TUI を共通 UI substrate として実装する。既存 TUI は保守対象として残すのではなく、terminal backend として段階的に再設計・再実装する。

この計画は `plan.md` を変更せず、NEPLg2.1 の現行 `stdlib/` 上で進める。

## 現状

- 明示的な GUI 標準ライブラリは存在し、`core/gui`、`alloc/gui`、`std/gui`、`platforms/gui/terminal` の初期 checkpoint まで進んでいる。
- 現在の実装は bounded data contract と flat arena tree の初期 checkpoint を優先した段階であり、arena を使う focus traversal、pointer routing、diff / invalidation、arena order の linear layout connector、parent-local stack layout policy、stack cross-axis alignment、overflow rejection は実装済みである。Web Playground には floating GUI window layer、backend-neutral command DTO、Canvas adapter、TypeScript 側 host frame decode / present 境界、`presentCommands` と `beginFrame` / `pushCommand` / `endFrame` streaming runtime bridge、NEPL stdout protocol parser、ActionId input target decode、typed input queue、SharedArrayBuffer event queue、Web-only `nepl_gui_web` input host import、Counter の NEPL-side wait/update/render loop、native には minifb optional window runner を追加済みである。flex / grid / scroll layout、stateful pointer routing、正式 Wasm presentation host import ABI、full `GuiEvent` input poll、mobile backend はまだ未実装である。
- `examples/gui_counter.nepl`、`examples/gui_life.nepl`、`examples/gui_mandelbrot.nepl` は GUI substrate の application update と render command stream を確認しつつ、`platforms/gui/web` の stdout frame protocol で Web Playground host へ frame を出力する。
- GUI/TUI の executable NEPLg2 code、stdlib doctest、`tests/stdlib/gui_*.n.md`、headless GUI examples は、括弧付き call を使わず、中間 `let` と pipeline で式境界を明示する方針に揃えた。prose の `O(1)` や WIT sketch は対象外である。
- 既存の近い資産は `features/tui` と `platforms/wasix/tui` である。
- `platforms/wasix/tui` は TTY ABI、ANSI 出力、text width、box line、line buffer、diff present を持つが、raw terminal helper と UI concept が同じ層に混ざっている。
- Web Playground は browser 上の editor / terminal と、その panel layout の上に重ねる floating GUI window layer を持つ。`host-bridge.ts` は unknown input の `present-commands` 風 frame を typed frame へ decode できる。`runtime-bridge.ts` は floating GUI manager を presenter として登録し、global `neplGuiHost.presentCommands` と `beginFrame` / `pushCommand` / `endFrame` から typed Result 境界で frame を渡せる。さらに `platforms/gui/web/stdout_protocol.nepl` と `web/src/gui-preview/stdout-protocol.ts` により、Web Playground の `Run` で実行された NEPL program の stdout frame stream が floating GUI window を開く。stdout helper は `GuiWebTextAlign` enum と `Result unit GuiError` で invalid geometry を返し、TypeScript parser は stdout fd=1 のみを protocol として扱い、frame 内 parse error では partial frame を破棄する。button hit target は `NEPLG2_GUI_ACTION_RECT` として出力され、Web 側は active run が表示した window の `GuiWebInputEvent::action` だけを typed queue から SharedArrayBuffer ring buffer へ渡し、worker の `nepl_gui_web.wait_action_id` import と `platforms/gui/web/input.nepl` の `Result Option ActionId GuiError` wrapper を通して NEPL app の update loop へ戻せる。正式な Wasm presentation host import ABI と full `GuiEvent` poll ABI はまだ未実装である。

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
stdlib/alloc/gui/layout/arena.nepl
stdlib/alloc/gui/layout/stack.nepl
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

2026-06-01 checkpoint では、`Update.effects` の境界を `GuiEffectBatch` に変更し、`alloc/gui/layout` の `TextMeasurer` 注入、`alloc/gui/widget` の button / label descriptor、`alloc/gui/tree` の bounded retained tree と parent index / depth を持つ `ViewTreeArena` / `LayoutTreeArena`、`alloc/gui/theme` の typed palette / metrics、`alloc/gui/accessibility` の semantic tree 初期 slice まで実装した。arena child insertion は owner-recovery error payload で tree owner と rejected child を返す。`GuiEffectBatch` は現時点では capacity 2 の bounded data であり、`alloc` collection 側の owner contract が安定した段階で `Vec GuiEffect` へ置き換える。

Focus traversal は `alloc/gui/focus` の platform 非依存 data contract として扱う。順方向 / 逆方向、wrap の有無、現在 focus が tree に存在しない場合の結果、disabled widget の除外を `Option` / enum で表し、host focus や accessibility focus の反映は `std/gui` 以降へ渡す。2026-06-01 の arena focus checkpoint では、bounded `FocusOrder` に任意長 arena を押し込めず、`ViewTreeArena` を直接走査して `focus_next_in_arena` / `focus_previous_in_arena` が `Option WidgetId` を返すようにした。

Focus routing は traversal とは別に `alloc/gui/routing/focus` へ置く。`FocusRouteCommand::Next` / `Previous` は focus movement だけを返し、`Activate` は current focus id の widget action だけを `GuiEvent::Action` として返す。戻り値は `FocusRouteResult::Ignored` / `MoveFocus WidgetId` / `Emit GuiEvent` で分ける。Tab、Shift+Tab、Enter、Space の portable default mapping は `std/gui/keymap` が `KeyboardEvent` と `FocusKeyMap` から `Option FocusRouteCommand` へ変換する。Arrow key や modifier bit の std contract は `std/gui/keymap` に置くが、ANSI escape sequence、DOM keyboard event、OS virtual key は platform backend が `KeyboardEvent` へ正規化し、application は raw key sequence を直接扱わない。

Event routing は `alloc/gui/routing` の pure data contract として扱う。pointer routing は `LayoutTree` hit test で `WidgetId` を得て、`ViewTree` の widget data から `GuiEvent::Action` を導出する。現 checkpoint は bounded root + 2 child、flat arena tree storage、arena focus next / previous traversal、arena pointer hit test / action lowering、half-open `GuiRect`、second child topmost、arena insertion order topmost、disabled widget suppression、layout hit だけで view widget が無い場合の `Option::None`、focus command routing、std keymap、terminal 1 byte input normalization、`ESC [ Z` Shift+Tab normalization、`ESC [ A/B/C/D` arrow key normalization、`ESC [ H/F` と `ESC [ 1/3/4 ~` の Home / End / Delete normalization、`ESC [ 1 ; <modifier> A/B/C/D` xterm modifier arrow normalization までを固定する。pointer capture、gesture、Web / native / mobile raw keyboard normalization、terminal の追加 ANSI / CSI sequence、途中入力 buffering は後続で実装する。TUI では keyboard / focus routing から同じ `FocusRouteCommand` / `GuiEvent::Action` を生成し、raw ANSI input を application が直接扱わないようにする。

Arena layout connector は `alloc/gui/layout/arena` に置く。現 checkpoint の `layout_view_tree_arena_linear` は `LayoutContext`、borrowed `ViewTreeArena`、`LayoutHint`、`LayoutConstraints` だけを使い、owner-consuming な `LayoutTreeArena` を返す。node は arena insertion order で y 方向へ配置し、parent index と depth は `ViewTreeArena` の構造を保つ。invalid constraints は `GuiError::InvalidGeometry`、空 tree や欠落 node は `GuiError::InvalidCommand` を返し、構築途中で失敗した `LayoutTreeArena` owner は connector が内部で解放する。

Stack layout policy は `alloc/gui/layout/stack` に置く。現 checkpoint の `layout_view_tree_arena_stack` は `StackLayoutPolicy` の `StackAxis`、spacing、`StackCrossAlignment`、`StackOverflowPolicy` を pure data として受け取り、同じ parent を持つ previous sibling の extent と spacing から parent-local offset を計算する。`Start` / `Center` / `End` は cross-axis position を決め、`Stretch` は vertical stack なら child width、horizontal stack なら child height を parent cross size にそろえる。overflow policy の `Allow` は現状互換の配置を維持し、clip / scroll は後続 layer に委ねる。`Reject` は parent bounds 外へ出る配置を `GuiError::InvalidGeometry` として拒否する。`ViewTreeArena` は borrow-only、成功時だけ `LayoutTreeArena` owner を返す。negative spacing、負 constraints、min > max constraints は `GuiError::InvalidGeometry`、壊れた parent index / 欠落 node は `GuiError::InvalidCommand` とし、途中で作った `LayoutTreeArena` owner は内部で解放する。flex grow、grid placement、scroll state、text buffer と arena node の対応付けは後続で実装する。

Diff / invalidation は `alloc/gui/diff` に置く。ここでは dirty widget / tree / layout などの共通 data contract だけを持ち、terminal line buffer diff、DOM patch、framebuffer dirty rect compression は `platforms/gui/*` の実装詳細にする。現 checkpoint では bounded `ViewTreeDiff` に加え、allocator-backed `ViewTreeArenaDiff` が node count、shape change、content change count、単一 changed `WidgetId` を保持する。arena owner は消費せず、parent index / depth / slot `WidgetId` の変化は `GuiInvalidation::Tree`、単一 content change は `GuiInvalidation::Widget id`、複数 content change は `GuiInvalidation::Tree` へ畳む。

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

2026-06-02 checkpoint では、Web Playground workspace に `gui-preview` pane を追加した後、editor の panel layout の上に重なる independent floating GUI window layer へ表示経路を移した。window は minimize、maximize / restore、drag move、edge / corner resize、dock restore を持つ。`window-manager.ts` の transient state は `idle` / `drag` / `resize`、source は `source-path` / `preview-kind` / `host-frame`、window mode は `normal` / `minimized previousMode` / `maximized restoreRect`、dock button は `none` / `mounted` 形式の union で表し、maximize 中に minimize しても original restore rect を失わない。top bar の `GUI` button と editor header の `G` button は削除し、Web Playground の `Run` で実行された NEPL program が stdout frame protocol を出した時だけ host-frame window を開く。`window-manager.ts` と `panel.ts` には `null` / `undefined` / non-null assertion に依存しない source policy を追加した。

`web/src/gui-preview/commands.ts` は `fill-rect` / `text-run` の preview command DTO、`rgba8888` 相当の color struct、text align enum、command frame、`action-rect` input target を持つ。`web/src/gui-preview/renderer.ts` は compatibility preview fixture として残すが、Run 経路の Mandelbrot / Life / Counter 表示は TS simulation ではなく `examples/gui_*.nepl` が出す stdout protocol によって駆動する。host frame 表示中の `panel.ts` は compatibility scene を生成せず、NEPL 実行結果の command frame だけを描画する。`web/src/gui-preview/host-bridge.ts` は unknown input を `GuiWebHostResult` の `ok` / `err` union で decode し、invalid color byte、unsupported command、invalid frame shape、invalid input target を typed error として返す。`web/src/gui-preview/runtime-bridge.ts` は presenter の未登録、global install 先の不正、streaming frame state、host decode error を `GuiWebRuntimeResult` の `ok` / `err` union で扱い、Playground 初期化時に floating GUI manager を global `neplGuiHost.presentCommands` と `beginFrame` / `pushCommand` / `endFrame` / `discardFrame` へ登録する。あわせて `neplGuiHost.takeInputEvents` / `resetInputEvents` を公開し、Web 側で queue された `GuiWebInputEvent::action` を Result として取り出せる。`web/src/gui-preview/input-bridge.ts` は listener にも typed event だけを通知し、`shared-event-queue.ts` は SharedArrayBuffer ring buffer として worker へ渡す。`web/src/runtime/worker.ts` は Web-only MVP host import module `nepl_gui_web` の `poll_action_id` / `wait_action_id` を提供し、`platforms/gui/web/input.nepl` は raw sentinel を `Result Option ActionId GuiError` へ正規化する。`web/src/terminal/shell.ts` は現在の run が stdout frame で present した window id だけを input queue 対象にし、stale window の click を実行中 app へ混入させない。Counter は button click、Life は next step / animate / cell pixel size、Mandelbrot は sample resolution を、通常実行時の `gui_web_wait_action_result` で NEPL 側 event loop に戻して `update` と `render` を再実行する。`web/src/gui-preview/stdout-protocol.ts` は stdout fd=1 の line protocol だけを typed command frame へ decode し、`NEPLG2_GUI_ACTION_RECT` を input target として保持し、frame 内 parse error では partial frame を fail-closed に破棄する。`web/src/terminal/shell.ts` が stdout chunk を parser に通して host frame を present し、run-wasm 開始/終了で GUI event queue を reset / inactive 化する。Canvas 固有の色変換、font、text align、scale / viewport 計算は `canvas-renderer.ts` に閉じ込める。DOM / Canvas / SharedArrayBuffer は Web frontend の backend detail であり、`core/gui`、`alloc/gui`、`std/gui` の public type には入れない。

次の Web checkpoint では stdout fallback ではなく、NEPL/Wasm runtime が正式 host import ABI 経由で `neplGuiHost.beginFrame` / `pushCommand` / `endFrame` 相当を呼ぶ接続を追加する。あわせて action id だけの MVP poll を full `GuiEvent` record poll へ拡張し、session id / window id の正式化と Mandelbrot progressive rendering を NEPL app の update loop で処理する。現在の stdout protocol と preview renderer は、その接続後に fallback / smoke fixture として残す。

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
- `platforms/gui/terminal/input.nepl` は terminal raw byte、3 byte ESC sequence、4 byte CSI tilde sequence、bounded 6 byte CSI modifier sequence を `TerminalInputEvents` へ正規化する。`ESC [ Z` は Shift+Tab として key code 9、modifier bit 1 へ正規化し、`ESC [ A/B/C/D` と `ESC [ 1 ; <modifier> A/B/C/D` は std navigation key code と modifier bitset へ正規化する。`ESC [ H/F` と `ESC [ 1/3/4 ~` は Home / End / Delete の typed key code へ正規化するが、`FocusRouteCommand` や `ActionId` は作らず、`std/gui/keymap` と `alloc/gui/routing/focus` の責務を保つ。
- `features/tui` の利用者向け path は壊さず、内部を新 substrate に差し替える。

互換維持:

- `features_tui.n.md` は当面維持する。
- 新規 test は `tests/stdlib/gui_terminal.n.md` に追加し、旧 TUI helper と新 TextGrid backend の対応を固定する。
- input normalization は `tests/stdlib/gui_terminal_input.n.md` に分け、Tab / LF / CR / Space / printable ASCII / invalid byte / unsupported control byte / Shift+Tab ESC sequence / arrow key ESC sequence / Home / End / Delete CSI sequence / xterm modifier arrow CSI sequence / unknown sequence / invalid numeric parameter / invalid modifier parameter を固定する。

## Phase 7: Embedded backend

目的:

- Phase 1 で固定した `core/gui` no_alloc contract を real-style backend で再検査する。
- `MockDrawTarget`、`DirtyRegion`、`DirtyRegionSet` を用意する。現 checkpoint の `DirtyRegionSet` は fixed capacity 2 の O(1) contract であり、generic capacity や backend-specific compression は後続で追加する。
- Optional `FlushTarget` を確認する。

## Phase 8: Native / Mobile backend

Native:

- window / surface / clipboard / timer / cursor を `std/gui` host contract に接続する。
- File dialog、menu、tray、drag-and-drop は extension module に分ける。

2026-06-02 checkpoint では、workspace member `nepl-gui-native` を追加し、pure framebuffer renderer と minifb window runner を分けて実装した。CI / headless 環境では `cargo test -p nepl-gui-native --lib` が framebuffer 変換と metric contract だけを検査する。実 window は target-specific optional dependency の `window` feature で有効化し、`cargo run -p nepl-gui-native --features window -- mandelbrot` のように明示実行する。

この crate も現時点では正式な `std/gui::GuiHost` 実装ではない。次の native checkpoint では `std/gui` の host contract が固まった後、`nepl-gui-native` の framebuffer renderer を `platforms/gui/native` 側の `present` 実装へ寄せる。

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
    ViewTreeArena to LayoutTreeArena linear connector
    invalid arena layout constraints
    StackLayoutPolicy vertical sibling spacing
    stack nested parent-local offset
    invalid stack policy error
    stack vertical center alignment
    stack vertical stretch alignment
    stack horizontal end alignment
    stack reject overflow error

tests/stdlib/gui_widget.n.md
    Button action event generation
    disabled action suppression
    semantic node generation

tests/stdlib/gui_tree.n.md
    bounded ViewTree child insertion
    ViewTreeArena nested insertion and owner-recovery error
    first focusable WidgetId
    LayoutTree child insertion
    LayoutTreeArena nested insertion

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
    ViewTreeArena single widget content invalidation
    ViewTreeArena shape / multiple content change invalidation

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
