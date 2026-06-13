# NEPLg2 GUI bitmap surface detailed design

作成日: 2026-06-13

## 目的

この文書は `doc/neplg2/gui_redesign_spec.md` の詳細設計である。主に pixel buffer、video memory surface、Web bitmap presenter、offscreen / headless backend、virtual event source の内部 contract を固定する。

## 型と責務

標準 API は platform 名を public type に入れない。

```text
GuiSurfaceId
GuiSurfaceConfig
GuiSurfaceState
PixelBufferDescriptor
VideoMemoryDescriptor
FrameEpoch
DirtyRegion
GuiEventSource
VirtualGuiHost
OffscreenGuiHost
HeadlessGuiHost
```

Web の `SharedArrayBuffer`、Canvas `ImageData`、native の window handle、bare の display driver は backend implementation detail である。

## Pixel format

初期実装の pixel format は `Rgba8888` に限定する。

```text
byte offset:
    0 red
    1 green
    2 blue
    3 alpha
```

Alpha は straight alpha とする。Rasterizer が source-over alpha blending を行う場合、pixel buffer には合成後の straight alpha を書く。

Unsupported format は `GuiError::Unsupported` で返す。Format 変換を暗黙に行わない。

## Pixel buffer

Pixel buffer は次の invariant を持つ。

```text
width > 0
height > 0
stride_bytes >= width * 4
stride_bytes % 4 == 0
pixels.length >= stride_bytes * height
```

Out-of-bounds drawing は backend contract で決める。Web software rasterizer は clipping により範囲外 pixel を破棄する。Invalid geometry は clipping ではなく `GuiError::InvalidGeometry` として拒否する。

最低性能目標は 1920 x 1080 の `Rgba8888` pixel buffer を 60 fps で present することである。したがって、hot path は same-size frame ごとの巨大配列再確保を避け、resize generation が変わった時だけ surface slot / bitmap storage を作り直す。Web の legacy command frame renderer も、正式 video memory surface へ移行するまでの checkpoint として canvas ごとの bitmap buffer と `ImageData` を再利用する。これは fallback ではなく、同一 presentation contract 内の allocation policy である。

## Effect and runtime command bridge

`GuiSurfacePresentCommand` は `std/gui` の host surface ABI data contract であり、application はこれを直接 platform へ送らない。`alloc/gui/app` は std 型を import せず、core 型と検査前の request data だけを effect として保持する。`std/gui/runtime` が host capability と checked id constructor を使って `GuiSurfacePresentCommand` を作り、runtime command へ変換する。

```text
alloc/gui/app:
    PresentSurfaceEffect:
        surface i32
        frame i32
        width i32
        height i32
        stride_bytes i32
        format ColorFormat
        dirty DirtyRegion

    GuiEffect:
        None
        RequestRedraw
        SetTitle
        PresentSurface PresentSurfaceEffect

std/gui/runtime:
    GuiRuntimeCommand:
        Noop
        RequestRedraw
        SetTitle
        PresentSurface GuiSurfacePresentCommand
```

Runtime gate:

```text
SurfaceKind::WindowPixel      allow PresentSurface
SurfaceKind::OffscreenPixel   allow PresentSurface
SurfaceKind::DevicePixel      allow PresentSurface
SurfaceKind::TextGrid         reject GuiError::Unsupported
SurfaceKind::Headless         reject GuiError::Unsupported
```

`TextGrid` を reject する理由は、pixel frame と cell frame は同じ surface presentation ではないためである。TUI backend は `TextCellRun` / terminal frame contract を使い、pixel buffer を暗黙に text grid へ変換しない。

`Headless` を reject する理由は、headless が surface なし backend であり、presentation を伴わない app state transition と effect interpretation を検査する target だからである。`PresentSurface` が出る app を headless で実行した場合は `GuiError::Unsupported` を返し、test はその unsupported を `match` で検査する。

`OffscreenPixel` は visible window を持たないが pixel buffer surface を持つため allow する。Screenshot / snapshot test は runtime command を offscreen host が受け、owned pixel buffer へ反映する。

Command validation:

