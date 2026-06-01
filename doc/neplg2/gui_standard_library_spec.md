# NEPLg2 GUI/TUI 標準ライブラリ仕様

作成日: 2026-06-01

## 目的

NEPLg2 の GUI 標準ライブラリは、単一の GUI framework ではなく、Web Playground、native desktop、mobile、embedded、terminal UI を同じ application model へ接続する UI substrate として定義する。

この仕様では、GUI と TUI を別系統の library として育てない。TUI は text-cell surface を持つ backend であり、GUI と同じ event、capability、layout、application update、host effect の抽象に載せる。既存の `features/tui` / `platforms/wasix/tui` は、最終的にこの共通 substrate の terminal backend として再設計・再実装する。

## 設計原則

- 最下層は embedded を最低制約として扱い、heap allocation、OS、window system、clipboard、DOM、GPU、font shaping に依存しない。
- Web Playground は最初の実動 backend として扱う。
- native desktop と mobile は Web や embedded の派生ではなく、host interface を実装する別 backend として扱う。
- public standard API は `Canvas`、`DOM`、`UIKit`、`Android View`、`Win32`、`Wayland`、`Skia`、`SSD1306` などへ直接依存しない。
- TUI 固有 API も raw ANSI helper 集ではなく、text-cell render target、terminal host、keyboard/text input backend として GUI substrate に接続する。
- error と error display は分離し、失敗は `Option` / `Result` と enum で表す。
- callback-heavy widget を避け、widget は `ActionId` を持ち、application `update` が `GuiEvent` を `match` する。

## 層構造

依存方向は次に固定する。

```text
core/gui
    ↑
alloc/gui
    ↑
std/gui
    ↑
platforms/gui/web
platforms/gui/native
platforms/gui/mobile
platforms/gui/embedded
platforms/gui/terminal
```

逆方向の依存は禁止する。

```text
core/gui -> alloc/gui          禁止
core/gui -> std/gui            禁止
core/gui -> platforms/gui/web  禁止
alloc/gui -> DOM / OS API      禁止
std/gui -> application model   原則禁止
```

## Rendering Model

標準 pipeline は次である。

```text
State + Event -> State + Effects
State -> ViewTree
ViewTree + LayoutContext -> LayoutTree
LayoutTree + RenderContext -> DrawCommand stream
DrawCommand stream -> RenderTarget
RenderTarget -> DrawTarget fallback
DrawTarget -> display / canvas / framebuffer / image
```

TUI では `RenderTarget` が text-cell command を受け取り、terminal host が ANSI / alternate screen / raw mode などへ変換する。ANSI escape sequence は `platforms/gui/terminal` または `platforms/wasix/tui` backend の実装詳細であり、`alloc/gui/widget` には漏らさない。

## Layer 1: Pixel Drawing

`core/gui` の pixel layer は no_alloc で成立する。

```text
core/gui/geometry
    GuiPoint
    GuiSize
    GuiRect
    GuiInsets
    GuiScaleFactor

core/gui/color
    BinaryColor
    Gray8
    Rgb565
    Rgb888
    Rgba8888

core/gui/pixel
    Pixel Color

core/gui/draw_target
    DrawTarget Color Error
    FlushTarget Error
    Drawable Color Output
```

`DrawTarget` は「描けること」だけを表す。`flush` は display / framebuffer / browser / GPU surface ごとに presentation model が異なるため、`FlushTarget` として分ける。

## Layer 2: Command Rendering

Pixel-only API は embedded では有効だが、Web、native、mobile、terminal では抽象度が低すぎる。標準 API は高水準 command stream を持つ。

```text
DrawCommand:
    FillRect GuiRect GuiPaint
    StrokeRect GuiRect GuiStroke
    Line GuiPoint GuiPoint GuiStroke
    TextRun TextRunId GuiPoint TextPaint
    ImageRect ImageId GuiRect
    PushClip GuiRect
    PopClip
    PushTransform Transform2d
    PopTransform
    TextCellRun TextGridPoint TextCellRun
```

