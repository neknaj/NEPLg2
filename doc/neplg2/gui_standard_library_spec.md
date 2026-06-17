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

`DirtyRegionSet` は embedded / framebuffer 向けの fixed-capacity no_alloc rect set contract である。現 checkpoint は最大 2 個の `GuiRect` を保持し、3 個目の追加は silent no-op や panic ではなく `Full` 状態への昇格として表す。負の width / height は `GuiError::InvalidGeometry` として拒否し、x / y の負値は相対座標として許容する。zero-size rect は有効な `GuiRect` として扱い、必要なら backend 側が present 時に無視できる。`dirty_regions_push_region_checked` は `DirtyRegion` を pre-transport aggregation として取り込み、`Empty` は既存 set を返し、`Full` は `dirty_regions_full` を返し、`Rect` は `dirty_regions_push_checked` を経由して invalid rect を拒否する。これは fallback ではなく、`DirtyRegion` の明示状態を fixed-capacity set へ写す checked boundary である。DOM patch、terminal line diff、GPU surface damage compression、tile / bitmap transport は standard API の semantic diff ではなく backend / later transport detail とする。

`alloc/gui/render2d/dirty_surface` の `GuiRgba8888SoftwareSurfaceDirtyOwner` は、RGBA8888 software surface owner と `DirtyRegionSet` を同じ surface + dirty owner boundary に束ねる。dirty の更新は `dirty_regions_push_region_checked` を surface move より前に通し、失敗時は owner-bearing error で元 owner を返す。公開 API は shape / dirty の Copy metadata、`finish_surface`、free に限定し、raw surface accessor、mutable accessor、split accessor は出さない。`finish_surface` は dirty metadata を捨てる recovery / teardown API であり、transport / present / fallback ではない。

