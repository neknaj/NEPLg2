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
- GUI/TUI の executable NEPLg2 code は括弧付き call に戻さない。stdlib implementation、`//:` doctest、`tests/stdlib/gui_*.n.md`、`examples/gui_*.nepl` では、nested call を中間 `let`、block、pipeline で分け、prefix expression の式境界を明示する。通常文の `O(1)`、WIT sketch、非 NEPL pseudo code の括弧はこの制約の対象外である。

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
DrawCommand stream -> Rasterizer / RenderTarget
Rasterizer / RenderTarget -> PixelBuffer / TextGridFrame
PixelBuffer / TextGridFrame -> SurfacePresenter
```

TUI では `RenderTarget` が text-cell command を受け取り、terminal host が ANSI / alternate screen / raw mode などへ変換する。ANSI escape sequence は `platforms/gui/terminal` または `platforms/wasix/tui` backend の実装詳細であり、`alloc/gui/widget` には漏らさない。

Web の可視 canvas は正式経路では pixel buffer を `putImageData` で表示するだけである。Canvas2D の `fillRect`、`stroke`、`fillText`、`drawImage` を標準 GUI content renderer として使う経路は持たない。CLI-only、headless、bare、unsupported surface では別の表示経路へ自動で落とさず、capability と `GuiError::Unsupported` で表す。

2D rendering engine と font rendering engine は別々の描画規則を持たない。Path、SVG、image、button skin、glyph mask、ruby、math glyph は `Paint`、`Stroke`、`Shadow`、`BlendMode`、`Clip`、`Transform2d` を共有する。詳細は `doc/neplg2/gui_2d_rendering_design.md` と `doc/neplg2/gui_font_rendering_design.md` で定義する。

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
    init   %fn void Model
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

Allocator-backed tree では同じ contract を次の経路で扱う。

```text
LayoutTreeArena + GuiPoint
    -> Option WidgetId

ViewTreeArena + WidgetId
    -> Option WidgetDescriptor

WidgetDescriptor
    -> Option GuiEvent
