# NEPLg2 GUI bitmap surface redesign specification

作成日: 2026-06-13

## 目的

この文書は、NEPLg2 GUI を DOM や Canvas2D primitive に依存しない bitmap surface model へ再設計するための標準仕様である。

GUI application code は Web、native、bare、offscreen、headless のどの backend でも同じ `init`、`update`、`view`、`render` の形で動く。違いは `GuiHost`、`GuiCapabilities`、surface、event source に閉じる。

この仕様は `doc/neplg2/gui_standard_library_spec.md` を置き換えるものではなく、Web GUI 表示経路と pixel surface / video memory contract の正式化を追加する。既存の stdout frame stream や Canvas2D direct drawing は正式な presentation contract ではない。

## 方針

- GUI app content は DOM node として描画しない。
- Web の可視 canvas は `putImageData` による bitmap presentation だけを行う。
- `draw_line`、`draw_text`、`fill_rect` などの public drawing API は `DrawCommand` として保持する。
- `DrawCommand` は backend が直接可視 surface に描かず、必ず rasterizer を通して pixel buffer へ反映する。
- `SharedArrayBuffer` video memory surface を Web GUI の正式 presentation path とする。
- 代替実行経路を silent に選ぶ fallback は作らない。
- 未対応の surface、command、pixel format、host capability は `Result::Err GuiError::Unsupported` またはより具体的な `GuiError` で返す。
- headless と offscreen は fallback ではなく、独立した正式 backend である。
- platform 固有名、DOM、Canvas、minifb、Win32、AppKit、Wayland、X11、JavaScript `null` / `undefined` は `core/gui`、`alloc/gui`、`std/gui` の public type に出さない。

この方針は、platform 依存処理を表層へ閉じ、失敗を `Option` / `Result` と enum で表す設計方針に従う。実装都合の暫定 path を hidden fallback として残してはいけない。

## 標準 pipeline

標準 rendering pipeline は次で固定する。

```text
Model + GuiEvent -> Update Model
Model -> ViewTree
ViewTree + LayoutContext -> LayoutTree
LayoutTree + RenderContext -> DrawCommand stream
DrawCommand stream + Rasterizer -> PixelBuffer
PixelBuffer + DirtyRegion + FrameEpoch -> SurfaceFrame
SurfaceFrame -> SurfacePresenter
```

`SurfacePresenter` は visible window、offscreen image、bare display などの presentation を担当する。`SurfacePresenter` は application state を知らない。

## Application-facing present effect

Application が pixel frame を[表示/ひょうじ]したい場合、platform host を直接呼ばない。`update` は `GuiEffect::PresentSurface` 相当の request data を返し、`std/gui/runtime` が checked `GuiSurfacePresentCommand` を[作/つく]って `GuiRuntimeCommand::PresentSurface` へ[解釈/かいしゃく]し、platform backend だけが実際の host call を行う。

```text
Update Model
    effects:
        PresentSurface PresentSurfaceEffect

std/gui/runtime
    GuiEffect::PresentSurface
        -> validate SurfaceId / FrameId / PixelBufferDescriptor
        -> GuiSurfacePresentCommand
        -> GuiRuntimeCommand::PresentSurface

platform backend
    GuiRuntimeCommand::PresentSurface
        -> Web video memory surface
        -> native framebuffer presenter
        -> bare framebuffer / flush target
        -> offscreen pixel buffer
```

Contract:

- `alloc/gui` は `std/gui/surface` を import しない。`PresentSurfaceEffect` は core 型と検査前の request data だけを保持する。
- `std/gui/runtime` は host capability を確認し、pixel buffer を持たない backend では `GuiError::Unsupported` を返す。
- `std/gui/runtime` は surface id、frame id、pixel buffer descriptor を検査し、不正な id は `GuiError::InvalidCommand`、不正な geometry / stride は `GuiError::InvalidGeometry`、未対応 format は `GuiError::Unsupported` として返す。
- `Headless` backend は present を成功させない。test が present 不要な app logic だけを検査する場合は、そもそも `PresentSurface` effect を発行しない。
- `OffscreenPixel` backend は visible window を作らず、同じ `PresentSurface` command を owned pixel buffer へ反映する。
- `TextGrid` backend は pixel frame present を自動的に text grid present へ変換しない。必要なら application / renderer が `TextCellRun` command stream を生成する。
- Web stdout protocol は `PresentSurface` の代替ではない。互換 smoke transport として隔離し、正式 effect / runtime command へ暗黙に置き換えない。

この境界により、同じ NEPL app code は `GuiEffect` と `GuiRuntimeCommand` だけを見ればよく、platform 差は `GuiHost` と backend implementation に閉じる。

Web backend の presentation は次である。

```text
DrawCommand stream
    -> Web software rasterizer
    -> Rgba8888 PixelBuffer
    -> SharedArrayBuffer video memory surface
    -> ImageData
    -> visible canvas putImageData
```

Visible canvas に `fillRect`、`stroke`、`fillText`、`drawImage` を直接呼ぶ path は標準 GUI content renderer ではない。

## Surface model

Surface は描画対象の種類ではなく、pixel ownership と presentation capability の contract で分ける。

```text
SurfaceKind:
    WindowPixel
    OffscreenPixel
    DevicePixel
    TextGrid
    Headless
```

`WindowPixel` は Web / native window に表示する pixel surface である。`OffscreenPixel` は screenshot、image export、CI snapshot 用の owned pixel surface である。`DevicePixel` は bare / embedded display や framebuffer である。`TextGrid` は terminal / TUI の cell surface である。`Headless` は event / update / layout / effect interpretation を検査する surface なし backend である。

`Headless` backend は `present` を成功させない。`present` が必要な code は `Result::Err GuiError::Unsupported` を受け取り、呼び出し側が `match` で扱う。

既存文書や現行実装で使っている `SurfaceKind::Pixel` は、移行後は `WindowPixel` または `DevicePixel` へ分かれる。`SurfaceKind::Command` は surface ではなく、rasterizer へ渡す前の `DrawCommand stream` を表す概念として扱う。実装を更新するまで現行 enum は checkpoint として残るが、新しい実装 slice は `WindowPixel` / `OffscreenPixel` / `DevicePixel` / `TextGrid` / `Headless` の意味へ寄せる。

```text
Old term -> New contract
    Pixel -> WindowPixel or DevicePixel
    Command -> DrawCommand stream before rasterizer
    stdout fallback -> legacy smoke transport, not presentation backend
    Canvas adapter -> bitmap presenter using putImageData only
```

## Pixel buffer contract

最初の正式 pixel format は `Rgba8888` とする。

```text
PixelBuffer:
    width
    height
    stride_bytes
    format
    pixels
```

Contract:

- `width` と `height` は 1 以上である。
- `stride_bytes` は 1 row の byte 数であり、`width * 4` 以上である。
- `format` は初期実装では `Rgba8888` のみである。
- pixel の channel は 0..255 の byte value とする。
- alpha は premultiplied ではなく straight alpha とし、合成責務は rasterizer が持つ。
- invalid geometry、unsupported format、stride mismatch は `GuiError` で返す。

## Video memory surface contract

Video memory surface は host と guest / worker が frame を受け渡す共有 surface である。

```text
VideoMemorySurface:
    header
    pixel_plane

VideoMemoryHeader:
    magic
    version
    width
    height
    stride_bytes
    format
    frame_epoch
    writer_state
    presenter_state
    dirty_kind
    dirty_rect
    resize_generation
```

Contract:

- surface は 2 個以上の pixel plane slot を持つ。
- writer は free slot だけを取得して書き込み、published / reading slot を変更しない。
- presenter は published slot だけを reading にして読み、`putImageData` 完了後に free へ返す。
- resize は新しい surface allocation と `resize_generation` で表す。古い surface に silent に書き続けてはいけない。
- dirty region が `Full` の場合は全体を present する。`Rect` / bounded rect set の場合はその範囲だけを present してよい。
- `SharedArrayBuffer` が使えない Web environment は `GuiError::Unsupported` とする。ArrayBuffer transfer や stdout transport を自動選択しない。
- tearing 防止のため、single buffer video memory surface は禁止する。

Typed failure:

```text
SharedBufferUnavailable
InvalidHeaderMagic
UnsupportedHeaderVersion
InvalidHeaderLayout
NoWritableSlot
StaleResizeGeneration
PresenterUnavailable
WriterClosed
UnsupportedCommand
```

これらは Web backend 内では typed error union として保持し、stdlib 境界では `GuiError` へ変換する。

## Rasterizer contract

Rasterizer は `DrawCommand` を pixel buffer に変換する pure data processor である。

最初に扱う command:

```text
FillRect
Line
TextRun
RgbaRow
Clear
```

`TextRun` は visible canvas の `fillText` を直接呼ばない。初期実装では ASCII bitmap font rasterizer を使う。将来は NEPL stdlib 側の font / glyph rasterizer に置き換える。

Unsupported command は描画を継続しない。`GuiError::Unsupported` を返し、partial frame を publish しない。

## Event model

Event は platform raw event ではなく `GuiEvent` として扱う。

Event source は次に分ける。

```text
EventSource:
    Platform
    VirtualScript
    ReplayLog
    PollingDevice
```

Web / native は platform event を `GuiEvent` へ正規化する。Bare は polling input を `GuiEvent` へ正規化する。Headless / offscreen test は virtual event script と virtual clock で同じ `GuiEvent` を注入する。

Virtual event は test-only shortcut ではなく、正式 backend contract である。timer、resize、close request、pointer、keyboard、text input、lifecycle を deterministic に再生できなければならない。

## Screenshot and offscreen

Screenshot は visible window の副作用ではない。`OffscreenPixel` surface または `capture_surface` が `PixelBuffer` snapshot を返す。

Contract:

- screenshot は `PixelBuffer` と metadata を返す。
- snapshot comparison は deterministic pixel hash または golden image で行える。
- visible window が存在しない CI でも offscreen rasterization と screenshot は動作する。
- headless は pixel surface を持たないため screenshot は `GuiError::Unsupported` である。

## Backend requirements

Web:

- visible canvas は `putImageData` only。
- GUI content 用 DOM node を作らない。
- `SharedArrayBuffer` video memory surface を正式 path とする。
- direct Canvas2D primitive drawing は禁止する。
- `platforms/gui/web/stdout_protocol.nepl` を application が直接 import する経路は移行対象であり、正式な same-app-code contract ではない。

Native:

- framebuffer / pixel surface を native window へ present する。
- OS event pump は backend が所有する。
- resize、close request、minimize / restore、surface unavailable は typed event / state に正規化する。
- minifb 等の crate は backend detail であり public stdlib API に出さない。

Bare:

- window system を要求しない。
- fixed framebuffer、optional `FlushTarget`、polling input を持つ。
- allocator がない場合でも `core/gui` の drawing と dirty region を使える。

Offscreen:

- owned pixel buffer に render する。
- screenshot / snapshot test を primary use case とする。

Headless:

- app update、event replay、effect interpretation を検査する。
- `present` と screenshot は unsupported。

## Implementation gate

実装開始前に次の 3 文書を作成し、subagent review で承認を得る。

- `doc/neplg2/gui_redesign_spec.md`
- `doc/neplg2/gui_redesign_detailed_design.md`
- `doc/neplg2/gui_redesign_implementation_plan.md`

Review が `Blocker` または実装開始不可を返した場合は、doc を修正して再 review する。承認前に stdlib / Web implementation を変更しない。