`TextCellRun` は TUI / terminal backend のための command である。terminal は pixel display ではないため、GUI と完全に同じ `FillRect` だけへ押し込めない。共通化する境界は widget / event / layout / effect / capability であり、render command は surface kind ごとの最小差を enum と capability で表す。

## Layer 3: UI Tree And App Model

Application model は次である。

```text
App Model:
    init   %fn unit Model
    update %fn Model GuiEvent (Update Model)
    view   %fn Model ViewTree

Update Model:
    model   Model
    effects Vec GuiEffect
```

Widget は closure を保持しない。例えば button は action identifier を持つ。

```text
ButtonConfig:
    id WidgetId
    text str
    action ActionId
    style ButtonStyle
```

`GuiEvent::Action action` を application の `update` が受け取り、`match` で処理する。

`WidgetId` と `ActionId` は `core/gui/event` が所有する。理由は、raw pointer / keyboard / lifecycle event と同じ no_alloc event identity として扱う必要があるためである。`alloc/gui` はそれらの id を `ViewTree`、`LayoutTree`、widget descriptor の意味へ結び付ける。platform backend は raw input を typed event へ変換してよいが、application action の意味付けを platform 固有 code に閉じ込めてはいけない。

## Event Model

標準 event は全 platform を表現できる union とする。

```text
GuiEvent:
    Pointer PointerEvent
    Keyboard KeyboardEvent
    TextInput TextInputEvent
    Window WindowEvent
    Timer TimerEvent
    Lifecycle LifecycleEvent
    Accessibility AccessibilityEvent
    Action ActionId
```

Mobile では `LifecycleEvent` が必須である。Native desktop では一部だけ使う。Embedded や TUI では多くを使わない場合があるが、使わない event があることと、型体系上 event を持たないことは別である。標準 API は全 platform を表現できる event enum を持ち、実際の有無は `GuiCapabilities` で示す。

TUI は keyboard event と text input event の両方を持つ。raw key sequence は backend が `KeyboardEvent` / `TextInputEvent` へ正規化し、focus 移動や activate は `std/gui/keymap` が `KeyboardEvent` と `FocusKeyMap` から `Option FocusRouteCommand` へ変換する。`GuiEvent::Action` は `alloc/gui/routing/focus` が current focus と widget descriptor を見て発行する。application model は ANSI byte sequence や DOM key string を直接扱わない。terminal input については `platforms/gui/terminal/input` が `TerminalInputEvents` を返し、Space のように keyboard と text input の両方になりうる入力を `Option` pair として保持する。std layer は navigation key code と modifier bit の正規化 contract を持つが、ANSI / CSI byte pattern 自体は持たない。

## Routing Model

Event routing は `alloc/gui/routing` が所有する pure data 変換である。標準経路は次である。

```text
LayoutTree + GuiPoint
    -> Option WidgetId

ViewTree + WidgetId
    -> Option WidgetDescriptor

WidgetDescriptor
    -> Option GuiEvent
```

Pointer hit test は `GuiRect` の half-open bounds を使う。つまり left/top は含み、right/bottom は含まない。bounded checkpoint では child が root より前面にあり、second child は first child より前面にある。disabled widget は hit しても `Option::None` を返し、panic や silent raw event にはしない。

Focus traversal、keyboard mapping、focus routing は別契約である。`alloc/gui/focus` は `ViewTree` から `FocusOrder` を作り、current focus id から next / previous focus target を返すだけで、application event は発行しない。`std/gui/keymap` は platform raw input から切り離された `KeyboardEvent` を `FocusRouteCommand` へ変換する。`alloc/gui/routing/focus` は解釈済みの `FocusRouteCommand` を受け取り、結果を次の enum で返す。