- `PresentSurfaceEffect` は `alloc/gui` に置くため、`SurfaceId`、`FrameId`、`GuiPixelBufferDescriptor`、`GuiSurfacePresentCommand` を持たない。
- `PresentSurfaceEffect` の `surface` と `frame` は platform handle ではなく、runtime が checked constructor に渡す request id である。0 以下や不正値は `GuiError::InvalidCommand` になる。
- `PresentSurfaceEffect` の `width`、`height`、`stride_bytes`、`format` は runtime が `gui_pixel_buffer_descriptor` で検査する。不正 geometry は `GuiError::InvalidGeometry`、未対応 format は `GuiError::Unsupported` になる。
- runtime は payload の中身を platform handle に戻さない。
- unsupported surface kind は `GuiError::Unsupported`、malformed effect payload を作れる経路が後で増えた場合は `GuiError::InvalidCommand` とする。
- batch capacity overflow は既存 `GuiEffectBatch` / `GuiRuntimeCommandBatch` と同じ `GuiError::ResourceExhausted` を返す。

Implementation note:

- Phase 4.1 の NEPL stdlib 実装では `GuiEffectBatch` の capacity 2 を維持する。これは現在の bounded data contract の継続であり、hidden fallback ではない。
- 将来 `Vec GuiEffect` へ置換するときも、`PresentSurfaceEffect` は `alloc/gui` の request data として維持し、checked `GuiSurfacePresentCommand` の生成責務は `std/gui/runtime` に残す。

## Offscreen snapshot and virtual event contract

Offscreen は visible window のない pixel surface backend である。`Headless` と同一視しない。

```text
SurfaceKind::OffscreenPixel:
    accepts GuiRuntimeCommand::PresentSurface
    can produce GuiOffscreenSnapshot
    can be used by screenshot / golden image tests

SurfaceKind::Headless:
    rejects PresentSurface with GuiError::Unsupported
    rejects screenshot with GuiError::Unsupported
    can run update / event replay tests without presentation
```

`GuiOffscreenSnapshot` は platform handle や pixel owner を持たない std layer value である。

```text
GuiOffscreenSnapshot:
    surface SurfaceId
    frame FrameId
    width i32
    height i32
    stride_bytes i32
    format ColorFormat
    dirty DirtyRegion
    pixel_hash i32
```

`pixel_hash` は backend presenter が実 pixel bytes から計算して std contract へ渡す。`std/gui/offscreen` は pixel memory を読まない。Web では video memory surface、native では framebuffer presenter、bare では device / offscreen adapter が hash authority である。これにより std layer は deterministic snapshot comparison の data boundary を持つが、DOM、Canvas、OS handle、raw pointer へ依存しない。

Validation:

- `GuiRuntimeCommand::PresentSurface` だけが snapshot source になる。
- `GuiRuntimeCommand::Noop`、`RequestRedraw`、`SetTitle` は screenshot source ではないため `GuiError::Unsupported` を返す。
- `SurfaceKind::OffscreenPixel` 以外の host で snapshot capture を要求した場合は `GuiError::Unsupported` を返す。visible window screenshot は将来別 command として定義する。
- `pixel_hash` は backend-supplied value として保持する。0 を special sentinel にしない。

Virtual event source は正規化済み `GuiEvent` を保持する test helper である。

```text
GuiVirtualClock:
    now_ms i32
    tick i32

GuiVirtualEventScript:
    first Option GuiEvent
    second Option GuiEvent
    count i32
    cursor i32

GuiVirtualEventPoll:
    script GuiVirtualEventScript
    event Option GuiEvent
```

Initial implementation は `GuiEffectBatch` と同じく bounded capacity 2 とする。内部 slot は `Option GuiEvent` であり、empty script は `Option::None` を 2 つ保持する。push は空 slot に `Option::Some event` を入れる。capacity overflow は `GuiError::ResourceExhausted` を返す。この bounded script は test helper contract であり、platform event queue の最終設計ではない。後続で `Vec GuiEvent` に置換しても、poll が `Option GuiEvent` を返し、overflow を typed error として扱う契約は維持する。

Sentinel は使わない。

- `GuiEvent::None` は追加しない。
- empty poll は `Option::None` で表す。
- raw string、DOM event object、OS event handle は virtual event script に入れない。