`alloc/gui/render2d/bitmap_frame` の `GuiRgba8888BitmapFrameOwner` は、dirty surface owner を formal transport 前に validated bitmap frame owner へ変換する。`frame_id > 0`、surface width / height / stride / byte_len は `gui_rgba8888_software_surface_shape` で再検証した expected metadata と一致し、dirty rect は x/y、width/height、right/bottom overflow、surface containment を通過する必要がある。失敗は `GuiRgba8888BitmapFramePrepareErrorKind` と owner-bearing `GuiRgba8888BitmapFramePrepareError` で返し、代表 kind として `SurfaceStrideMismatch`、`SurfaceByteLengthMismatch`、`DirtyRectOutOfBounds` を持つ。`finish_surface` は全 validation 成功後だけ surface owner を move する recovery / teardown boundary であり、host present、video memory host call、row byte copy、tile list、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_plan` の `GuiRgba8888RowBatchPlanOwner` は、validated bitmap frame owner を formal byte payload / host present 前の row batch plan owner へ変換する。通常 application code の owner aggregate 直 constructor は compiler が拒否するが、compiler memory boundary や trusted producer から forged metadata が来てもよい前提で、`max_rows_per_batch > 0`、`frame_id > 0`、frame width / height / stride / byte_len の shape 再検証、dirty rect origin / size / right-bottom overflow / surface containment を通過した後で、dirty state を contiguous row span と batch count に畳む。`GuiRgba8888RowBatchPlanPrepareErrorKind` は `MaxRowsPerBatchInvalid`、`FrameStrideMismatch`、`DirtyRectBottomOverflow`、`DirtyRectOutOfBounds` などを持ち、prepare error は bitmap frame owner を保持する。`Empty` dirty は row_count 0 の clean-frame plan であり fallback ではない。`Two` dirty は 2 rect を覆う contiguous row span として扱い、tile list や row byte copy は作らない。`finish_frame` は row plan metadata を捨てて bitmap frame owner を返す recovery / teardown boundary であり、`finish_surface`、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_cursor` の `GuiRgba8888RowBatchCursorOwner` は、row batch cursor owner として row batch plan owner を scheduler が 1 batch ずつ進められる descriptor stream へ変換する。`gui_rgba8888_row_batch_cursor_start` は full plan invariant を再検証し、失敗時は `GuiRgba8888RowBatchCursorErrorKind::PlanInvariant` に lower `GuiRgba8888RowBatchPlanInvariantErrorKind` を保持する。`GuiRgba8888RowBatchCursorStatus` は `Ready` / `Complete` の Copy enum であり、`batch_index == batch_count` だけが complete、負値や past-end index は typed error である。`gui_rgba8888_row_batch_cursor_next_batch` は `Ready` cursor から `GuiRgba8888RowBatchDescriptor` と continuation cursor owner を返し、descriptor は frame_id / batch_index / row_start / row_count / width / height / stride_bytes / byte_len の metadata のみを持つ。caller は descriptor を読んだあと `gui_rgba8888_row_batch_cursor_batch_finish_cursor` で continuation cursor owner を取り出す。complete 用 owner terminal、drain / budget、row byte payload、tile list、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_drain` の row batch scheduler drain は、row batch cursor owner を scheduler budget 内で進めた結果を表す progress-only terminal である。`GuiRgba8888RowBatchDrainTerminal` は owner-bearing struct で、Copy enum `GuiRgba8888RowBatchDrainStatus`、continuation cursor owner、`emitted_count` を持つ。status は budget より先に判定され、complete cursor は負 budget でも `Completed` になる。ready cursor で `remaining_steps == 0` は `StepBudgetExhausted`、`remaining_steps < 0` は owner-bearing `InvalidBudget` error である。positive budget では `next_batch` 後に descriptor batch index と continuation cursor index の progress invariant を検査し、`ProgressInvariantInvalid` で止める。count 加算も checked にし、overflow は `EmittedCountOverflow` で返す。`emitted_count` は進めた batch descriptor 数であり、row payload や transport emission ではない。row bytes、tile / RLE、`Vec` collection、surface finish、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_range` の row batch range metadata boundary は、`GuiRgba8888RowBatchCursorBatchOwner` から `GuiRgba8888RowBatchRangeOwner` を作る。`GuiRgba8888RowBatchRangeOwner` は元の batch owner と Copy metadata `GuiRgba8888RowBatchRange` を保持し、range は frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_len、`start_byte_offset`、`byte_count` を持つ。prepare はまず cursor 側の `gui_rgba8888_row_batch_cursor_batch_validate_descriptor_authority` を呼び、embedded plan invariant を再検査したうえで正規 descriptor と一致するかを確認する。authority failure は `BatchAuthorityInvalid %GuiRgba8888RowBatchCursorErrorKind` として lower kind を保持し、continuation cursor status の failure は別に `ContinuationCursorInvalid %GuiRgba8888RowBatchCursorErrorKind` として保持する。range arithmetic は `width * 4 == stride_bytes`、`height * stride_bytes == byte_len`、`row_start + row_count <= height`、`start_byte_offset + byte_count <= byte_len` を checked に検査する。byte storage 前の borrowed revalidation は stored range metadata を再計算 range と比較し、不一致なら `RangeMetadataMismatch` を返す。検査中に batch owner を消費せず、success owner の finish / free だけが continuation cursor へ戻る。row byte storage、tile / RLE、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_byte_storage` の row byte storage boundary は、`GuiRgba8888RowBatchRangeOwner` を `GuiRgba8888RowByteStorageOwner` へ変換する。`GuiRgba8888RowByteStorageOwner` は continuation cursor、`GuiRgba8888RowBatchRange`、exact `byte_count` の copied byte storage を所有する。source storage は private sealed copy helper だけが借用し、public API は source `RegionToken`、`MemPtr`、raw storage accessor を返さない。prepare は `gui_rgba8888_row_batch_range_owner_validate_authority` で range owner authority を再検証してから allocation / copy に進み、copy が完全に成功するまで range owner を消費しない。copy error は source offset overflow、destination index invalid、projection、load、store に分け、copy failure 後の scratch cleanup 失敗は `ScratchDeallocFailed` として original copy error と分ける。read helper は copied destination byte の checked reader であり、source surface escape ではない。この layer は no tile / RLE / host present で止まり、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_plan` の row tile plan metadata boundary は、`GuiRgba8888RowByteStorageOwner` を `GuiRgba8888RowTilePlanOwner` へ変換する。`GuiRgba8888RowTilePlanOwner` は exact copied byte storage owner と Copy metadata `GuiRgba8888RowTilePlan` を保持し、`GuiRgba8888RowTilePlan` は frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_count、tile_rows、tile_count を持つ。prepare は `gui_rgba8888_row_byte_storage_validate_authority` で continuation cursor の `batch_index - 1` から expected range を再計算し、stored range と一致することを借用で再検証する。`tile_count` は quotient / remainder による checked ceil で計算し、overflow しやすい加算式は使わない。`descriptor_at` は `&GuiRgba8888RowTilePlanOwner` を借用し、`gui_rgba8888_row_tile_plan_validate_invariants` で storage authority、range metadata、`stride_bytes == width * 4`、`row_start + row_count <= height`、`byte_count == row_count * stride_bytes`、`tile_count == ceil(row_count / tile_rows)` を再検証してから descriptor を返す。`GuiRgba8888RowTileDescriptor` の `row_start` は frame-absolute row、`byte_offset` は copied row byte storage 内の storage-relative byte offset である。この layer は no RLE / host present で止まり、byte payload split、RLE encode、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_payload` の row tile payload view boundary は、`GuiRgba8888RowTilePlanOwner` と tile index を `GuiRgba8888RowTilePayloadOwner` へ変換する。これは owned payload buffer ではなく、existing copied row storage 上に `GuiRgba8888RowTileDescriptor` を重ねた tile-scoped byte payload view である。prepare は `gui_rgba8888_row_tile_plan_descriptor_at &plan tile_index` を通して descriptor authority と bounds を再検証し、失敗時は `DescriptorInvalid` と original tile plan owner を owner-bearing error に保持する。`byte_at` は tile-relative index を `0 <= index < descriptor.byte_count` で検査し、`descriptor.byte_offset + index` を checked add で storage-relative index へ変換してから `gui_rgba8888_row_byte_storage_byte_at` を呼ぶ。lower storage read failure は `StorageReadFailed` に包む。`gui_rgba8888_row_tile_plan_storage_ref` は raw `RegionToken` / `MemPtr` を返さず、typed borrowed authority として `&GuiRgba8888RowByteStorageOwner` だけを返す。この layer は no RLE / host present で止まり、追加 allocation、追加 copy、RLE encode、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_rle` の row tile RLE cursor boundary は、`GuiRgba8888RowTilePayloadOwner` を `GuiRgba8888RowTileRleCursorOwner` へ変換し、tile 内の RGBA8888 pixel run を streaming で返す。`GuiRgba8888RowTileRleRun` は `pixel_offset`、`pixel_count`、`Rgba8888 color` を持つ Copy metadata であり、cursor / step / step error は payload owner または continuation cursor owner を保持する owner-bearing value なので Clone / Copy を実装しない。`cursor_start` は payload byte count が正で 4 byte RGBA8888 pixel に整列していることを検査し、失敗時は payload owner を start error に保持する。`cursor_status` は `Ready` / `Complete` の Copy enum であり、負 index と past-end index は typed error である。`cursor_next_run` は complete cursor に対して silent no-op を返さず、`CursorComplete` owner-bearing error を返す。pixel read は `pixel_index * 4` と channel offsets `+1` / `+2` / `+3` を checked arithmetic で検査し、payload read failure は `PayloadReadFailed %GuiRgba8888RowTilePayloadReadErrorKind` に包む。この layer は streaming-only であり、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_rle_drain` の row tile RLE drain boundary は、`GuiRgba8888RowTileRleCursorOwner` を scheduler budget 内で進め、`GuiRgba8888RowTileRleDrainTerminal` または `GuiRgba8888RowTileRleDrainError` を返す。`GuiRgba8888RowTileRleDrainTerminal` は `status`、continuation cursor、`emitted_run_count` を保持する owner-bearing value であり、Clone / Copy を実装しない。`status` は Copy enum の `Completed` または `StepBudgetExhausted` である。drain は complete status を budget より先に判定し、complete cursor は負 budget でも `Completed`、Ready cursor の負 budget は `InvalidBudget` owner-bearing error、Ready cursor の zero budget は step を実行せず `StepBudgetExhausted` になる。positive budget では discard する run metadata の `pixel_offset` と `pixel_count` を continuation cursor の `next_pixel_index` と照合し、`emitted_run_count` を checked arithmetic で増やす。この layer は encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_rle_count` の row tile RLE count boundary は、F5cd の drain が返す slice-local `emitted_run_count` を future encoded RLE transport の exact capacity evidence として累積する。`GuiRgba8888RowTileRleCountOwner` は `GuiRgba8888RowTileRleCursorOwner` と `accumulated_run_count` を保持し、`gui_rgba8888_row_tile_rle_count_step_budget` は lower `gui_rgba8888_row_tile_rle_drain_budget` だけに委譲する。`GuiRgba8888RowTileRleCountStepStatus` は `Pending` / `Completed` の Copy enum であり、`GuiRgba8888RowTileRleCountOwner`、`GuiRgba8888RowTileRleCountStep`、`GuiRgba8888RowTileRleCountError` は owner-bearing value なので Clone / Copy を実装しない。`count_start` は Ready cursor だけを受け入れ、Complete cursor は過去に消費された run count evidence を持たないため `InitialCursorComplete` で拒否する。lower drain error は `DrainFailed %GuiRgba8888RowTileRleDrainErrorKind` に包み、`accumulated_run_count + emitted_run_count` の overflow は `AccumulatedRunCountOverflow` とする。overflow では cursor が既に進んでいる可能性があるため fake continuation owner を返さず、error に recoverable cursor と prior `accumulated_run_count` を保持する。この layer は encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_count_completed` の row tile RLE completed count boundary は、F5ce の count owner を formal transport allocation 前の completed evidence へ昇格する。`GuiRgba8888RowTileRleCountCompletedOwner` は `GuiRgba8888RowTileRleCountOwner` と `total_run_count` を保持し、`gui_rgba8888_row_tile_rle_count_completed_prepare` は cursor status を先に検査する。status error は `CursorInvalid %GuiRgba8888RowTileRleStepErrorKind`、Ready cursor は `CountNotCompleted`、Complete cursor で total run count が 0 以下なら `TotalRunCountInvalid` を返す。error は original count owner を保持し、caller が recover / free を選ぶ。completed module は count owner 内部に直接依存せず、`gui_rgba8888_row_tile_rle_count_owner_cursor_status` などの borrowed helper を通す。この layer は RLE 再走査、drain、payload byte read、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_encode_seed` の row tile RLE encode seed boundary は、F5cf の completed count evidence を formal encoded transport 前の payload seed へ変換する。`GuiRgba8888RowTileRleEncodeSeedOwner` は `GuiRgba8888RowTilePayloadOwner` と exact `total_run_count` を保持し、`gui_rgba8888_row_tile_rle_encode_seed_prepare` は total run count が 0 以下なら `TotalRunCountInvalid` として original completed owner を保持する owner-bearing error を返す。成功時は `completed -> count -> cursor -> payload` の順に owner を閉じ、payload seed と total run count だけを残す。この layer は cursor restart、RLE 再走査、drain、payload byte read、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。cursor restart error、encoded buffer allocation、tile transport ABI は後続 phase の owner boundary として定義する。