```text
FocusRouteCommand:
    Next
    Previous
    Activate

FocusRouteResult:
    Ignored
    MoveFocus WidgetId
    Emit GuiEvent
```

`Next` / `Previous` は focus 移動だけを `MoveFocus` として返す。`Activate` は current focus id が指す widget の action event だけを `Emit` として返す。current focus が `None`、古い `WidgetId`、disabled widget、action を持たない widget、端で移動先がない traversal は `Ignored` になる。Tab、Shift+Tab、Enter、Space の portable default mapping は `std/gui/keymap` の `FocusKeyMap` が所有する。terminal raw byte / escape sequence、DOM keyboard event、OS virtual key は platform backend が `KeyboardEvent` / `TextInputEvent` へ正規化し、platform から `FocusRouteCommand`、`GuiEvent::Action`、application-specific `ActionId` を直接作らない。

標準の keyboard focus 経路は次で固定する。

```text
platform raw input
    -> KeyboardEvent / TextInputEvent
KeyboardEvent + FocusKeyMap
    -> Option FocusRouteCommand
FocusRouteCommand + FocusOrder + ViewTree
    -> FocusRouteResult
FocusRouteResult::Emit
    -> GuiEvent::Action
```

`FocusRouteCommand` は focus intent だけを表す。application 固有の action の意味は application `update` が `GuiEvent::Action` を `match` して決める。focus state の保持と `MoveFocus` の反映は runtime または application model の上位状態が担当し、`alloc/gui/routing/focus` 自体は純粋関数として状態を変更しない。

Terminal input の現 checkpoint は 1 byte ASCII subset、3 byte ESC sequence の一部、4 byte CSI tilde sequence の一部、xterm style の bounded 6 byte modifier arrow sequence を扱う。Tab(9) は key code 9、LF(10) / CR(13) は key code 13、Space(32) は key code 32 と text input `' '`、printable ASCII(33..126) は text input のみへ正規化する。`ESC [ Z` は Shift+Tab として key code 9、modifier bit 1、text input なしの `KeyboardEvent` へ正規化する。`ESC [ A/B/C/D` は std navigation key code の ArrowUp / ArrowDown / ArrowRight / ArrowLeft へ正規化する。`ESC [ H/F` と `ESC [ 1/4 ~` は Home / End、`ESC [ 3 ~` は Delete の typed key code へ正規化する。`ESC [ 1 ; <modifier> A/B/C/D` は modifier byte `2..8` を Shift / Alt / Control bitset へ変換し、arrow key の `KeyboardEvent` として返す。範囲外 byte、不正な CSI numeric parameter、または認識済み arrow key に対する範囲外 modifier parameter は `GuiError::InvalidCommand` である。範囲内で未対応の control byte、未知の 3 byte sequence、valid shape だが未対応 final key の CSI sequence、未対応だが valid な CSI tilde numeric parameter は event なしとして扱う。Function key、IME/text-edit context による Enter / Tab の text 化、途中入力 buffering は後続 slice で実装する。

この routing は callback を呼ばず、clipboard、window、terminal、DOM、OS API に触れない。pointer capture、gesture recognition、pointer cancel、IME focus、accessibility focus、recursive tree traversal は `std/gui` / `platforms/gui/*` と連携する上位状態であり、現 checkpoint では未実装である。ただし TUI でも keyboard / focus routing は最終的に `FocusRouteCommand` と `GuiEvent::Action` を使い、application が raw ANSI sequence を直接 `match` する経路を作らない。

## Capability Model

Backend は capability を公開する。

```text
GuiCapabilities:
    surface_kind SurfaceKind
    color_format ColorFormat
    has_allocator bool
    has_windowing bool
    has_multi_window bool
    has_pointer bool
    has_touch bool
    has_keyboard bool
    has_text_input bool
    has_clipboard bool
    has_accessibility bool
    requires_flush bool
    max_texture_width Option i32
    max_texture_height Option i32
```

