# NEPLg2 GUI font rendering detailed design

作成日: 2026-06-13

## 責務分離

Font rendering は `core/gui`、`alloc/gui`、`std/gui`、`platforms/gui/*` で責務を分ける。

```text
core/gui/font
    id
    writing mode
    font request flags
    no_alloc metrics
    glyph paint bridge

alloc/gui/font
    sfnt table directory
    cmap / head / hhea / hmtx / maxp / name / os2
    loca / glyf / CFF / CFF2
    gsub / gpos / kern
    vhea / vmtx / vorg
    variation tables
    shaped run
    glyph outline
    glyph cache

alloc/gui/text
    inline layout
    bidi
    line break
    ruby
    vertical layout
    math inline bridge

std/gui/font_resource
    resource request
    VFS / filesystem / embedded blob source descriptor
    decode policy
    font collection registration boundary

platforms/gui/web
    bundled font VFS mapping
    resource bytes provider
    pixel presentation only

platforms/gui/native
    resource root / packaged resource provider
    framebuffer presenter

platforms/gui/bare
    embedded blob provider
    unsupported filesystem behavior

platforms/gui/headless
    explicit fixture resource provider
    no present / screenshot fallback
```

依存方向は上から下へ流れない。`core/gui/font` は `alloc`、`std`、`platforms` を import しない。`std/gui/font_resource` は platform resource provider の抽象 request だけを作り、OS handle や browser object を持たない。

## Resource flow

初期 fixture は repository 既存の `web/src/fonts/HackGenConsoleNF-Regular.ttf` である。ただし public API は HackGen 専用名を持たない。

```text
resource root
    -> fonts/HackGenConsoleNF-Regular.ttf
    -> GuiFontResourceRequest
    -> std/gui resource provider
    -> alloc/gui sfnt parser
    -> GuiFontFaceId
    -> shaped run / metrics / glyph masks
    -> render2d pixel buffer
```

Web では build/VFS が `fonts/HackGenConsoleNF-Regular.ttf` として提供する。Native では packaged resource directory または configured resource root から同じ path を読む。Bare では embedded blob table を使い、存在しない場合は unsupported または resource missing として返す。

## Decode policy

```text
GuiFontDecodePolicy:
    SfntOnly
    SfntAndWoff
    AllSupportedContainers
```

`SfntOnly` は TTF / OTF / TTC / OTC の sfnt container だけを許す。WOFF / WOFF2 は compressed container decode が必要なため後続 phase とする。`AllSupportedContainers` でも、実装がまだ対応していない container は typed error で拒否する。

## Face selection

```text
GuiFontFaceSelection:
    resource GuiFontResourceId
    face_index Option i32
```

`Option::None` の意味は font container の face count に依存する。

- single face: `None` は face 0 を表す。
- collection face: `None` は invalid。`FaceIndexRequired` 相当の error にする。
- negative index: invalid。
- `face_count <= index`: invalid。

この規則は source policy と doctest で固定する。

F2 の `GuiFontResourceRequest` constructor は request shape だけを検査する。つまり path が empty でないこと、`face_index` が `Some n` の場合に `n >= 0` であること、hash value が typed `GuiResourceHash` であること、decode policy が enum value であることを確認する。Collection font の `face_count` を必要とする `FaceIndexRequired` / out-of-range 判定は、F4 の sfnt metadata parser または font registry が font bytes を読める段階で行う。

## Metrics fixed-point

初期 core contract は i32 fixed-point value を使う。scale 単位は renderer/layout contract で決める。`GuiFontSize` は numerator/denominator を持つ。

```text
GuiFontSize:
    px_num i32
    px_den i32
```

`px_den <= 0` は invalid command である。Validated constructor は `Result GuiFontSize GuiError` を返す。Unchecked raw struct construction を public helper として推奨しない。

## Glyph metrics

Glyph metrics は logical advance と rendered bounds を分ける。

```text
GuiGlyphMetrics:
    glyph GuiGlyphId
    advance_x i32
    advance_y i32
    ink_bounds GuiRect
    allocation_bounds GuiRect
```

`advance_x` / `advance_y` は writing mode に依存する。縦書きでは `advance_y` が主 advance になり得る。`ink_bounds` は実際に塗られる可能性のある領域で、`allocation_bounds` は layout が確保する領域である。

## Rendered text metrics

Rendered text metrics は layout と screenshot / dirty region の接点である。

```text
GuiRenderedTextMetrics:
    logical_bounds GuiRect
    ink_bounds GuiRect
    allocation_bounds GuiRect
    baseline i32
```

`logical_bounds` は inline layout 上の box、`ink_bounds` は actual glyph mask / stroke / shadow の bounds、`allocation_bounds` は repaint / hit test / caret に使う保守的 bounds である。Shadow がある場合は `ink_bounds` に含める。

## Glyph paint

Glyph paint は `render2d` と同じ style model を使う。

```text
GuiBlendMode:
    SourceOver
    Copy
    Multiply
    Screen

GuiShadow:
    offset GuiPoint
    blur_radius i32
    spread i32
    paint GuiPaint

GuiGlyphPaint:
    fill Option GuiPaint
    stroke Option GuiStroke
    shadows GuiShadowRef
    blend GuiBlendMode
```

```text
GuiShadowRef:
    NoShadow
    SingleShadow GuiShadow
    ShadowRun GuiShadowRunId

GuiShadowRunId:
    raw i32
```

初期 slice は `Solid` 相当の既存 `GuiPaint` と既存 `GuiStroke` を再利用する。Gradient / pattern / layer は後続で `GuiPaint` の拡張または `render_style` 側の上位型として追加する。

High-level style は `Vec GuiShadow` を持てるが、no_alloc core command は `GuiShadowRef` だけを持つ。`GuiShadowRef::ShadowRun` は alloc layer が owns する shadow run を参照する id であり、core は `Vec` を import しない。F1 の実装は `NoShadow` と `SingleShadow` を扱い、`ShadowRun` は id value として保持するだけで展開しない。

## Validation

`core/gui/font` は no_alloc の O(1) validation helper だけを持つ。

- font size denominator が 0 以下なら invalid。
- font weight が 1..1000 外なら invalid。
- variation axis tag は four-byte tag として別 type 化する。
- fill と stroke が両方 `None` の glyph paint は invalid。
- shadow blur / spread が negative の場合は invalid。

F1/F2 の戻り値は既存 `Result T GuiError` を使う。font-specific reason は `GuiFontErrorKind` data value として定義し、将来の detailed diagnostic payload へ接続できるようにする。`GuiFontErrorKind` は表示文字列ではなく enum であり、error display は std/platform 側に置く。

## Test strategy

最小 slice では次をテストする。

- `GuiWritingMode` を match できる。
- `GuiFontSize` validation が invalid denominator を拒否する。
- `GuiRenderedTextMetrics` が logical / ink / allocation を別々に保持する。
- `GuiGlyphPaint` が fill-only、stroke-only、fill+stroke を受け、none+none を拒否する。
- `GuiShadowRef` が no shadow、single shadow、shadow run id を sentinel なしで保持する。
- `GuiFontResourceRequest` が path、face index、expected hash、decode policy を保持する。
- `std/gui/font_resource` が DOM / Canvas / OS handle を公開しない。
- Formal font renderer の contract が `MockTextMeasurer` / `HostTextMeasurer` を fallback として参照しない。

TTF table parsing、glyph outline、rasterization、ruby/vertical/math layout は後続 phase で別テストを追加する。