`alloc/gui/render2d/row_tile_rle_encode_cursor` の row tile RLE encode cursor boundary は、F5cg の payload seed を formal encoded writer 前の ready cursor owner へ変換する。`GuiRgba8888RowTileRleEncodeCursorOwner` は `GuiRgba8888RowTileRleCursorOwner` と exact `total_run_count` を保持し、`gui_rgba8888_row_tile_rle_encode_cursor_start` は seed の total count を読んでから payload owner を finish し、`gui_rgba8888_row_tile_rle_cursor_start` を 1 回だけ呼ぶ。F5cg で total count は検査済みなので、この boundary は invalid count branch を追加しない。restart failure は `CursorStartFailed %GuiRgba8888RowTileRleStartErrorKind` とし、lower `GuiRgba8888RowTileRleStartError` と total count を owner-bearing error に保持する。`cursor_start` は正で RGBA8888 に整列した payload を next pixel index 0 の cursor にするため、成功結果を ready cursor として扱い、この layer では `cursor_status` を呼ばない。この layer は RLE 再走査、drain、`cursor_next_run`、payload byte read、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_writer_plan` の row tile RLE writer plan boundary は、F5ch の ready cursor owner を formal encoded writer 前の capacity plan owner へ変換する。`GuiRgba8888RowTileRleWriterPlanOwner` は `GuiRgba8888RowTileRleCursorOwner`、exact `total_run_count`、exact `encoded_byte_count` を保持する。encoded RLE transport の 1 run は `pixel_offset i32`、`pixel_count i32`、`Rgba8888` 4 bytes の固定 12 bytes とし、`encoded_byte_count = total_run_count * 12` を checked multiplication で検査する。capacity boundary として `total_run_count > 0` を再検査し、0 以下は `TotalRunCountInvalid`、overflow は `EncodedByteCountOverflow` として original `GuiRgba8888RowTileRleEncodeCursorOwner` を保持する owner-bearing error を返す。success path だけが ready owner を finish して cursor owner を writer plan owner へ移す。この layer は cursor status 再検査、RLE 再走査、drain、`cursor_next_run`、payload byte read、encoded buffer allocation、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_storage` の row tile RLE encoded storage boundary は、F5ci の writer plan owner を future encoded writer 用の owned byte storage へ変換する。`GuiRgba8888RowTileRleStorageOwner` は `GuiRgba8888RowTileRleCursorOwner`、exact `total_run_count`、exact `encoded_byte_count`、`RegionToken u8` storage を保持する。F5cj の `storage_prepare` は allocation / reservation only boundary であり、RLE run writer ではない。`GuiRgba8888RowTileRleStoragePrepareErrorKind` は `EncodedByteCountInvalid`、`TotalRunCountInvalid`、`EncodedByteCountOverflow`、`EncodedByteCountMismatch`、`AllocationFailed` を持ち、prepare の全 failure path は original `GuiRgba8888RowTileRleWriterPlanOwner` を保持する owner-bearing error を返す。prepare は encoded byte count 正値、total run count 正値、`total_run_count * 12` の checked recompute、stored byte count との一致、exact byte allocation の順に進み、allocation が成功してからだけ writer plan owner を finish する。この prepare layer は `cursor_next_run`、drain、payload byte read、encoded byte write、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