Virtual clock:

- `gui_virtual_clock_result now_ms` は 0 以上の initial clock を作る。negative time は `GuiError::InvalidCommand` として拒否する。
- `gui_virtual_clock_advance clock delta_ms` は negative delta を `GuiError::InvalidCommand` として拒否する。
- `now_ms + delta_ms` または `tick + 1` が i32 positive range を超える場合は wrap せず `GuiError::InvalidCommand` とする。
- advance は OS clock を読まない。caller が渡した delta だけで deterministic に進む。
- timer event は `GuiEvent::Timer` として script に入れる。virtual timer scheduler は後続 slice で追加する。

Pixel hash:

- `pixel_hash` は signed opaque `i32` として全 bit pattern を有効値にする。
- 0 や -1 を sentinel にしない。
- hash algorithm と collision policy は backend presenter contract で定義し、std layer は hash value を保持するだけにする。

## Video memory header

Web backend の video memory surface は `SharedArrayBuffer` を 1 つ使い、header と 2 個以上の pixel plane を同じ buffer 内に置く。単一 pixel plane は writer と presenter が同じ memory を同時に触る危険があるため、正式 contract では禁止する。

Header は `Int32Array` view で扱う。

```text
index 0  magic
index 1  version
index 2  width
index 3  height
index 4  stride_bytes
index 5  format
index 6  resize_generation
index 7  slot_count
index 8  latest_published_epoch
index 9  latest_presented_epoch
index 10 surface_state
index 11 error_code
index 12 header_int32_length
index 13 pixel_plane_byte_offset
index 14 pixel_plane_byte_length
index 15 reserved
```

Slot header は global header の後ろに slot ごとに並べる。

```text
slot index 0  slot_state
slot index 1  slot_epoch
slot index 2  dirty_kind
slot index 3  dirty_x
slot index 4  dirty_y
slot index 5  dirty_width
slot index 6  dirty_height
slot index 7  reserved
```

Pixel plane は slot header 群の後ろに slot 順で置き、各 slot を `Uint8ClampedArray` view として読む。`pixel_plane_byte_length` は 1 slot 分の byte length であり、`stride_bytes * height` 以上である。

`magic` と `version` は incompatible buffer を fail-closed に拒否するために使う。Unknown magic / version は `GuiError::InvalidCommand` 相当の Web typed error にする。

## Web Canvas video memory presenter

Web の正式 presenter は video memory surface の published slot を `ImageData` として `putImageData` へ渡す。Visible Canvas は presentation device であり、GUI content の drawing authority ではない。

Presenter が使ってよい Canvas API は次に限定する。

```text
new ImageData
putImageData
```

`fillRect`、`stroke`、`drawImage`、CSS transform、DOM element による widget 表現は正式 video memory presenter の hot path に入れない。図形、文字、UI chrome は pixel buffer へ rasterize 済みの byte 列として渡される。

Initial Web presenter は tightly packed `Rgba8888` だけを受ける。

```text
stride_bytes == width * 4
pixel_byte_length == width * height * 4
```

Padded stride は暗黙に row copy しない。`ImageData` と互換でない stride は typed `UnsupportedStride` 相当の error として拒否する。Acquired slot の stride が拒否された場合は、slot を discard して writer を詰まらせず、presented epoch は進めない。後続で tiled presenter や row-copy presenter を追加する場合も、allocation / copy policy を明示した別 path とする。

Dirty region は fail-closed に検査する。

```text
x >= 0
y >= 0
width >= 0
height >= 0
x + width <= surface.width
y + height <= surface.height
```

範囲外 dirty region は clamp しない。`InvalidDirtyRegion` として返し、slot は `Reading` から解放するが、presented epoch は進めない。

Zero-size dirty region は valid な no-op presentation とする。

```text
width == 0 または height == 0:
    putImageData は呼ばない
    slot は release する
    presented epoch は進める
```

Canvas presentation が失敗した場合は typed `PresentFailed` として返す。JavaScript exception の message は branch authority にしない。失敗した frame は表示済みではないため、slot は discard して writer を詰まらせないが、presented epoch は進めない。

成功時:

```text
Published -> Reading -> putImageData -> Free
presented_epoch = slot.epoch
```

reject / failure 時:

```text
Published -> Reading -> discard -> Free
presented_epoch は変更しない
```

FHD 60 fps の最低性能目標を満たすため、presenter は `SharedArrayBuffer` と slot index ごとに `ImageData` を cache する。同じ slot を再利用する frame では `ImageData` を再生成せず、同じ underlying byte view の内容更新だけを presentation に反映する。

## Frame publish protocol

Slot state:

```text
Free
Writing
Published
Reading
Closed
```

Surface state:

```text
Ready
Closing
Closed
Unavailable
```

Protocol:

1. writer は slot header を走査し、`Atomics.compareExchange(slot_state, Free, Writing)` に成功した slot だけを取得する。
2. writer は取得した slot の pixel plane だけを更新する。`Published`、`Reading`、`Closed` の slot へ書いてはいけない。
3. writer は slot dirty region を書く。
4. writer は slot epoch を新しい値へ `Atomics.store` する。
5. writer は `Atomics.store(slot_state, Published)` で publish し、`latest_published_epoch` を更新して `Atomics.notify` する。
6. presenter は `Atomics.compareExchange(slot_state, Published, Reading)` に成功した slot だけを読む。
7. presenter は `ImageData` を作り、visible canvas へ `putImageData` する。`putImageData` が完了するまで slot は `Reading` のまま保持する。
8. presenter は `latest_presented_epoch` を更新し、`Atomics.store(slot_state, Free)` で slot を writer へ返す。

Atomics ordering:

- slot acquisition は `Atomics.compareExchange` で行う。
- pixel plane 書き込み後、slot metadata と slot state は `Atomics.store` で publish する。
- presenter は slot state を `Atomics.load` / `compareExchange` で確認してから pixel plane を読む。
- presenter への通知は `Atomics.notify`、blocking wait が必要な writer / presenter は `Atomics.wait` を使う。
- writer は free slot がない場合、API contract に応じて wait するか `GuiVideoMemoryError::NoWritableSlot` を返す。別 transport を選んではいけない。

途中で surface が resize / close された場合、writer は old generation への publish を `GuiVideoMemoryError::StaleResizeGeneration` として失敗させる。Old surface へ silently publish しない。

Visible window presenter は published pixel buffer を 1:1 で表示する。Window が広がった場合も、presenter は古い pixel buffer を拡大縮小しない。Backend は drawable surface の logical pixel size を `WindowEvent::Resized` として発行し、application / layout engine が新しい size の pixel buffer を生成する。新しい frame が来るまでの余白は surface background であり、CSS transform、Canvas scale、fit-to-window viewport による content stretch は禁止する。

## Dirty region

Dirty region は初期実装では次を扱う。

```text
DirtyKind:
    Empty
    Rect
    Full
```

Multiple rect set は stdlib の `DirtyRegionSet` contract が安定した後に追加する。初期 Web presenter は `Rect` と `Full` を扱い、`Empty` は present しない。

Dirty rect の width / height が負の場合は invalid。Zero-size rect は valid だが present は行わない。

## Web rasterizer

`web/src/gui-preview` は次に分割する。

```text
commands.ts
host-bridge.ts
stdout-protocol.ts
bitmap-buffer.ts
bitmap-rasterizer.ts
bitmap-presenter.ts
video-memory-surface.ts
canvas-renderer.ts
```

`canvas-renderer.ts` は compatibility facade として残す場合でも、visible canvas direct primitive を呼ばない。責務は frame を bitmap buffer に rasterize し、presenter に渡すことだけである。

`canvas-renderer.ts` が legacy command frame を visible canvas に表示する場合でも、viewport は `left = 0`、`top = 0`、`scale = 1` の logical pixel mapping に固定する。Device pixel ratio は backing bitmap への rasterize scale としてだけ扱う。`padding`、centering、`availableWidth / frame.width` による fit scale を再導入してはいけない。

禁止:

```text
ctx.fillText
ctx.strokeText
ctx.fillRect for app content
ctx.stroke
ctx.drawImage for app content
DOM element creation for app content
```

許可:

```text
new ImageData
ctx.putImageData
canvas width / height synchronization
```

Canvas background clear も pixel buffer 側で行う。Visible canvas context は pixel presentation 以外に使わない。

## Text rasterization

初期 Web implementation は ASCII bitmap font を持つ。

Contract:

- ASCII printable range を deterministic bitmap glyph として描く。
- Unsupported scalar は replacement box glyph として描くのではなく、初期実装では `GuiError::Unsupported` として frame を publish しない。
- Text alignment は rasterizer が glyph width から開始 x を計算する。
- Font shaping、IME composition、complex script は後続の NEPL font rasterizer で扱う。

この設計により、visible canvas の `fillText` に依存しない。

## Offscreen backend

Offscreen backend は owned `PixelBuffer` を持つ。

```text
OffscreenGuiHost:
    create_surface
    render_frame
    capture_surface
    destroy_surface
```

`capture_surface` は current pixel buffer の snapshot を返す。Snapshot は pixel hash、width、height、format を持つ。

Offscreen backend は CI、screenshot、visual regression に使う。Visible window を作らない。

## Headless backend

Headless backend は surface を持たない。

許可:

- `init`
- `update`
- `view`
- layout without presentation
- effect interpretation
- virtual event replay

禁止:

- `present`
- `capture_surface`
- visible window creation

禁止 operation は `GuiError::Unsupported` で返す。

## Virtual event source

Virtual event source は platform event と同じ `GuiEvent` を生成する。

```text
VirtualEventScript:
    events
    clock
    cursor

VirtualClock:
    now_ms
    scheduled_timers
```

Contract:

- replay order は deterministic。
- timer は virtual clock の advance によってだけ発火する。
- pointer / keyboard / text input / resize / close request は platform event と同じ typed shape を使う。
- invalid event script は `GuiError::InvalidCommand` とする。

## Native backend

Native backend は OS event pump を ownership boundary とする。

```text
native event pump
    -> GuiEvent
GuiEvent
    -> app update
DrawCommand
    -> native software rasterizer
PixelBuffer
    -> native framebuffer presenter
```

Window handle、minifb、Win32、AppKit、Wayland、X11 は backend 内部に閉じる。Resize 中の pointer move や window state は coalesce してよいが、close request、button up、keyboard、text input、action event を上書きしてはいけない。

## Bare backend

Bare backend は `DevicePixel` surface を使う。

```text
fixed framebuffer
optional flush target
polling input
dirty region
```

Allocator は要求しない。Widget tree が allocator を必要とする場合、その app は bare no_alloc profile では `MissingCapability` になる。

## Error policy

Video memory typed errors:

```text
GuiVideoMemoryError:
    SharedBufferUnavailable
    InvalidSurfaceConfig
    InvalidBufferLength
    InvalidHeaderMagic
    UnsupportedHeaderVersion
    InvalidHeaderLayout
    InvalidSurfaceState
    InvalidSlotState
    NoWritableSlot
    NoPublishedSlot
    StaleResizeGeneration
    PresenterUnavailable
    WriterClosed
    WaitUnavailable
    UnsupportedPixelFormat
    UnsupportedCommand
```

`GuiVideoMemoryError` は Web implementation detail の typed error であり、stdlib 境界では対応する `GuiError::Unsupported`、`GuiError::InvalidCommand`、`GuiError::ResourceExhausted`、`GuiError::BackendFailure` へ lossless に近い形で map する。文字列 sentinel へ潰さない。

次を禁止する。

- unsupported operation の no-op 成功
- hidden fallback
- panic / unreachable による通常失敗処理
- string sentinel
- `null` / `undefined` の public boundary 露出

次を使う。

- `Option`
- `Result`
- `GuiError`
- backend-specific typed error union
- exhaustive `match`

## Static regression policy

実装後は source policy test で次を固定する。

- visible Web renderer に `fillText` がない。
- visible Web renderer に app content 用 `fillRect` がない。
- visible Web renderer に `putImageData` がある。
- Web command DTO に DOM / Canvas 型がない。
- `core/gui` / `alloc/gui` / `std/gui` に Web / native concrete type name がない。
- stdout GUI presentation を正式 path として参照しない。