```

Pointer hit test は `GuiRect` の half-open bounds を使う。つまり left/top は含み、right/bottom は含まない。bounded checkpoint では child が root より前面にあり、second child は first child より前面にある。`LayoutTreeArena` では arena insertion order の後方を前面として末尾から走査する。layout arena と view arena の対応は `WidgetId` identity で行い、parent index や arena storage index は routing の public identity にしない。disabled widget、または layout hit に対応する widget が view tree に存在しない場合は `Option::None` を返し、panic や silent raw event にはしない。

Focus traversal、keyboard mapping、focus routing は別契約である。`alloc/gui/focus` は bounded `ViewTree` から `FocusOrder` を作り、また allocator-backed `ViewTreeArena` を直接走査して、current focus id から next / previous focus target を返すだけで、application event は発行しない。`WidgetId` は arena storage index ではなく widget identity として比較する。`std/gui/keymap` は platform raw input から切り離された `KeyboardEvent` を `FocusRouteCommand` へ変換する。`alloc/gui/routing/focus` は解釈済みの `FocusRouteCommand` を受け取り、結果を次の enum で返す。

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

この routing は callback を呼ばず、clipboard、window、terminal、DOM、OS API に触れない。pointer capture、gesture recognition、pointer cancel、IME focus、accessibility focus、flex / grid / scroll などの本格 layout policy は `std/gui` / `platforms/gui/*` と連携する上位状態であり、現 checkpoint では未実装である。stack layout は `alloc/gui/layout/stack` の pure data policy として実装済みであり、axis / spacing、cross-axis alignment、overflow rejection を `Result` で扱う。ただし flex grow、grid placement、scroll state はまだ扱わない。TUI でも keyboard / focus routing は最終的に `FocusRouteCommand` と `GuiEvent::Action` を使い、application が raw ANSI sequence を直接 `match` する経路を作らない。

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
    WindowPixel
    OffscreenPixel
    DevicePixel
    TextGrid
    Headless
```

TUI backend は `SurfaceKind::TextGrid` を返す。Web / native window backend は `WindowPixel`、offscreen screenshot backend は `OffscreenPixel`、bare / embedded display backend は `DevicePixel`、GUI 表示を持たず virtual event replay だけを扱う backend は `Headless` を返す。旧文書や既存 checkpoint の `Pixel` は `WindowPixel` または `DevicePixel` へ分解する移行対象であり、`Command` は surface kind ではなく rasterizer へ渡す前の `DrawCommand stream` の概念として扱う。

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

`GuiInvalidation` は widget tree や layout tree のどこが古くなったかを表す。bounded `ViewTree` は slot diff を `ViewTreeDiff` として保持し、allocator-backed `ViewTreeArena` は `ViewTreeArenaDiff` として node count、shape change、content change count、単一 changed `WidgetId` を保持する。arena diff では parent index、depth、slot の `WidgetId` を tree shape / order として扱い、arena storage index を invalidation payload にしない。単一 content change は `GuiInvalidation::Widget id`、node count / shape / id 対応変更、または複数 content change は `GuiInvalidation::Tree` へ畳む。

`DirtyRegion` は `GuiRect` を使い、embedded / framebuffer backend が redraw area を allocator なしで表すための値である。現 checkpoint の `DirtyRegion` は `Empty` / `Rect` / `Full` のみを持ち、複数 rect は保持せず、Rect 同士の merge は bounding rect へ O(1) で畳む。

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

Layout は text measurement に依存するため、`LayoutContext` に `TextMeasurer` contract を注入する。この contract は `core/gui` 側の data / trait として定義し、`alloc/gui/layout` が `std/gui` に依存しないようにする。

Formal GUI text measurement は host browser / OS API の測定結果を authority にしない。正式経路は `FontResourceRequest -> GuiFontFace -> ScaledFont -> ShapedRun -> RenderedTextMetrics` であり、font file parsing、metrics、shaping、glyph rasterization から layout 用の寸法を得る。`std/gui/text_measure` の host wrapper は legacy smoke、terminal cell measurement、mock test、移行期 compatibility のための境界として扱い、formal GUI renderer の寸法決定には使わない。

`alloc/gui/layout` は browser global、OS font API、terminal escape sequence を直接呼ばない。font loading、IME、complex shaping などの side effect は `std/gui` または `platforms/gui/*` に閉じ込める。ただし font metrics / shaping / rasterization の authority は `alloc/gui/font` と `alloc/gui/text` の data contract に置く。

TUI text measurement は terminal cell width を返す `TextMeasurer` 実装として扱う。現行の `platforms/wasix/tui/text/width.nepl` にある表示幅近似は、将来 `platforms/gui/terminal` 側の measurer へ移す。

Outline font rendering、font resource loading、ruby / furigana、Japanese vertical writing、math inline layout との接続は `doc/neplg2/gui_font_rendering_design.md` で扱う。Formal GUI text renderer は `fonts/HackGenConsoleNF-Regular.ttf` を resource path として読み、font metrics、shaped run、positioned glyph、glyph rasterization を同じ font face から導く。Browser / OS text measurement は authority にせず、unsupported feature は typed error として扱う。

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

## Current Display Smoke Backends

2026-06-02 checkpoint では、正式な `neknaj:gui` host ABI へ到達する前の表示 smoke backend として、次を実装している。

```text
web/src/gui-preview
    Web Playground の floating GUI window layer、typed command DTO、runtime bridge、host frame decoder、NEPL stdout legacy smoke transport、bitmap video memory presenter

nepl-gui-native
    pure framebuffer renderer と resizable minifb window smoke backend
```

`examples/gui_mandelbrot.nepl`、`examples/gui_life.nepl`、`examples/gui_counter.nepl`、`examples/gui_calculator.nepl`、`examples/gui_scientific_calculator.nepl`、`examples/gui_paint.nepl`、`examples/gui_breakout.nepl` は、NEPL 側で application model、typed event update、render command frame を作る。現 checkpoint では `platforms/gui/web/stdout_protocol.nepl` を通して Web Playground host へ frame stream を出すが、これは formal host surface ABI へ到達する前の legacy smoke transport であり、same app code contract の正式 path ではない。stdout helper は platform backend detail として `GuiWebTextAlign` enum と `Result unit GuiError` を返す checked API を持ち、invalid geometry を panic や silent no-op にしない。text label を持つ button は `GuiWebButtonConfig` と `gui_web_stdout_button` で `fill_rect`、`text_run`、`action_rect` の順序を一箇所に集約し、example 側は app 固有の `ActionId`、label、色、geometry だけを渡す。Mandelbrot は Preview / HD / Detail action で sample grid と logical surface size を切り替え、HD / Detail mode は 1280x720 logical frame の raster 部分を `rgba-row` payload で描画する。さらに `--video-memory-once` は 32x18 の有限 surface を formal Web video memory row host import へ出す opt-in 検査 path であり、stdout protocol、command frame、TS simulation へ fallback しない。Life は next step、animate toggle、cell pixel size、HD view を扱う。Counter、四則電卓、関数電卓は button の `ActionId` を update で解釈する。Paint は button action だけでなく full `GuiWebEvent` の pointer position を model update に使う。Breakout は button action と timer tick で model を進める。各 button 領域は `NEPLG2_GUI_ACTION_RECT` で `ActionId` hit target として出力される。Web Playground の Run 経路では、この NEPL stdout frame stream が floating GUI window を開く。TypeScript はこれらの example を simulation せず、stdout frame decode と backend presentation だけを担当する。

Native smoke backend は macOS AppKit、Windows Win32、Linux Wayland / X11 の window lifecycle 調査を踏まえ、OS window manager が与える resize / close / event pump を受ける形へ寄せている。`WindowOptions.resize = true`、`ScaleMode::AspectRatioStretch`、`set_target_fps 60`、current window size 監視、letterbox-aware hit test、`NativeSurfaceState::Unavailable` を使い、固定 size framebuffer 前提の click mapping を避ける。調査内容と native backend contract は `doc/neplg2/gui_native_platform_behavior.md` に分けて記録する。これはまだ正式な `std/gui::GuiHost.present` 実装ではなく、minifb と native handle は標準 API の public type へ出さない。

Web Playground の表示 smoke は editor の panel layout の上に独立した DOM layer を置き、`GuiFloatingWindowManager` が minimize、maximize / restore、drag move、edge / corner resize、dock restore を扱う。これは native window と同等の基本操作を browser 上で検査するための backend UI であり、標準 API の window model ではない。`GuiFloatingWindowManager` の move state、source、window mode、dock state は discriminated union で表す。`minimized` mode は previous mode を保持するため、maximized window を minimize / restore しても original restore rect は失われない。top bar の `GUI` button と editor header の `G` button は user-facing 導線から外し、NEPL execution が stdout protocol を出した時だけ window を開く。host event / queue status は GUI window content に挟まず、折りたたみ式の `GuiWindowDebugPanel` へ分離する。通常の window body は host frame canvas だけを含む。host frame の title は window titlebar の表示責務であり、canvas renderer は同じ title を content 内へ再描画しない。debug panel は通常 window より低い補助 z-layer に置き、collapsed 時は toggle 以外の pointer capture を持たず、`aria-live` を off にして queue 更新を main GUI live region の読み上げ対象にしない。`window-manager.ts` と `panel.ts` が `null` / `undefined` / non-null assertion に頼らないこと、かつ debug/status DOM を window content に戻さないことを source policy regression で固定する。

Host frame の描画 data は `web/src/gui-preview/commands.ts` の `fill-rect` / `rgba-row` / `text-run` command union、`rgba8888` 相当の color struct、command frame、`action-rect` input target で表す。`rgba-row` は legacy smoke transport で HD raster の row payload を bounded command count で運ぶための現 checkpoint の command であり、Canvas や DOM 型を public DTO に入れない。旧 `renderer.ts` による Mandelbrot / Life / Counter の TS scene simulation は削除済みであり、Run 経路の GUI 表示は現 checkpoint では NEPL stdout protocol によって駆動する。`panel.ts` は host-frame surface として、NEPL 実行が出した command frame だけを描画する。`host-bridge.ts` は unknown input を `GuiWebHostResult` の `ok` / `err` union で decode し、invalid frame、invalid command、invalid rect、invalid color、invalid text、invalid input target、unsupported command を typed error として返す。`runtime-bridge.ts` は presenter missing、invalid install target、invalid frame state、host decode error、invalid video memory frame、video memory open / present failure を `GuiWebRuntimeResult` で返し、global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame` streaming path、`presentVideoMemory`、`closeWindow` を floating window presenter へ接続する。`presentVideoMemory` は `windowId`、`title`、`SharedArrayBuffer` を持つ video memory frame だけを受け付け、`ArrayBuffer`、typed array、numeric id、string handle、transferable object を typed error として拒否する。stdout protocol や command frame path への自動迂回は持たない。`panel.ts` は `none` / `command-frame` / `video-memory` の state を分け、同じ `SharedArrayBuffer` identity の opened surface を再利用する。video memory presentation は `ImageData` と `putImageData` だけで行い、surface size と drawable size が一致しない場合も CSS scale、Canvas transform、`drawImage` による伸縮を行わず、top-left に 1:1 で提示する。window resize は `WindowEventKind::Resized` として application に渡し、application 側が新しい pixel buffer size を決める。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` に `video_memory_create_surface`、`video_memory_acquire_write_slot`、`video_memory_write_slot_bytes`、`video_memory_write_rgba8888_row`、`video_memory_discard_write_slot`、`video_memory_publish_slot`、`video_memory_present_surface`、`video_memory_close_surface` を持つ。`surface_id` と `frame_id` は worker-local opaque positive integer であり、`SharedArrayBuffer`、DOM handle、Canvas handle、ArrayBuffer transfer object、JS object handle、string handle は NEPL/Wasm へ渡さない。`video_memory_present_surface` は typed `gui_video_memory_present` worker message と ack `SharedArrayBuffer` で main thread presenter の実結果を待ってから status を返す。`platforms/gui/web/surface.nepl` は raw negative status を module private helper で `Result` / `GuiError` へ写す。`web/src/gui-preview/stdout-protocol.ts` は stdout fd=1 の line protocol だけを typed command frame と typed animation timer request へ decode し、`NEPLG2_GUI_RGBA_ROW` を `rgba-row` command、`NEPLG2_GUI_ACTION_RECT` を frame-local input target として読む。chunk split、invalid frame state、invalid color、invalid rgba row、invalid action rect、invalid animation timer を discriminated error で表す。frame 内 parse error は partial frame を破棄し、壊れた frame を present しない。Web checkpoint の presentation hot path は video memory surface と `putImageData` only presenter に寄せる。DrawCommand stream / tile / bitmap / row / RLE を直接渡す正式 host import ABI は残件である。`web/src/gui-preview/input-bridge.ts` は DOM / Canvas に依存しない typed queue として `GuiWebInputEvent::action`、`GuiWebInputEvent::pointer`、`GuiWebInputEvent::keyboard`、`GuiWebInputEvent::text-input`、`GuiWebInputEvent::window`、`GuiWebInputEvent::timer` を保持し、listener へも typed event だけを通知する。DOM `KeyboardEvent` の key string は `panel.ts` で std key code contract と Unicode scalar value へ正規化され、application code へ DOM string は渡らない。Space は keyboard event と text input event の両方として queue し、composition 中、Meta shortcut、multi-scalar text は現 checkpoint では queue しない。pointer down / up は DOM `button` から changed button を正規化し、pointer move は DOM `buttons` bitmask から現在押下中の button state を正規化する。これにより paint のような app は hover move と primary drag を `PointerButton` で区別できる。floating host frame window は resize 時に `WindowEventKind::Resized` を worker queue へ渡す。stdout animation timer request は window id と timer id を持ち、Shell が browser timer を管理して `TimerEvent` を worker queue へ渡す。close button は現 checkpoint では拒否可能 close request ではなく host lifecycle signal として扱い、window を削除した後で active worker を interrupt する。terminal stop / process finish は `neplGuiHost.closeWindow` presenter path で host-frame window を削除し、active timer も停止する。`web/src/gui-preview/shared-event-queue.ts` は SharedArrayBuffer の full event queue と legacy action projection queue を分ける。full event queue は action / pointer / keyboard / text input / window / timer の kind、window id、action id、pointer milli-position、pointer kind、pointer id、button、keyboard kind、key code、modifier bit、text scalar value、window kind、window size、timer id、timer tick を worker へ渡す。record slot length は 8 のまま固定し、event kind ごとに payload slot を再利用する。action-only queue は `poll_action_id` / `wait_action_id` 互換 path が pointer / keyboard / text input / window / timer event を consume しないための projection である。queue は bounded だが、producer は `event-queue-full` / `action-queue-full` を返さない。容量に達した場合は古い unread record を明示的に押し出し、新しい input を受け入れる。full event poll と legacy action-only poll を同じ app run で混用すると action projection queue に残る event があるため、互換 path は action-only app 用である。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` の `poll_action_id` / `wait_action_id` に加えて、`poll_event_kind` / `wait_event_kind` と last-event field accessors を提供する。`platforms/gui/web/input.nepl` は raw sentinel を public API に出さず、`gui_web_wait_action_result` で unsupported host を `GuiError::Unsupported`、timeout を `Option::None`、action を `Option::Some ActionId` として正規化する。さらに `gui_web_wait_event_result` / `gui_web_poll_event_result` は `Result Option GuiWebEvent GuiError` を返し、現 checkpoint の action record を `GuiEvent::Action`、pointer down / move / up / cancel record を `GuiEvent::Pointer`、keyboard record を `GuiEvent::Keyboard`、text scalar record を `GuiEvent::TextInput`、window resize record を `GuiEvent::Window`、timer record を `GuiEvent::Timer` として保持する。text scalar は `char_from_i32_result` で検証し、surrogate や範囲外を `GuiError::InvalidCommand` にする。window kind と size も raw value を `WindowEventKind` と `GuiSize` へ正規化し、未知 kind や 0 以下の size は `GuiError::InvalidCommand` にする。timer id と tick は正の id と 0 以上の tick として検証し、不正な record は `GuiError::InvalidCommand` にする。`web/src/terminal/shell.ts` は active run が present した window id だけを queue 対象にし、stale window の input event が別 app に混入しないようにする。空 poll の busy spin を避けるため、interactive app は `wait_action_id` または `wait_event_kind` の Atomics wait path を使う。

`video_memory_discard_write_slot` は未公開 write frame の所有権を `Writing -> Free` に戻すためだけの Web backend import である。成功時は dirty metadata を消し、published / presented epoch は進めない。frame が存在しない、既に publish / discard 済み、または resize generation が古い場合は typed status を返し、stdout protocol、command frame、別 surface への fallback は行わない。

`video_memory_write_rgba8888_row` は formal row payload の最小 writer である。`write_slot_bytes` と違い app は byte offset を渡さず、origin、pixel width、source pointer だけを渡す。`width <= 0`、surface 範囲外、`width * 4` と一致しない source byte length は typed error で拒否し、clamp / truncate / zero-byte no-op は行わない。row write は pixel plane だけを更新し、dirty metadata、slot epoch、published epoch、presented epoch は publish path へ残す。

`examples/gui_video_memory_rows.nepl` は focused NEPL example として、`ByteBuilder` / `ByteBuf` owner で row bytes を構築し、borrowed `MemPtr u8` を `gui_web_video_memory_write_rgba8888_row` へ渡す。これは stdout `rgba-row` を使わない formal row host import の source contract を示すための例である。現行 CI の `run_test.js` は default `nepl_gui_web` video memory host import を unsupported stub として持つため、通常 doctest では host capability missing を `Result` に写す境界を壊さない。この example の happy path は fake positive `nepl_gui_web` host import harness が通常 path の NEPL/Wasm 実行として検査する。Mandelbrot も `--video-memory-once` で有限 preview model の RGBA8888 row payload を同じ host import path へ出す。Mandelbrot の resize event loop / surface recreate / true responsive high-resolution は後続 slice である。

pointer move と window state coalescing は順序保存 contract として扱う。`panel.ts` は pointermove を `requestAnimationFrame` 単位で最新の move へまとめ、`input-bridge.ts` の stored queue は直前に保持された同じ window id、pointer id、button の `PointerEventKind::Move` だけを最新座標へ置き換える。`shared-event-queue.ts` は write tail 直前の unread slot が同一 pointer move または同一 window kind の state record である場合だけ最新値へ置換でき、queue 全体の未読 slot は走査しない。timer tick も同じく直前の unread slot が同一 window id / timer id の `TimerEvent` である場合だけ最新 tick へ置換でき、action や pointer / keyboard / text / window event をまたいで更新してはいけない。`Down` / `Up` / `Cancel`、action、keyboard、text input、close lifecycle signal をまたいで古い move / window / timer record を更新してはいけない。

ただし、この checkpoint はまだ NEPLg2 program から `DrawCommand` stream や tile / bitmap / row / RLE payload を JS / native host へ直接 export する全体正式 ABI ではない。現在の Web example は legacy stdout protocol で command frame を出す path を残すが、これは正式 backend へ到達できない時の代替実行経路ではない。Web video memory surface については `nepl_gui_web` host import で create / write / discard / publish / present できる初期経路を持つ。`CanvasRenderingContext2D`、DOM element、stdout transport、SharedArrayBuffer queue、minifb は backend implementation detail であり、`core/gui`、`alloc/gui`、`std/gui` の public type には入れない。HD example の現状は high-resolution logical surface の raster 部分を legacy transport の `rgba-row` payload で描く段階であり、1280x720 の全 pixel を個別 command として stdout へ流す契約ではない。次の段階で NEPL/Wasm が生成した command stream から formal host import ABI を呼び、native 側も正式 `GuiHost.present` へ寄せる。Web 側 input は action event、pointer down / move / up / cancel、keyboard down / up、single-scalar text input、host-frame window resized、timer tick を full event queue と `GuiEvent` wrapper へ接続済みである。close button と terminal stop は現 checkpoint では host lifecycle cleanup として実装済みであり、拒否可能 close request は formal host ABI 後に扱う。残件は IME composition / multi-scalar text、window focus / unfocus の発火 policy、rejectable close request、lifecycle variant の poll ABI、run/session/window/timer id の正式化、Mandelbrot progressive / formal tiled rendering、Life の任意解像度 board storage、Paint の persistent stroke / bitmap storage、DrawCommand / tile presentation の formal host import ABI である。

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

契約と現状実装は分けて扱う。2026-06-02 時点の checkpoint は次である。

| Layer | Contract | Current implementation |
|---|---|---|
| `core/gui/geometry` | `GuiPoint`、`GuiSize`、`GuiRect`、`GuiInsets`、`GuiScaleFactor` | constructor / accessor / basic arithmetic を実装済み |
| `core/gui/color` | `BinaryColor`、`Gray8`、`Rgb565`、`Rgb888`、`Rgba8888`、`GuiColor` | constructor / accessor を実装済み |
| `core/gui/event` | `GuiEvent`、`ActionId`、`WidgetId`、pointer / keyboard / lifecycle data | initial data contract を実装済み |
| `core/gui/error` | enum-based GUI/TUI error data | `Unsupported`、`InvalidGeometry`、`ResourceExhausted`、`InvalidCommand` を実装済み。`ResourceMissing`、`TextMeasureFailed`、`FontError`、`SurfaceUnavailable` など formal font / mobile lifecycle extension error は未実装 |
| `core/gui/text_measure` | `TextMeasurer` contract、request/result、font id | borrowed `&Self` contract と fixed-cell `MockTextMeasurer` を実装済み。host wrapper は legacy / terminal / mock 境界として `std/gui` / platform 側で継続し、formal GUI font measurement は `GuiFontFace` based engine へ移す |
| `core/gui/draw_target` | `DrawTarget`、`FlushTarget`、pixel-level drawing | `MockDrawTarget` と O(1) contract test を実装済み。iterator stream と rasterizer は未実装 |
| `core/gui/render_target` | streaming `RenderTarget` | `MockRenderTarget` を実装済み。command list / typed rasterizer は未実装 |
| `core/gui/dirty_region` | no_alloc `DirtyRegion`、checked rect constructor、O(1) merge | `Empty` / `Rect` / `Full` と bounding rect merge を実装済み |
| `core/gui/dirty_region_set` | no_alloc fixed-capacity rect set | 最大 2 rect の `DirtyRegionSet`、overflow to Full、checked push を実装済み。generic capacity と backend damage compression は未実装 |
| `alloc/gui/app` | callback-free app model、`ViewNode`、`GuiEffect`、`Update` | leaf view、button config、redraw/title effect、bounded `GuiEffectBatch` を実装済み。将来の `Vec GuiEffect` へ置換する境界は `Update.effects` に固定 |
| `alloc/gui/layout` | `LayoutContext`、constraints、text measurement injection、measure/place result、arena layout connector、stack layout policy | `TextMeasurer` 注入、constraint validation、fixed text measure、place-at helper、`ViewTreeArena` を `LayoutTreeArena` へ変換する arena order の縦積み connector、parent-local sibling offset を使う vertical / horizontal stack layout policy を実装済み。stack は `StackLayoutPolicy` の axis / spacing / cross-axis alignment / overflow policy を `Result` で扱い、`Allow` は現状互換の配置、`Reject` は parent bounds 外の配置を `GuiError::InvalidGeometry` とする。途中失敗時の `LayoutTreeArena` owner は内部で解放する。flex / grid / scroll、text buffer と arena node の対応付けは未実装 |
| `alloc/gui/widget` | callback-free widget descriptor、action event、semantic lowering、measure bridge | button / label descriptor、`ActionId` event 生成、semantic node 生成、layout measure bridge、focusable accessor を実装済み |
| `alloc/gui/tree` | retained `ViewTree` / `LayoutTree`、allocator-backed arena、focus target query | root + 2 child の bounded tree、capacity error、first focusable id、focusable count に加え、parent index / depth を持つ `ViewTreeArena` / `LayoutTreeArena`、owner-recovery 付き arena child insertion、arena focus count / first focusable query を実装済み。arena を使った pointer routing、diff / invalidation、初期 linear layout connector、stack layout policy は接続済み |
| `alloc/gui/focus` | platform 非依存 focus order / next / previous traversal | bounded `ViewTree` から `FocusOrder` を作り、allocator-backed `ViewTreeArena` は unbounded `FocusOrder` へ落とさず直接走査して、current id から next / previous focus target を `Option WidgetId` で返す実装を追加済み。wrap policy と route command integration は未実装 |
| `alloc/gui/routing` | `LayoutTree` hit test、`WidgetId` lookup、widget action lowering、focus command routing | bounded root + 2 child の pointer action routing、`LayoutTreeArena` の末尾優先 hit test、`ViewTreeArena` の `WidgetId` lookup、arena pointer action lowering、`FocusRouteCommand` / `FocusRouteResult` を実装済み。pointer capture、gesture、stateful pointer routing は未実装 |
| `alloc/gui/diff` | retained tree diff / invalidation data contract | bounded `ViewTree` の slot diff、allocator-backed `ViewTreeArenaDiff`、`GuiInvalidation::Clean` / `Widget` / `Tree` への畳み込みを実装済み。terminal line diff、DOM patch、dirty rect compression は platform 側で未実装 |
| `alloc/gui/text` | platform 非依存 `TextBuffer` / checked edit storage / measured text layout data | `TextBufferId`、`TextBuffer`、checked insert / replace / delete を `Result TextBuffer GuiError` で実装済み。`TextLayout` は injected `TextMeasurer` だけを使って測定し、byte length、char count、fallback cell count、width / height / baseline、max width を保持する。`CachedTextLayout` は buffer id、run id、font id、max width、byte length、char count から deterministic cache key を作る。line break、text hash / revision based invalidation、complex shaping cache は未実装 |
| `alloc/gui/theme` | typed theme scheme / color role / metric role、fallible color/metric helper | `GuiColor` palette、`ThemeMetrics` validation、`Option FontId`、text-cell style helper を実装済み。full typography / component style は未実装 |
| `alloc/gui/accessibility` | semantic node / role / state / action tree | bounded semantic tree の初期 slice を実装済み。host accessibility bridge は `std/gui` / platform 側で継続 |
| `std/gui` | host/runtime/window/timer/text/IME/accessibility/error display/keymap contract | typed data contract、legacy / mock / terminal 用 core `TextMeasurer` host wrapper、`GuiEffectBatch -> GuiRuntimeCommandBatch` 解釈、capability unsupported error、`FocusKeyMap` による Tab / Shift+Tab / Enter / Space から `FocusRouteCommand` への変換、std navigation key code と modifier bit accessor を実装済み。formal GUI text measurement は `GuiFontFace` based engine へ移す。platform 実行は未実装。raw input normalization は `platforms/gui/*` 側で継続 |
| `platforms/gui/terminal` | terminal as `SurfaceKind::TextGrid` backend and terminal input normalization | `TerminalProfile` と core `TextCellRun` based frame、1 byte ASCII subset、`ESC [ Z`、`ESC [ A/B/C/D/H/F`、`ESC [ 1/3/4 ~`、`ESC [ 1 ; <modifier> A/B/C/D` から `TerminalInputEvents` への正規化を実装済み。custom capability と grid size は `Result` で検証し、TextGrid 以外や負 size を拒否する。ANSI / TTY present、Function key などの追加 CSI sequence、途中入力 buffering は未実装 |
| `web/src/gui-preview` | Web Playground display smoke backend | editor panel layout の上に floating GUI window layer を描画し、NEPL legacy stdout protocol で出力された Counter / Mandelbrot / Life / calculator / scientific calculator / paint / breakout frame と host-decoded frame を typed command DTO 経由で表示する。Web checkpoint の bitmap video memory path は `SharedArrayBuffer`、`ImageData`、`putImageData` only presenter で表示し、`nepl_gui_web` video memory host import で create / write / discard / publish / present できる。DrawCommand stream と tile / bitmap / RLE payload の formal presentation ABI は引き続き残件として扱う。old `renderer.ts` / `gui-preview` pane / TS example simulation は削除済みであり、window manager は host-frame source / move / window mode / dock 状態を union で表す。`commands.ts` は Canvas / DOM 型、`null | undefined`、optional metric field を持たず、`fill-rect` / `rgba-row` / `text-run` command union を持つ。`panel.ts` は `none` / `command-frame` / `video-memory` の state を分け、command-frame では `renderGuiPreviewFrameToCanvas`、video-memory では `presentNewestGuiVideoMemoryFrameToCanvas` だけを呼ぶ。同じ `SharedArrayBuffer` identity の opened surface は再利用し、surface size と drawable size が違っても CSS scale、Canvas transform、`drawImage` による伸縮はしない。`host-bridge.ts` は unknown input を `GuiWebHostResult` で decode し、Canvas / DOM 型や throw に依存しない。`runtime-bridge.ts` は global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame` streaming path、`presentVideoMemory`、`closeWindow`、typed presenter registration、typed frame state error、typed video memory frame error、`takeInputEvents` / `resetInputEvents` を持ち、DOM / Canvas 型に依存しない。`presentVideoMemory` は `SharedArrayBuffer` だけを受け、`ArrayBuffer`、typed array、numeric id、string handle、transferable object を拒否する。stdout protocol や command frame path への自動迂回は持たない。`stdout-protocol.ts` は NEPL 実行 stdout fd=1 の line protocol を typed frame と animation timer request に変換し、`NEPLG2_GUI_RGBA_ROW` を row payload command、`NEPLG2_GUI_ACTION_RECT` を input target として扱い、frame 内 parse error では partial frame を破棄する。`input-bridge.ts` は action / pointer / keyboard / text-input / window / timer event を typed queue に保持し、`shared-event-queue.ts` は high-frequency pointer move / window state / adjacent timer tick を coalesce し、容量到達時は古い unread record を押し出して producer に overflow error を返さない。Shell は active-run window id で filter し、stdout timer request を active timer として管理する。Counter は legacy action projection path を維持し、Mandelbrot / Life / calculator / scientific calculator / paint / breakout は `gui_web_wait_event_result` で full event queue を読み、NEPL 側 update loop を継続する。Mandelbrot の HD / Detail mode は 1280x720 logical frame の raster 部分を legacy `rgba-row` payload として描画し、Life の HD mode は現 checkpoint では bounded sample rectangle stream を使う。Paint は pointer event を model に反映し、Breakout は `GuiEvent::Timer` で animation を進める。terminal stop / process finish は `closeWindow` で host-frame window を削除し、window close button は active worker を interrupt し、active timer は run lifecycle とともに停止する。`platforms/gui/web/input.nepl` は full event poll を `Result Option GuiWebEvent GuiError` として公開し、action、pointer down / move / up / cancel、keyboard down / up、single-scalar text input、host-frame window resized、timer tick を `GuiEvent` へ正規化する。formal host import ABI の代替経路としては扱わず、presentation formal ABI、tile / bitmap / RLE payload、IME composition、window focus / unfocus の発火 policy、rejectable close request、lifecycle event は未実装 |

この表にない Web / native / mobile / embedded backend、flex / grid / scroll layout policy、text buffer と arena node の対応付け、stateful pointer capture / gesture、Web / native / mobile raw keyboard normalization、terminal の Function key などの追加 ANSI / CSI sequence と途中入力 buffering、text line break / text hash based cache invalidation、resource loading、real host presentation、formal host import ABI 上の tile / bitmap / row / RLE transport、persistent paint canvas、arbitrary-size Life board は未実装である。

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