F5ck の run writer cursor boundary は、`GuiRgba8888RowTileRleStorageOwner` を `GuiRgba8888RowTileRleWriteCursorOwner` へ変換し、1 step ごとに固定 12 byte record を書く。record は `pixel_offset i32 little-endian`、`pixel_count i32 little-endian`、`Rgba8888 r,g,b,a` である。writer は consuming `cursor_next_run` を使わず、borrowed `gui_rgba8888_row_tile_rle_cursor_peek_run` で run metadata を得て、12 byte write がすべて成功してから consuming `gui_rgba8888_row_tile_rle_cursor_advance_by_run` で cursor を進める。store / projection / advance failure は owner-bearing error で unchanged `written_run_count` / `written_byte_count` を返す。partial bytes が target slot に存在しても、その slot は uncommitted であり public reader はない。completion は `written_run_count == total_run_count` と lower cursor `Complete` の両方を要求し、ready cursor なら invariant error とする。この writer boundary は payload byte reader、encoded byte reader、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_encoded` の row tile RLE sealed encoded owner boundary は、F5ck の writer cursor が complete した storage だけを formal tile / bitmap transport 前の sealed owner として扱う。`GuiRgba8888RowTileRleEncodedOwner` は lower cursor、total run count、encoded byte count、private storage を保持する。seal は encoded byte count、total run count、`total_run_count * 12`、written run count range、written byte count range、`written_run_count * 12 == written_byte_count`、`written_run_count == total_run_count`、`written_byte_count == encoded_byte_count`、lower cursor `Complete` の順に検査する。未完了 writer は `WriterNotComplete`、lower cursor がまだ `Ready` なら `CursorNotComplete` で拒否し、すべての failure は original write cursor owner を保持する。sealed owner は byte reader、storage pointer accessor、host present、video memory host call、platform API、fallback を提供しない。

`alloc/gui/render2d/row_tile_rle_packet` の row tile RLE packet owner boundary は、F5cl の sealed encoded owner を formal tile / bitmap transport 前の validated descriptor owner へ変換する。`GuiRgba8888RowTileRlePacketOwner` は `GuiRgba8888RowTileRleEncodedOwner` と `GuiRgba8888RowTileRlePacketDescriptor` を同じ owner boundary に束ねる。descriptor は frame id、batch index、tile index、plan row start、plan row count、tile row start、tile row count、width、height、stride bytes、tile rows、tile count、pixel count、total run count、encoded byte count を持つ Copy metadata である。`plan_row_count` は後続の std layer row tile RLE present-frame owner が tile count を再導出するための authority であり、tile 自身の `row_count` と混同してはならない。prepare は encoded count、`total_run_count * 12`、cursor completion、payload descriptor authority、`pixel_count * 4 == descriptor_byte_count`、`width * 4 == stride_bytes`、`row_count * stride_bytes == descriptor_byte_count`、row extent、derived tile count、tile index range を checked arithmetic で検査する。payload descriptor authority failure は `PayloadDescriptorInvalid` として lower authority error を包む。failure は original sealed encoded owner を owner-bearing error に保持し、success path だけが sealed owner を packet owner へ move する。この layer は byte reader、raw storage、host present、video memory host call、platform API、fallback、silent no-op を提供しない。

`std/gui/tile_present` の std layer row tile RLE present-frame owner は、packet owner と `SurfaceId` / `FrameId` を同じ owner boundary に束ねる。`GuiRgba8888RowTileRlePresentDescriptor` は `surface`、`frame`、`packet` descriptor copy を持つ Copy metadata であり、`GuiRgba8888RowTileRlePresentFrameOwner` が actual `GuiRgba8888RowTileRlePacketOwner` を保持する。prepare は `SurfaceId` / `FrameId` raw value、packet frame id と frame id の一致、positive geometry、plan row extent、tile row extent、`width * 4 == stride_bytes`、`plan_row_count` と `tile_rows` から再導出した tile count、tile index range、`row_count * width == pixel_count`、`total_run_count * 12 == encoded_byte_count` を checked arithmetic で再検査する。failure は packet owner を `GuiRgba8888RowTileRlePresentFramePrepareError` に保持し、success path だけが packet owner を present-frame owner へ move する。この phase は `GuiSurfacePresentCommand` を拡張せず、host import、byte reader、raw storage、video memory host call、platform API、fallback、silent no-op を提供しない。Web / native / headless presenter はこの owner を消費する後続 phase で定義する。

`alloc/gui/render2d/row_tile_rle_packet_record` の row tile RLE packet typed record reader は、F5cn の後続で presenter が必要とする最小の typed read boundary である。`GuiRgba8888RowTileRlePacketRecordReadErrorKind` は total run count、encoded byte count、record index、record byte offset、raw projection / load、decoded i32、channel、run extent の失敗を enum として分ける。この module だけが quarantined typed record reader として private `RegionToken u8` を借用し、12 byte record を `GuiRgba8888RowTileRleRun` に戻す。public API は `gui_rgba8888_row_tile_rle_packet_record_at &packet record_index` だけを typed run reader として公開し、raw pointer、byte slice、storage accessor は公開しない。`row_tile_rle_packet` と `row_tile_rle_encoded` の no-reader contract は維持され、例外はこの typed record reader module に閉じる。

`std/gui/tile_present_run_cursor` の std layer row tile RLE present run cursor は、`GuiRgba8888RowTileRlePresentRunCursorOwner` が `GuiRgba8888RowTileRlePresentFrameOwner`、`next_record_index`、`total_run_count` を保持する presenter-neutral owner boundary である。start は present descriptor から `total_run_count * 12 == encoded_byte_count` を再検査し、failure では original present owner を `GuiRgba8888RowTileRlePresentRunCursorStartError` に保持する。step は `record_index == total_run_count` を explicit `Completed` として返し、`record_index > total_run_count` は `RecordIndexPastEnd` owner-bearing error にする。record read failure は `PacketRecordReadFailed %GuiRgba8888RowTileRlePacketRecordReadErrorKind` に包む。この cursor は host import、raw memory、video memory host call、platform API、fallback、silent no-op を提供しない。Web / native / headless の host import はこの cursor を消費する後続 phase とする。

`std/gui/tile_present_command_cursor` の std layer row tile RLE present command cursor は、F5co の run cursor を presenter-facing frame command stream へ昇格する。`GuiRgba8888RowTileRlePresentCommandCursorOwner` は lower run cursor owner、present descriptor copy、phase を保持し、owner-bearing value なので Clone / Copy を実装しない。command は `GuiRgba8888RowTileRlePresentCommand::BeginFrame`、`Run`、`GuiRgba8888RowTileRlePresentCommand::EndFrame` であり、public step は one typed output per public step を守る。`BeginPending` は BeginFrame、`RunPending` の lower `RunReady` は Run、lower `Completed` は同じ public step で EndFrame を返して phase を Completed に進める。Completed phase は terminal Completed を返す。lower start / step failure は F5co error kind と category を包み、present owner または command cursor owner を失わない。この command cursor does not bypass F5co。packet record reader、packet storage、`RegionToken`、`MemPtr`、host import、video memory host call、platform API、fallback、silent no-op には進まない。actual Web / native / bare / headless presenter はこの command stream を消費する後続 phase で定義する。

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

`examples/gui_mandelbrot.nepl`、`examples/gui_life.nepl`、`examples/gui_counter.nepl`、`examples/gui_calculator.nepl`、`examples/gui_scientific_calculator.nepl`、`examples/gui_paint.nepl`、`examples/gui_breakout.nepl` は、NEPL 側で application model、typed event update、render command frame を作る。現 checkpoint では `platforms/gui/web/stdout_protocol.nepl` を通して Web Playground host へ frame stream を出すが、これは formal host surface ABI へ到達する前の legacy smoke transport であり、same app code contract の正式 path ではない。stdout helper は platform backend detail として `GuiWebTextAlign` enum と `Result unit GuiError` を返す checked API を持ち、invalid geometry を panic や silent no-op にしない。text label を持つ button は `GuiWebButtonConfig` と `gui_web_stdout_button` で `fill_rect`、`text_run`、`action_rect` の順序を一箇所に集約し、example 側は app 固有の `ActionId`、label、色、geometry だけを渡す。Mandelbrot は Preview / HD / Detail action で sample grid と logical surface size を切り替え、HD / Detail mode は 1280x720 logical frame の raster 部分を `rgba-row` payload で描画する。さらに `--video-memory-once` は 32x18 の有限 surface を formal Web video memory row host import へ出す opt-in 検査 path であり、stdout protocol、command frame、TS simulation へ fallback しない。`--video-memory-progressive-once` は同じ preview surface を row batch ごとの rect dirty publish で更新する有限検査 path であり、`--video-memory-progressive-test` は CI から同じ implementation を呼ぶ alias である。`--video-memory-progressive-loop-test` は `GuiEvent::Timer` の matching timer id だけで row batch を 1 つ進める有限検査 path である。Life は next step、animate toggle、cell pixel size、HD view を扱う。Counter、四則電卓、関数電卓は button の `ActionId` を update で解釈する。Paint は button action だけでなく full `GuiWebEvent` の pointer position を model update に使う。Breakout は button action と timer tick で model を進める。各 button 領域は `NEPLG2_GUI_ACTION_RECT` で `ActionId` hit target として出力される。Web Playground の Run 経路では、この NEPL stdout frame stream が floating GUI window を開く。TypeScript はこれらの example を simulation せず、stdout frame decode と backend presentation だけを担当する。

Native smoke backend は macOS AppKit、Windows Win32、Linux Wayland / X11 の window lifecycle 調査を踏まえ、OS window manager が与える resize / close / event pump を受ける形へ寄せている。`WindowOptions.resize = true`、`ScaleMode::AspectRatioStretch`、`set_target_fps 60`、current window size 監視、letterbox-aware hit test、`NativeSurfaceState::Unavailable` を使い、固定 size framebuffer 前提の click mapping を避ける。調査内容と native backend contract は `doc/neplg2/gui_native_platform_behavior.md` に分けて記録する。これはまだ正式な `std/gui::GuiHost.present` 実装ではなく、minifb と native handle は標準 API の public type へ出さない。

Web Playground の表示 smoke は editor の panel layout の上に独立した DOM layer を置き、`GuiFloatingWindowManager` が minimize、maximize / restore、drag move、edge / corner resize、dock restore を扱う。これは native window と同等の基本操作を browser 上で検査するための backend UI であり、標準 API の window model ではない。`GuiFloatingWindowManager` の move state、source、window mode、dock state は discriminated union で表す。`minimized` mode は previous mode を保持するため、maximized window を minimize / restore しても original restore rect は失われない。top bar の `GUI` button と editor header の `G` button は user-facing 導線から外し、NEPL execution が stdout protocol を出した時だけ window を開く。host event / queue status は GUI window content に挟まず、折りたたみ式の `GuiWindowDebugPanel` へ分離する。通常の window body は host frame canvas だけを含む。host frame の title は window titlebar の表示責務であり、canvas renderer は同じ title を content 内へ再描画しない。debug panel は通常 window より低い補助 z-layer に置き、collapsed 時は toggle 以外の pointer capture を持たず、`aria-live` を off にして queue 更新を main GUI live region の読み上げ対象にしない。`window-manager.ts` と `panel.ts` が `null` / `undefined` / non-null assertion に頼らないこと、かつ debug/status DOM を window content に戻さないことを source policy regression で固定する。

Host frame の描画 data は `web/src/gui-preview/commands.ts` の `fill-rect` / `rgba-row` / `text-run` command union、`rgba8888` 相当の color struct、command frame、`action-rect` input target で表す。`rgba-row` は legacy smoke transport で HD raster の row payload を bounded command count で運ぶための現 checkpoint の command であり、Canvas や DOM 型を public DTO に入れない。旧 `renderer.ts` による Mandelbrot / Life / Counter の TS scene simulation は削除済みであり、Run 経路の GUI 表示は現 checkpoint では NEPL stdout protocol によって駆動する。`panel.ts` は host-frame surface として、NEPL 実行が出した command frame だけを描画する。`host-bridge.ts` は unknown input を `GuiWebHostResult` の `ok` / `err` union で decode し、invalid frame、invalid command、invalid rect、invalid color、invalid text、invalid input target、unsupported command を typed error として返す。`runtime-bridge.ts` は presenter missing、invalid install target、invalid frame state、host decode error、invalid video memory frame、video memory open / present failure を `GuiWebRuntimeResult` で返し、global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame` streaming path、`presentVideoMemory`、`closeWindow` を floating window presenter へ接続する。`presentVideoMemory` は `windowId`、`title`、`SharedArrayBuffer` を持つ video memory frame だけを受け付け、`ArrayBuffer`、typed array、numeric id、string handle、transferable object を typed error として拒否する。stdout protocol や command frame path への自動迂回は持たない。`panel.ts` は `none` / `command-frame` / `video-memory` の state を分け、同じ `SharedArrayBuffer` identity の opened surface を再利用する。video memory presentation は `ImageData` と `putImageData` だけで行い、surface size と drawable size が一致しない場合も CSS scale、Canvas transform、`drawImage` による伸縮を行わず、top-left に 1:1 で提示する。window resize は `WindowEventKind::Resized` として application に渡し、application 側が新しい pixel buffer size を決める。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` に `video_memory_create_surface`、`video_memory_acquire_write_slot`、`video_memory_write_slot_bytes`、`video_memory_write_rgba8888_row`、`video_memory_discard_write_slot`、`video_memory_publish_slot`、`video_memory_present_surface`、`video_memory_close_surface` を持つ。`surface_id` と `frame_id` は worker-local opaque positive integer であり、`SharedArrayBuffer`、DOM handle、Canvas handle、ArrayBuffer transfer object、JS object handle、string handle は NEPL/Wasm へ渡さない。`video_memory_present_surface` は typed `gui_video_memory_present` worker message と ack `SharedArrayBuffer` で main thread presenter の実結果を待ってから status を返す。`platforms/gui/web/surface.nepl` は raw negative status を module private helper で `Result` / `GuiError` へ写す。`web/src/gui-preview/stdout-protocol.ts` は stdout fd=1 の line protocol だけを typed command frame と typed animation timer request へ decode し、`NEPLG2_GUI_RGBA_ROW` を `rgba-row` command、`NEPLG2_GUI_ACTION_RECT` を frame-local input target として読む。chunk split、invalid frame state、invalid color、invalid rgba row、invalid action rect、invalid animation timer を discriminated error で表す。frame 内 parse error は partial frame を破棄し、壊れた frame を present しない。Web checkpoint の presentation hot path は video memory surface と `putImageData` only presenter に寄せる。DrawCommand stream / tile / bitmap / row / RLE を直接渡す正式 host import ABI は残件である。`web/src/gui-preview/input-bridge.ts` は DOM / Canvas に依存しない typed queue として `GuiWebInputEvent::action`、`GuiWebInputEvent::pointer`、`GuiWebInputEvent::keyboard`、`GuiWebInputEvent::text-input`、`GuiWebInputEvent::window`、`GuiWebInputEvent::timer` を保持し、listener へも typed event だけを通知する。DOM `KeyboardEvent` の key string は `panel.ts` で std key code contract と Unicode scalar value へ正規化され、application code へ DOM string は渡らない。Space は keyboard event と text input event の両方として queue し、composition 中、Meta shortcut、multi-scalar text は現 checkpoint では queue しない。pointer down / up は DOM `button` から changed button を正規化し、pointer move は DOM `buttons` bitmask から現在押下中の button state を正規化する。これにより paint のような app は hover move と primary drag を `PointerButton` で区別できる。floating host frame window は resize 時に `WindowEventKind::Resized` を worker queue へ渡す。stdout animation timer request は window id と timer id を持ち、Shell が browser timer を管理して `TimerEvent` を worker queue へ渡す。close button は現 checkpoint では拒否可能 close request ではなく host lifecycle signal として扱い、window を削除した後で active worker を interrupt する。terminal stop / process finish は `neplGuiHost.closeWindow` presenter path で host-frame window を削除し、active timer も停止する。`web/src/gui-preview/shared-event-queue.ts` は SharedArrayBuffer の full event queue と legacy action projection queue を分ける。full event queue は action / pointer / keyboard / text input / window / timer の kind、window id、action id、pointer milli-position、pointer kind、pointer id、button、keyboard kind、key code、modifier bit、text scalar value、window kind、window size、timer id、timer tick を worker へ渡す。record slot length は 8 のまま固定し、event kind ごとに payload slot を再利用する。action-only queue は `poll_action_id` / `wait_action_id` 互換 path が pointer / keyboard / text input / window / timer event を consume しないための projection である。queue は bounded だが、producer は `event-queue-full` / `action-queue-full` を返さない。容量に達した場合は古い unread record を明示的に押し出し、新しい input を受け入れる。full event poll と legacy action-only poll を同じ app run で混用すると action projection queue に残る event があるため、互換 path は action-only app 用である。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` の `poll_action_id` / `wait_action_id` に加えて、`poll_event_kind` / `wait_event_kind` と last-event field accessors を提供する。`platforms/gui/web/input.nepl` は raw sentinel を public API に出さず、`gui_web_wait_action_result` で unsupported host を `GuiError::Unsupported`、timeout を `Option::None`、action を `Option::Some ActionId` として正規化する。さらに `gui_web_wait_event_result` / `gui_web_poll_event_result` は `Result Option GuiWebEvent GuiError` を返し、現 checkpoint の action record を `GuiEvent::Action`、pointer down / move / up / cancel record を `GuiEvent::Pointer`、keyboard record を `GuiEvent::Keyboard`、text scalar record を `GuiEvent::TextInput`、window resize record を `GuiEvent::Window`、timer record を `GuiEvent::Timer` として保持する。text scalar は `char_from_i32_result` で検証し、surrogate や範囲外を `GuiError::InvalidCommand` にする。window kind と size も raw value を `WindowEventKind` と `GuiSize` へ正規化し、未知 kind や 0 以下の size は `GuiError::InvalidCommand` にする。timer id と tick は正の id と 0 以上の tick として検証し、不正な record は `GuiError::InvalidCommand` にする。`web/src/terminal/shell.ts` は active run が present した window id だけを queue 対象にし、stale window の input event が別 app に混入しないようにする。空 poll の busy spin を避けるため、interactive app は `wait_action_id` または `wait_event_kind` の Atomics wait path を使う。