`SurfaceKind` は少なくとも次を持つ。

```text
SurfaceKind:
    Pixel
    Command
    TextGrid
    Headless
```

TUI backend は `SurfaceKind::TextGrid` を返す。GUI pixel / canvas backend は `Pixel` または `Command` を返す。

Unsupported operation は panic や silent no-op ではなく `GuiError::Unsupported` を返す。ただし、仕様として best-effort と明記した effect だけは no-op を許す。

## Invalidation And Dirty Region

再描画に関する用語は 3 層に分ける。

```text
alloc/gui/diff
    GuiInvalidation
    retained tree の semantic invalidation

core/gui/dirty_region
    DirtyRegion
    no_alloc framebuffer / embedded 向け rect dirty contract

core/gui/dirty_region_set
    DirtyRegionSet
    no_alloc fixed-capacity rect set contract

platforms/gui/*
    DOM patch、terminal line diff、GPU surface damage、framebuffer compression
    backend implementation detail
```

`GuiInvalidation` は widget tree や layout tree のどこが古くなったかを表す。`DirtyRegion` は `GuiRect` を使い、embedded / framebuffer backend が redraw area を allocator なしで表すための値である。現 checkpoint の `DirtyRegion` は `Empty` / `Rect` / `Full` のみを持ち、複数 rect は保持せず、Rect 同士の merge は bounding rect へ O(1) で畳む。

`DirtyRegionSet` は embedded / framebuffer 向けの fixed-capacity no_alloc rect set contract である。現 checkpoint は最大 2 個の `GuiRect` を保持し、3 個目の追加は silent no-op や panic ではなく `Full` 状態への昇格として表す。負の width / height は `GuiError::InvalidGeometry` として拒否し、x / y の負値は相対座標として許容する。zero-size rect は有効な `GuiRect` として扱い、必要なら backend 側が present 時に無視できる。DOM patch、terminal line diff、GPU surface damage compression は standard API の semantic diff ではなく backend detail とする。

## Text Model

Text は platform 差が大きいため 3 層に分ける。

```text
core/gui
    TextRunId
    TextPaint
    FontId
    FontMetrics
    TextBounds
    TextMeasureRequest
    TextMeasureResult
    TextMeasurer

alloc/gui/text
    TextBuffer
    TextLayout
    LineBreak
    TextStyle
    CachedTextLayout

std/gui/text_measure
    HostFont
    FontLoadEffect

std/gui/ime
    ImeBridge
    ImeRequest
    ImeState
```

Layout は text measurement に依存するため、`LayoutContext` に `TextMeasurer` contract を注入する。この contract は `core/gui` 側の data / trait として定義し、`alloc/gui/layout` が `std/gui` に依存しないようにする。`std/gui/text_measure` と `platforms/gui/*` は host font、browser API、terminal cell width などを使って `TextMeasurer` を実装する。

`alloc/gui/layout` は browser global、OS font API、terminal escape sequence を直接呼ばない。font loading、IME、complex shaping などの side effect は `std/gui` または `platforms/gui/*` に閉じ込める。

TUI text measurement は terminal cell width を返す `TextMeasurer` 実装として扱う。現行の `platforms/wasix/tui/text/width.nepl` にある表示幅近似は、将来 `platforms/gui/terminal` 側の measurer へ移す。

## Mobile Lifecycle Contract

Mobile backend は native desktop の一種として扱わない。少なくとも次の状態遷移を event contract として表す。

```text
MobileLifecycleState:
    NotStarted
    ForegroundActive
    ForegroundInactive
    Background
    Suspended
    SurfaceLost
```

`LifecycleEvent` は `Started`、`Suspended`、`Resumed`、`Backgrounded`、`Foregrounded`、`LowMemory` に加えて、surface recreation と IME state の失効を表せる extension point を持つ。surface が失われた後の draw / present は panic ではなく `GuiError::SurfaceUnavailable` を返す。runtime は `RequestRedraw`、timer、IME composition、accessibility focus を lifecycle state と照合して処理する。

