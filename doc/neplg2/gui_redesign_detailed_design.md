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