`video_memory_discard_write_slot` は未公開 write frame の所有権を `Writing -> Free` に戻すためだけの Web backend import である。成功時は dirty metadata を消し、published / presented epoch は進めない。frame が存在しない、既に publish / discard 済み、または resize generation が古い場合は typed status を返し、stdout protocol、command frame、別 surface への fallback は行わない。

`video_memory_write_rgba8888_row` は formal row payload の最小 writer である。`write_slot_bytes` と違い app は byte offset を渡さず、origin、pixel width、source pointer だけを渡す。`width <= 0`、surface 範囲外、`width * 4` と一致しない source byte length は typed error で拒否し、clamp / truncate / zero-byte no-op は行わない。row write は pixel plane だけを更新し、dirty metadata、slot epoch、published epoch、presented epoch は publish path へ残す。

`examples/gui_video_memory_rows.nepl` は focused NEPL example として、`ByteBuilder` / `ByteBuf` owner で row bytes を構築し、borrowed `MemPtr u8` を `gui_web_video_memory_write_rgba8888_row` へ渡す。これは stdout `rgba-row` を使わない formal row host import の source contract を示すための例である。現行 CI の `run_test.js` は default `nepl_gui_web` video memory host import を unsupported stub として持つため、通常 doctest では host capability missing を `Result` に写す境界を壊さない。この example の happy path は fake positive `nepl_gui_web` host import harness が通常 path の NEPL/Wasm 実行として検査する。Mandelbrot も `--video-memory-once` で有限 preview model の RGBA8888 row payload を同じ host import path へ出す。Mandelbrot の legacy stdout interactive path は `WindowEventKind::Resized` を application update で受け、drawable pixel size から responsive model を作れる。`--video-memory-resize-once` は finite formal video memory resize/recreate checkpoint であり、typed resize event の後に old surface を close して new surface を create / render / present / close する。`--video-memory-loop` は formal video memory surface を保持し、typed event を待ち続ける loop checkpoint である。resize event では old surface close 成功後だけ new surface を create / render / present し、focus / unfocus / non-window event では current surface を維持し、close request では current surface を close して正常終了する。`--video-memory-loop-test` の wait count は CI の停止条件であり scheduler policy ではない。Mandelbrot の progressive video memory path は row batch ごとに dirty rect を publish する finite checkpoint であり、batch end は sample height で clamp する。Timer event driven progressive loop checkpoint は matching timer id の event だけを batch 進行条件にし、timer id 不一致、empty event、focus event では batch を進めない。FHD 60 fps readiness、formal tiled transport、formal timer registration ABI、real scheduler policy は後続 slice である。

pointer move と window state coalescing は順序保存 contract として扱う。`panel.ts` は pointermove を `requestAnimationFrame` 単位で最新の move へまとめ、`input-bridge.ts` の stored queue は直前に保持された同じ window id、pointer id、button の `PointerEventKind::Move` だけを最新座標へ置き換える。`shared-event-queue.ts` は write tail 直前の unread slot が同一 pointer move または同一 window kind の state record である場合だけ最新値へ置換でき、queue 全体の未読 slot は走査しない。timer tick も同じく直前の unread slot が同一 window id / timer id の `TimerEvent` である場合だけ最新 tick へ置換でき、action や pointer / keyboard / text / window event をまたいで更新してはいけない。`Down` / `Up` / `Cancel`、action、keyboard、text input、close lifecycle signal をまたいで古い move / window / timer record を更新してはいけない。

ただし、この checkpoint はまだ NEPLg2 program から `DrawCommand` stream や tile / bitmap / row / RLE payload を JS / native host へ直接 export する全体正式 ABI ではない。現在の Web example は legacy stdout protocol で command frame を出す path を残すが、これは正式 backend へ到達できない時の代替実行経路ではない。Web video memory surface については `nepl_gui_web` host import で create / write / discard / publish / present できる初期経路を持つ。`CanvasRenderingContext2D`、DOM element、stdout transport、SharedArrayBuffer queue、minifb は backend implementation detail であり、`core/gui`、`alloc/gui`、`std/gui` の public type には入れない。HD example の現状は high-resolution logical surface の raster 部分を legacy transport の `rgba-row` payload で描く段階であり、1280x720 の全 pixel を個別 command として stdout へ流す契約ではない。Mandelbrot の legacy stdout interactive path は host-frame resize event を `GuiWebEvent` として読み、1 drawable pixel per sample の model を作れる。Mandelbrot の finite video memory resize path は old surface close と resized surface recreate を検査する。Mandelbrot の formal video memory event loop path は open surface を維持し、typed window resize event で old surface を close して resized surface を recreate する。Mandelbrot の progressive video memory path は row batch ごとの dirty rect publish と timer event driven batch progression を検査する。次の段階で NEPL/Wasm が生成した command stream から formal host import ABI を呼び、native 側も正式 `GuiHost.present` へ寄せる。Web 側 input は action event、pointer down / move / up / cancel、keyboard down / up、single-scalar text input、host-frame window resized、timer tick を full event queue と `GuiEvent` wrapper へ接続済みである。close button と terminal stop は現 checkpoint では host lifecycle cleanup として実装済みであり、拒否可能 close request は formal host ABI 後に扱う。残件は IME composition / multi-scalar text、window focus / unfocus の発火 policy、rejectable close request、lifecycle variant の poll ABI、run/session/window/timer id の正式化、formal timer registration ABI、formal tiled rendering、real scheduler policy、Life の任意解像度 board storage、Paint の persistent stroke / bitmap storage、DrawCommand / tile presentation の formal host import ABI である。

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