## Accessibility Model

Visual tree と semantic tree は分ける。

```text
Visual tree:
    ViewTree
    LayoutTree
    DrawCommand

Semantic tree:
    AccessibilityTree
    SemanticNode
    Role
    Label
    State
    Action
```

Accessibility は drawing の副産物にしない。`DrawCommand` から button の意味は復元できないため、widget layer が semantic tree を生成する。

## Host ABI

Host boundary は WIT-like な schema として設計する。ただし、最初の Web Playground backend は TypeScript / JavaScript shim で同じ ABI shape を実装してよい。

```wit
package neknaj:gui;

interface surface {
    type window-id = u32;
    type surface-id = u32;

    request-redraw: func(window: window-id) -> result<_, gui-error>;
    begin-frame: func(window: window-id) -> result<frame-id, gui-error>;
    push-command: func(frame: frame-id, command: draw-command) -> result<_, gui-error>;
    end-frame: func(frame: frame-id) -> result<_, gui-error>;
    present-commands: func(window: window-id, commands: list<draw-command>) -> result<_, gui-error>;
}

interface events {
    poll-event: func() -> option<gui-event>;
}

interface capabilities {
    get-capabilities: func() -> gui-capabilities;
}

interface text {
    measure-text: func(run: text-run) -> result<text-metrics, text-error>;
}

interface ime {
    set-ime-state: func(window: window-id, state: ime-state) -> result<_, gui-error>;
}

interface accessibility {
    update-tree: func(window: window-id, tree: accessibility-tree) -> result<_, gui-error>;
}

world gui-app {
    import surface;
    import events;
    import capabilities;
    import text;
    import ime;
    import accessibility;

    export init: func();
    export update: func(event: gui-event) -> result<update-result, gui-error>;
    export render-frame: func(window: window-id) -> result<_, gui-error>;
}
```

`present-commands` は `Vec DrawCommand` を使える backend 向けの convenience path である。embedded / no_alloc target は `begin-frame`、`push-command`、`end-frame` の streaming path を使い、command list allocation を要求されない。

TUI terminal backend は同じ world の surface implementation として `TextGrid` surface を提供する。Raw mode、TTY state、ANSI cursor movement、alternate screen は backend detail である。

## Public Module Contract

最小契約は次である。

```text
core/gui:
    GuiPoint
    GuiSize
    GuiRect
    GuiInsets
    GuiScaleFactor
    WidgetId
    ActionId
    BinaryColor
    Gray8
    Rgb565
    Rgb888
    Rgba8888
    Pixel
    DrawTarget
    FlushTarget
    DirtyRegion
    DirtyRegionSet
    DrawCommand
    RenderTarget
    GuiEvent
    GuiCapabilities
    GuiError

alloc/gui:
    ViewTree
    WidgetId semantics and re-export
    ActionId semantics and re-export
    LayoutTree
    Routing
    Theme
    TextBuffer
    TextLayout
    AccessibilityTree
    App Model
    Update Model
    GuiEffect
    MockGuiHost
    SnapshotTest

std/gui:
    GuiHost
    WindowId
    SurfaceId
    Runtime
    FocusKeyMap
    KeyboardEvent to FocusRouteCommand mapping
    HostTextMeasurer
    ResourceHandle
    Clipboard
    Timer
    ImeBridge
    ErrorDisplay

platforms/gui:
    web host
    native host
    mobile host
    embedded host
    terminal host
    terminal input normalization
```

## Current Implementation Status

契約と現状実装は分けて扱う。2026-06-01 時点の checkpoint は次である。