std layer row tile RLE present host-command record は F5cq の checkpoint である。`std/gui/tile_present_host_command` は F5cp の public step descriptor accessor と step result accessor だけを使い、`GuiRgba8888RowTileRlePresentHostCommandRecord` と `GuiRgba8888RowTileRlePresentHostCommandStepResult` を作る。record shape は `BeginFrame descriptor`、`RunRecord run_record`、`EndFrame descriptor` であり、run record は descriptor と run を保持し、does not flatten to kind plus optional run。これは enum / match による静的検査で不正状態を表現不能にするためである。この layer does not bypass F5cp。F5co run cursor、packet record reader、raw storage、host import、platform API、fallback には直接触れない。

std layer row tile RLE present run-span boundary は F5df の checkpoint である。`std/gui/tile_present_run_span` は F5cq の `GuiRgba8888RowTileRlePresentHostCommandRunRecord` を消費し、tile-local linear pixel offset を `GuiRgba8888RowTileRlePresentRunRowSpan` の stream へ分解する。row span は x、y、width、color だけを持ち、高さは accessor が常に 1 を返すため、platform rect や renderer rect へ依存しない。start は width、height、row_start、row_count、tile_rows、pixel_count、run offset、run count、run end を checked arithmetic と enum error で検査し、invalid cursor を作らない。step は run を `local_row = offset / width`、`x = offset % width`、`y = row_start + local_row` で 1 行以内に切り、remaining が 0 の場合は explicit Completed を返す。空 span、silent no-op、unsupported host への fallback はない。F5df は does not call platform import。F5da-F5de action / driver、F5cs virtual drain、F5cp / F5co lower cursor、packet record reader、raw storage、queue、scheduler、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback に触れない。

std layer row tile RLE present host import request は F5cr の checkpoint である。`std/gui/tile_present_host_import` は F5cq の `GuiRgba8888RowTileRlePresentHostCommandRecord` だけを消費し、`GuiRgba8888RowTileRlePresentHostImportRequest` に包む。request target は `GuiRgba8888RowTileRlePresentHostImportTarget` の `Window WindowId`、`Offscreen`、`Device` に限定する。Headless is not a presentation target。headless / text grid は `GuiError::Unsupported` とし、fallback target を選ばない。RGBA8888 row tile RLE 専用の境界なので `ColorFormat::FormatRgba8888` 以外の capability も `GuiError::Unsupported` として、この mismatch を platform backend に持ち越さない。

std layer row tile RLE present virtual drain は F5cs の checkpoint である。`std/gui/tile_present_virtual_drain` は headless / test が F5cq host-command record を観測するための境界であり、presentation target ではないため does not consume F5cr。`GuiRgba8888RowTileRlePresentVirtualDrain` は Begin / Run / End の phase、optional surface / frame、expected / seen count を保持し、RunRecord では `run_pixel_offset == seen_pixel_count` を要求する。これにより total count だけでは見逃す gap / overlap / reorder を std layer で拒否できる。error は `GuiRgba8888RowTileRlePresentVirtualDrainErrorKind` と直前 drain state を保持し、silent no-op や fallback presenter へ逃げない。

std layer row tile RLE present schedule boundary は F5ct の checkpoint である。`std/gui/tile_present_schedule` は F5cq host-command record stream を platform host dispatch の前で deterministic slice budget に区切る。`GuiRgba8888RowTileRlePresentScheduleState` は F5cs virtual drain state と slice-local command / pixel counters だけを保持し、stream validation は F5cs virtual drain に委譲する。`Yield means exact slice budget` であり、valid record を消費した後に command budget または pixel budget へちょうど到達したときだけ `Yield` を返す。over-budget is a typed error であり、budget 超過、single RunRecord の pixel budget 超過、checked arithmetic overflow、lower F5cs failure は previous schedule state を持つ error で返す。この layer は queue、timer、F5cr request、host import call、raw packet storage、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present scheduled dispatch boundary は F5cu の checkpoint である。`std/gui/tile_present_dispatch` は F5ct before F5cr の順序で、schedule validation / budget decision の後に host import request value を作る。`GuiRgba8888RowTileRlePresentDispatchState` は `GuiRgba8888RowTileRlePresentScheduleState` だけを持つ。success path は `RequestReady request plus post phase` であり、request と `Continue` / `Yield` / `Completed` の post phase を同じ ready value に入れる。これにより exact-budget record と EndFrame record の request delivery を落とさない。F5ct error と F5cr error は previous dispatch state を返し、F5cr error では同じ step で得た updated schedule state を採用しない。この layer は F5cs direct call、F5cp / F5co cursor、raw packet storage、queue、timer、host import execution、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present dispatch loop outcome boundary は F5cv の checkpoint である。`std/gui/tile_present_dispatch_loop` は F5cu の `RequestReady request plus post phase` を、platform executor の host outcome と接続するための one-shot pending value に包む。`GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` は previous state、next state、request、post phase を同時に保持し、Clone / Copy を持たない。`complete_request consumes pending` ため、同じ host outcome を二重に完了して next state を複数回 publish する replay を型上の所有権境界で避ける。host outcome が Err の場合は previous state を持つ typed error を返し、Ok の場合だけ post phase に従って Continue / Yield / Completed の completion に next state を入れる。この layer は F5cu だけを authority とし、F5ct / F5cr / F5cs の direct call、host import execution、queue、timer、scheduler、raw storage、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host execution action boundary は F5cw の checkpoint である。`std/gui/tile_present_host_execution` は F5cr の `GuiRgba8888RowTileRlePresentHostImportRequest` を `GuiRgba8888RowTileRlePresentHostExecutionAction` に写す。action は flat target x record action であり、Window / Offscreen / Device と BeginFrame / RunRecord / EndFrame の直積を enum variant として持つ。Window variant は `WindowId` と descriptor / run record を payload struct に保持し、Offscreen / Device は variant 名で target を保持する。F5cw は F5cr request accessor と F5cq record / run record shape だけを authority とし、does not execute host imports。actual Web / native / bare executor の失敗はこの action ではなく `Result unit GuiError` として F5cv `complete_request` に戻す。この layer は F5cv / F5cu / F5ct / F5cs direct call、F5cr request constructor、raw storage、host execution API、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation boundary は F5dg の checkpoint である。`std/gui/tile_present_host_span_operation` は F5cw `GuiRgba8888RowTileRlePresentHostExecutionAction` を actual Web / native / bare presenter が 1 operation ずつ消費できる `GuiRgba8888RowTileRlePresentHostSpanOperation` stream に写す。cursor は `GuiRgba8888RowTileRlePresentHostSpanOperationCursor` で、phase は `SinglePending operation`、`RunPending target run_span_cursor`、`Completed` の 3 種だけである。Begin / End action は SinglePending operation として 1 回だけ発行され、次の step は explicit Completed を返す。Run action は start で F5df run-span cursor を 1 回だけ作り、step ごとに F5df `run_span_step` を最大 1 回だけ呼んで WindowRunSpan / OffscreenRunSpan / DeviceRunSpan に target-qualified mapping する。F5df start / step error は action または cursor context を保持する enum error に包む。F5dg は actual host import execution、F5da-F5de action driver、F5cs virtual drain、F5cp / F5co lower cursor、packet record / raw storage、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、queue、scheduler、fallback、silent no-op を提供しない。

std layer row tile RLE present scheduled span operation boundary は F5dh の checkpoint である。`std/gui/tile_present_scheduled_span_operation` は F5dg operation stream を platform presenter へ渡す前に deterministic slice budget で区切る。これは F5ct record scheduler の再利用ではない。F5ct は F5cq record 単位で full RunRecord を 1 cost とするため、F5dg が Run を複数 row span に分解した後の span operation scheduling とは authority が異なる。F5dh の `GuiRgba8888RowTileRlePresentScheduledSpanOperationState` は F5dg cursor と slice-local operation / pixel counters だけを保持し、F5cs / F5ct / F5cu を直接呼ばない。Begin / End は operation cost 1 and pixel cost 0、RunSpan は operation cost 1 and `span.width * span.height` pixel cost である。valid operation を消費した後に exact budget へ到達した場合だけ `Yield` を返し、operation、post phase、next state は `OperationReady` に同居するため exact-budget operation は失われない。over-budget、single span pixel budget 超過、checked arithmetic overflow、lower F5dg failure は typed error として previous state を保持する。`resume_slice` は F5dg cursor を保持して slice counters だけ reset する。この layer は actual host import execution、record scheduler、action driver、raw storage、queue、timer、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation attempt boundary は F5di の checkpoint である。`std/gui/tile_present_host_span_operation_attempt` は F5dh `GuiRgba8888RowTileRlePresentScheduledSpanOperationReady` と actual Web / native / bare / headless presenter が返す caller supplied outcome を対応づける。`GuiRgba8888RowTileRlePresentHostSpanOperationAttempt` は attempted operation と `Result unit GuiError` だけを保持し、std layer は `Result::Ok unit` や synthetic failure を作らない。`attempt_step` は support before equality の順序を守る。support は F5cy `GuiRgba8888RowTileRlePresentHostExecutorSupport` enum を target support set として読むが、F5cy `require_supported` や action equality helper へは戻らない。operation equality は 9 variants すべてで variant と target を比較し、Window variants は `window_id_raw`、Begin / End は descriptor、RunSpan は x / y / width / height / RGBA channel を public accessor で比較する。unsupported target と mismatched attempt は `GuiError::Unsupported` / `GuiError::InvalidCommand` category を持ち、scheduled ready と attempt を失わない typed error として返す。Yield phase is data only であり、F5di は resume、queue、timer、scheduler、actual platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation completion boundary は F5dj の checkpoint である。`std/gui/tile_present_host_span_operation_completion` は F5di の `GuiRgba8888RowTileRlePresentHostSpanOperationAttemptStep` を AttemptStep only の入力として受け、caller supplied outcome と ready phase を completion value へ写す。`GuiRgba8888RowTileRlePresentHostSpanOperationCompletion` は `Continue state` / `Yield state` だけを持つ。F5dh `Completed` は operation を持たない terminal なので、per-operation completion does not create Completed。host outcome failure does not publish state であり、`Err host_error` は `GuiRgba8888RowTileRlePresentHostSpanOperationCompletionHostFailed` に host error、scheduled ready、attempt、category `Some host_error` を保持する。F5dj は F5di の association validation を再実行せず、F5dh step / start / resume、F5cs / F5ct / F5cu、F5cy action validation、F5cw action equality、F5da-F5de action driver、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter step boundary は F5dk の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_step` は support set、F5dh ready、presenter supplied attempt を受け、F5di before F5dj の順序で 1 operation の戻り道を固定する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterStep` は F5dj の `GuiRgba8888RowTileRlePresentHostSpanOperationCompletionStep` を保持する success value である。F5di rejection は `AttemptRejected` として support、ready、attempt、lower F5di error、lower category を保持し、F5dj rejection は `CompletionRejected` として attempt step、lower F5dj error、lower category を保持する。F5dk does not execute host imports し、success / failure outcome を合成せず、actual Web / native / bare / headless presenter が作った attempt だけを扱う。Completed、F5dh start / step / resume、F5dg start / step、F5cw action validation、F5da-F5de action driver、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op は提供しない。

std layer row tile RLE present host span operation presenter loop boundary は F5dl の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_loop` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterLoopState` を LoopState として定義し、support、F5dh policy、scheduled state を同じ value に束ねる。`start` は F5dh start を 1 回だけ呼び、`request` は F5dh step を 1 回だけ呼ぶ。F5dh `OperationReady` は support / policy / ready を持つ presenter request へ写し、F5dh operation-less terminal は loop `Completed` として返す。`complete` は F5dk presenter step を 1 回だけ呼び、F5dk success branch でだけ F5dj completion step を読み、Continue / Yield を support / policy / scheduled state 付き LoopState へ再包装する。F5dl does not execute host imports し、actual presenter attempt を合成せず、F5dh `resume_slice`、F5di / F5dj direct call、F5dg start / step、F5cs / F5ct / F5cu、F5da-F5de action driver、F5cy / F5cw validation、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter outcome boundary は F5dm の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_outcome` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeRequest` を non-Copy request bridge として定義し、F5dl request と F5dh ready operation accessor から得た expected operation を保持する。actual presenter はこの request を borrow して operation を読み、host outcome を得た後で `OutcomeRequest` を value として消費し、F5di attempt constructor へ caller supplied `Result unit GuiError` を 1 回だけ渡して `OutcomeAttempt` を作る。`complete` は `OutcomeAttempt` を value として消費し、F5dl complete を 1 回だけ呼び、lower error は request、attempt、F5dl category accessor 由来 category とともに typed error へ保持する。F5dm does not execute host imports し、F5di validation、F5dk presenter step、F5dj completion step、F5dh start / step / resume、F5dg、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、loop `Completed` creation を提供しない。

std layer row tile RLE present host span operation presenter driver boundary は F5dn の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_driver` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterDriverState` を non-Copy driver state として定義し、actual Web / native / bare / headless presenter loop が F5dl と F5dm を別々に扱わず、driver start / request / complete contract だけを扱えるようにする。`start` は F5dl start を 1 回だけ呼んで F5dl loop state を DriverState へ包む。`request` は DriverState を value として消費して F5dl request を 1 回だけ呼び、F5dl `Request` の場合だけ F5dm outcome request を作る。F5dl terminal `Completed` は driver `Completed` へ写すだけで、F5dm outcome request は作らない。`complete` は `OutcomeRequest` と caller supplied outcome を value として受け、F5dm outcome attempt と F5dm outcome complete を 1 回ずつ呼び、F5dl Continue / Yield を次の DriverState へ再包装する。F5dn does not execute host imports し、F5dl complete direct call、F5di constructor / validation direct call、F5dh start / step / resume direct call、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` creation を提供しない。

std layer row tile RLE present host span operation presenter executor boundary は F5do の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorRequest` を non-Copy request として定義し、F5dn OutcomeRequest、OutcomeRequest 内の F5dl request から読んだ support、expected span operation を同じ value に束ねる。executor が返す `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttempt` は executed span operation と caller supplied outcome を持つ。request creation は OutcomeRequest 由来 support だけで unsupported operation を検査し、unsupported の場合も F5dn complete へ合成 outcome を流さない。complete は request の expected span operation と attempt の reported span operation を payload まで比較し、一致した場合だけ F5dn complete を 1 回呼ぶ。F5do does not execute host imports し、F5dl complete direct call、F5dm outcome attempt / complete direct call、F5di constructor / validation direct call、F5cw action mapping、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor loop boundary は F5dp の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_loop` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorLoopState` を non-Copy loop state として定義し、F5dn DriverState を保持する。`start` は F5dn start を 1 回だけ呼び、`request` は LoopState を value として消費して F5dn request を 1 回だけ呼ぶ。F5dn `Completed` branch は loop `Completed` へ写すだけで F5do を呼ばない。F5dn `Operation` branch は F5do executor request を 1 回だけ呼ぶ。`complete` は F5do executor complete を 1 回だけ呼び、F5dn DriverCompletion を Continue / Yield の LoopCompletion へ再包装する。F5dp is not actual Web / native / bare / headless execution であり、real scheduler policy でもない。F5dn complete direct call、F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、F5cw action mapping、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor attempt driver boundary は F5dq の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_attempt_driver` は actual Web / native / bare / headless presenter executor が返した executor supplied attempt を F5dp executor loop completion へ戻す。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttemptDriverStep` は completion-only success value であり、F5dp complete に消費された request / attempt を再保持しない。failure は category と lower F5dp error だけを持ち、lower F5dp error を recovery authority とする。F5dq は F5dp complete wrapper であり、F5do direct call、F5dn / F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、old action path、virtual executor、virtual drain、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `Result::Err GuiError` outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor session boundary は F5dr の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session` は actual Web / native / bare / headless presenter loop が ready state、executor pending request、completion result を sentinel / null なしで保持できる session contract を定義する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionState` は `Ready` または `Completed` であり、`Completed` request は lower loop を呼ばず terminal `Completed` result を返す。`Ready` state だけが F5dp request を 1 回呼び、operation request は `SessionPending` に移る。`session_complete` は pending request と executor attempt を value として消費し、F5dq attempt driver step を 1 回だけ呼び、Continue / Yield を Ready session state に写す。F5dr は actual execution、real scheduler policy、F5dp complete direct、F5do / F5dn / F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、old action path、virtual executor、virtual drain、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `Result::Err GuiError` outcome creation を提供しない。