| Layer | Contract | Current implementation |
|---|---|---|
| `core/gui/geometry` | `GuiPoint`、`GuiSize`、`GuiRect`、`GuiInsets`、`GuiScaleFactor` | constructor / accessor / basic arithmetic を実装済み |
| `core/gui/color` | `BinaryColor`、`Gray8`、`Rgb565`、`Rgb888`、`Rgba8888`、`GuiColor` | constructor / accessor を実装済み |
| `core/gui/event` | `GuiEvent`、`ActionId`、`WidgetId`、pointer / keyboard / lifecycle data | initial data contract を実装済み |
| `core/gui/error` | enum-based GUI/TUI error data | `Unsupported`、`InvalidGeometry`、`ResourceExhausted`、`InvalidCommand` を実装済み。`SurfaceUnavailable` など mobile lifecycle extension error は未実装 |
| `core/gui/text_measure` | `TextMeasurer` contract、request/result、font id | borrowed `&Self` contract と fixed-cell `MockTextMeasurer` を実装済み。host font 実装は `std/gui` / platform 側で継続 |
| `core/gui/draw_target` | `DrawTarget`、`FlushTarget`、pixel-level drawing | `MockDrawTarget` と O(1) contract test を実装済み。iterator stream と rasterizer は未実装 |
| `core/gui/render_target` | streaming `RenderTarget` | `MockRenderTarget` を実装済み。command list / fallback rasterizer は未実装 |
| `core/gui/dirty_region` | no_alloc `DirtyRegion`、checked rect constructor、O(1) merge | `Empty` / `Rect` / `Full` と bounding rect merge を実装済み |
| `core/gui/dirty_region_set` | no_alloc fixed-capacity rect set | 最大 2 rect の `DirtyRegionSet`、overflow to Full、checked push を実装済み。generic capacity と backend damage compression は未実装 |
| `alloc/gui/app` | callback-free app model、`ViewNode`、`GuiEffect`、`Update` | leaf view、button config、redraw/title effect、bounded `GuiEffectBatch` を実装済み。将来の `Vec GuiEffect` へ置換する境界は `Update.effects` に固定 |
| `alloc/gui/layout` | `LayoutContext`、constraints、text measurement injection、measure/place result | `TextMeasurer` 注入、constraint validation、fixed text measure、place-at helper を実装済み。tree layout と flex/grid/scroll は未実装 |
| `alloc/gui/widget` | callback-free widget descriptor、action event、semantic lowering、measure bridge | button / label descriptor、`ActionId` event 生成、semantic node 生成、layout measure bridge、focusable accessor を実装済み |
| `alloc/gui/tree` | retained `ViewTree` / `LayoutTree`、allocator-backed arena、focus target query | root + 2 child の bounded tree、capacity error、first focusable id、focusable count に加え、parent index / depth を持つ `ViewTreeArena` / `LayoutTreeArena`、owner-recovery 付き arena child insertion、arena focus count / first focusable query を実装済み。arena を使った focus / routing / diff / layout traversal 統合は未実装 |
| `alloc/gui/focus` | platform 非依存 focus order / next / previous traversal | bounded `ViewTree` から `FocusOrder` を作り、current id から next / previous focus target を `Option WidgetId` で返す実装を追加済み。wrap policy と recursive tree traversal は未実装 |
| `alloc/gui/routing` | `LayoutTree` hit test、`WidgetId` lookup、widget action lowering、focus command routing | bounded root + 2 child の pointer action routing と `FocusRouteCommand` / `FocusRouteResult` を実装済み。pointer capture、gesture、recursive tree traversal は未実装 |
| `alloc/gui/diff` | retained tree diff / invalidation data contract | bounded `ViewTree` の slot diff、`GuiInvalidation::Clean` / `Widget` / `Tree` を実装済み。terminal line diff、DOM patch、dirty rect compression は platform 側で未実装 |
| `alloc/gui/text` | platform 非依存 `TextBuffer` / checked edit storage / measured text layout data | `TextBufferId`、`TextBuffer`、checked insert / replace / delete を `Result TextBuffer GuiError` で実装済み。`TextLayout` は injected `TextMeasurer` だけを使って測定し、byte length、char count、fallback cell count、width / height / baseline、max width を保持する。`CachedTextLayout` は buffer id、run id、font id、max width、byte length、char count から deterministic cache key を作る。line break、text hash / revision based invalidation、complex shaping cache は未実装 |
| `alloc/gui/theme` | typed theme scheme / color role / metric role、fallible color/metric helper | `GuiColor` palette、`ThemeMetrics` validation、`Option FontId`、text-cell style helper を実装済み。full typography / component style は未実装 |
| `alloc/gui/accessibility` | semantic node / role / state / action tree | bounded semantic tree の初期 slice を実装済み。host accessibility bridge は `std/gui` / platform 側で継続 |
| `std/gui` | host/runtime/window/timer/text/IME/accessibility/error display/keymap contract | typed data contract、core `TextMeasurer` host wrapper、`GuiEffectBatch -> GuiRuntimeCommandBatch` 解釈、capability unsupported error、`FocusKeyMap` による Tab / Shift+Tab / Enter / Space から `FocusRouteCommand` への変換、std navigation key code と modifier bit accessor を実装済み。platform 実行は未実装。raw input normalization は `platforms/gui/*` 側で継続 |
| `platforms/gui/terminal` | terminal as `SurfaceKind::TextGrid` backend and terminal input normalization | `TerminalProfile` と core `TextCellRun` based frame、1 byte ASCII subset、`ESC [ Z`、`ESC [ A/B/C/D/H/F`、`ESC [ 1/3/4 ~`、`ESC [ 1 ; <modifier> A/B/C/D` から `TerminalInputEvents` への正規化を実装済み。custom capability と grid size は `Result` で検証し、TextGrid 以外や負 size を拒否する。ANSI / TTY present、Function key などの追加 CSI sequence、途中入力 buffering は未実装 |

この表にない Web / native / mobile / embedded backend、arena-backed tree を使う recursive focus / routing / diff / layout integration、recursive diff / invalidation、recursive event routing、pointer capture / gesture、Web / native / mobile raw keyboard normalization、terminal の Function key などの追加 ANSI / CSI sequence と途中入力 buffering、text line break / text hash based cache invalidation、resource loading、real host presentation は未実装である。

## TUI Migration Contract

既存 TUI は `features/tui` と `platforms/wasix/tui` に直接 helper が露出している。これを段階的に次へ移す。

```text
features/tui
    legacy-compatible facade
    -> features/gui + terminal profile helper

platforms/wasix/tui
    existing WASIX terminal backend
    -> platforms/gui/terminal/wasix

platforms/wasix/tui/text
    display width / wrap helpers
    -> std/gui/text_measure terminal measurer + alloc/gui/text layout helpers

platforms/wasix/tui/input
    raw byte / ANSI keyboard helper
    -> platforms/gui/terminal/input + std/gui/keymap + alloc/gui/routing/focus

platforms/wasix/tui/buffer
    raw line buffer and diff present
    -> RenderTarget TextGrid + terminal host present

platforms/wasix/tui/ansi
    ANSI output helper
    -> terminal backend implementation detail
```

移行中は既存 import path を壊さない。`features/tui` は compatibility facade とし、内部実装を GUI substrate へ寄せる。新しい application は `features/gui` / `alloc/gui` / `std/gui` を使い、terminal target では terminal backend を選ぶ。

## 参考

- `embedded-graphics` の `DrawTarget` は最下層 drawing abstraction の参考にする。
- WebAssembly Component Model / WIT は host interface schema の参考にする。
- Zenn 設計指針の platform 依存隔離、`Option` / `Result` / enum、契約と現状実装の分離、試作段階でも雑設計を残さない方針を正の制約として扱う。
- NEPLg2 の既存 `core` / `alloc` / `std` / `platforms` 分離と NEPLg2.1 prefix 型式移行を正の制約として扱う。