std layer row tile RLE present host execution report boundary は F5cx の checkpoint である。`std/gui/tile_present_host_execution_report` は F5cw の `GuiRgba8888RowTileRlePresentHostExecutionAction` と executor outcome を action context and executor outcome を失わない report に束ねる。`GuiRgba8888RowTileRlePresentHostExecutionReport` は action と `GuiRgba8888RowTileRlePresentHostExecutionReportKind` を保持し、kind は `Succeeded` または `Failed GuiError` である。report construction は executor-supplied `Result unit GuiError` を data に写すだけなので新しい failure mode を作らない。F5cx は not actual execution and not pending completion であり、F5cv / F5cu / F5ct / F5cs / F5cp / F5co、F5cr request constructor、raw storage、host API、platform API、queue、timer、scheduler、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。caller は `report_outcome` で元の `Result unit GuiError` を取り出し、F5cv `complete_request` へ渡せる。

std layer row tile RLE present host executor boundary は F5cy の checkpoint である。`std/gui/tile_present_host_executor` は actual Web / native / bare executor の手前で、executor target support と returned report association を検査する。`GuiRgba8888RowTileRlePresentHostExecutorSupport` は Window、Offscreen、Device とその非空の組み合わせだけを表す enum で、空 support を表現しない。`GuiRgba8888RowTileRlePresentHostExecutorError` は `UnsupportedAction` / `ReportActionMismatch`、category、expected action、reported action option を保持する typed error である。`validate_report_for_action` は support validation の後、F5cx report が持つ action と expected action の full action identity を比較する。full action identity は variant、window、surface、frame、packet metadata、run offset、run count、RGBA channel を含む。matching action の failed report は association として valid なので、この layer では拒否しない。actual host import execution、F5cv pending completion、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、raw storage、host API、platform API、queue、timer、scheduler、DOM / Canvas / minifb、video memory、fallback、silent no-op は提供しない。

std layer row tile RLE present host report loop bridge boundary は F5cz の checkpoint である。`std/gui/tile_present_host_report_loop_bridge` は F5cv pending request、F5cw action decoding、F5cx `report_outcome`、F5cy `validate_report_for_action` を接続する。`GuiRgba8888RowTileRlePresentHostReportLoopBridgeError` は `ExecutorValidationFailed` または `LoopCompletionFailed` の lower error、category、loop state を保持する。contract は validation before completion であり、support / full action identity 検査に失敗した場合は F5cv `complete_request` を呼ばず、pending previous state を error に返す。validation success の場合だけ F5cx `report_outcome` を F5cv `complete_request` に渡し、pending value を消費する。matching action の failed report は F5cv `HostImportExecutionFailed` へ進み、wrong action report は completion 前に `ExecutorValidationFailed` で止まる。この layer は actual host import execution、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、raw storage、host API、platform API、queue、timer、scheduler、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host execution driver boundary は F5da の checkpoint である。`std/gui/tile_present_host_execution_driver` は F5cv の one-shot pending request を、actual Web / native / bare / headless executor が読む action と completion 用 pending value の組へ束ねる。`GuiRgba8888RowTileRlePresentHostExecutionDriverPending` は `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` と `GuiRgba8888RowTileRlePresentHostExecutionAction` を保持し、pending を所有するため Clone / Copy を持たない。`prepare` は pending request accessor から request を読み、F5cw action を 1 回だけ導出して original pending value と一緒に保持する。executor は `pending_action` だけを読み、実行結果は `Result unit GuiError` として `complete_outcome` に返す。`complete_outcome` は stored action を読んでから pending value を取り出し、F5cx report を作って F5cz bridge へ渡す。F5da は F5cv `complete_request`、F5cy validation、F5cr request construction を直接呼ばず、actual platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present virtual host executor boundary は F5db の checkpoint である。`std/gui/tile_present_virtual_executor` は F5da の one-shot driver pending を deterministic headless / test executor で消費し、actual Web / native / bare executor と同じ F5cw action shape を使う。`GuiRgba8888RowTileRlePresentVirtualExecutor` は F5cy support と F5cs virtual drain を保持する。`execute` は F5da pending action を読み、F5cy `require_supported` を drain mutation より前に実行する。support rejection では F5da `complete_outcome` を one-shot cleanup として呼ぶが、virtual drain は更新せず `SupportRejected` を返す。support success の場合だけ F5cw action を F5cq host-command record へ total mapping し、F5cs virtual drain に流す。drain failure でも F5da `complete_outcome Err` で pending を消費し、recovery executor は original executor のまま `DrainFailed` を返す。driver completion が期待と矛盾した場合は `InconsistentCompletion` で返す。F5db は fallback ではなく actual platform presenter でもない。F5cv direct completion、F5cz direct bridge、F5cr request construction、actual platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、F5cu / F5ct / F5cp / F5co、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host action sink boundary は F5dc の checkpoint である。`std/gui/tile_present_host_action_sink` は actual Web / native / bare presenter から返る executor-supplied outcome を `GuiRgba8888RowTileRlePresentHostActionSinkStep` として action と一緒に保持する。`gui_rgba8888_row_tile_rle_present_host_action_sink_step` は F5cy `require_supported` を先に呼び、unsupported target では typed `UnsupportedAction` を返す。supported action の場合だけ caller が渡した `Result unit GuiError` を step に入れる。F5dc does not manufacture success であり、std layer は `Result::Ok unit` を作って actual execution を成功扱いにしない。F5dc は F5da pending を所有せず、F5da completion、F5cx report、F5cz bridge、F5cr request construction、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host action sink driver boundary は F5dd の checkpoint である。`std/gui/tile_present_host_action_sink_driver` は F5dc の action sink step と F5da の one-shot driver completion を接続する。`gui_rgba8888_row_tile_rle_present_host_action_sink_driver_step` は driver pending から action を借用で読み、caller が渡した executor-supplied outcome を F5dc step に渡す。F5dc rejection では completion を呼ばず、`SinkRejected` の owner-bearing error として original driver pending を返す。F5dc success では同じ outcome を F5da `complete_outcome` に渡し、`GuiRgba8888RowTileRlePresentHostActionSinkDriverStep` として sink step と completion を返す。driver completion failure では pending は既に消費済みなので、F5da driver error と sink step だけを `DriverCompletionFailed` に保持する。F5dd does not manufacture executor outcome。std layer で `Result::Ok unit` や synthetic `Result::Err` を作らず、actual executor の `Result unit GuiError` をそのまま流す。F5dd は F5cv direct completion、F5cz direct bridge、F5cx report construction、F5cr request construction、F5db virtual executor、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host action attempt driver boundary は F5de の checkpoint である。`std/gui/tile_present_host_action_attempt_driver` は actual Web / native / bare executor が返した action attempt と F5da driver pending の action identity を比較してから、F5dd action sink driver へ outcome を渡す。`GuiRgba8888RowTileRlePresentHostActionAttempt` は attempted action と executor-supplied outcome だけを持つ Copy value である。`gui_rgba8888_row_tile_rle_present_host_action_attempt_driver_step` は driver pending から expected action を借用で読み、attempt action と F5cy full action equality で比較する。一致しない場合は F5dd を呼ばず、`AttemptActionMismatch` の owner-bearing error として expected action、attempted action、`GuiError::InvalidCommand` category、original driver pending を返す。一致した場合だけ、attempt outcome を F5dd `gui_rgba8888_row_tile_rle_present_host_action_sink_driver_step` に委譲する。F5de does not manufacture executor outcome。`Result::Ok unit` や synthetic `Result::Err` を作らず、attempt に含まれる `Result unit GuiError` だけを流す。F5de は F5dc direct call、F5cv direct completion、F5cz direct bridge、F5cx report construction、F5cr request construction、F5db virtual executor、raw storage、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn boundary は F5ds の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn` は actual Web / native / bare / headless scheduler が F5dr session state または executor pending request のどちらを所有しているかを、1 turn 分の typed state として保持する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnState` は `Session` と `Pending` だけで構成し、no separate Completed turn state を持たない。terminal completion の authority は F5dr session state と F5dr session request に残すため、F5ds は F5dr `SessionState::Ready` / `Completed` を直接見ない。`turn_poll` は owner-consuming API であり、`Pending` は executor へそのまま移し、`Session` だけが F5dr session request を 1 回呼ぶ。`turn_complete` は F5dr session complete だけを 1 回呼び、Continue / Yield を `Session` turn state へ包み直す。この boundary は real scheduler policy、queue、timer、actual platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic outcome creation を提供しない。

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
