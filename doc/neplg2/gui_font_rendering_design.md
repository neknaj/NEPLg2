# NEPLg2 GUI font rendering design

作成日: 2026-06-13

## 目的

この文書は、bitmap font による暫定 GUI renderer の次に作る本格的な font rendering library の仕様と実装計画である。

目標は、Web、native、bare、offscreen、headless で同じ NEPL GUI application code が text layout と text rendering を扱えることである。Browser の `CanvasRenderingContext2D.fillText`、DOM、OS font API、JavaScript library は標準 API の前提にしない。Host は font bytes、surface、event、resource I/O を提供し、NEPL 側の library が font table parsing、metrics、layout、shaping、rasterization を持つ。

最初の実 font は repository 既存の `web/src/fonts/HackGenConsoleNF-Regular.ttf` を使う。Web では VFS の `fonts/HackGenConsoleNF-Regular.ttf` として見せ、native では filesystem 上の `fonts/HackGenConsoleNF-Regular.ttf` として読めるようにする。Resource path は platform ごとに変えず、host resource layer が実体へ mapping する。

2D rendering engine の shared paint / stroke / shadow / SVG / UI skin / pixel buffer contract は `doc/neplg2/gui_2d_rendering_design.md` に分ける。Font engine は glyph metrics、shaping、positioning、outline / mask generation を担当し、2D renderer は glyph mask と path / image / widget skin を同じ visual style model で合成する。

## Design principles

- `core/gui` は font file bytes、browser font object、OS font handle を持たない。
- `alloc/gui` は font tables、glyph map、layout cache、shaped run、ruby / vertical layout tree を owns してよい。
- `std/gui` は font resource loading、VFS / filesystem resource path、font collection registration、host cache integration を owns する。
- `platforms/gui/web` は repository bundled font を VFS に載せるが、browser の `FontFace` や Canvas text measurement は formal renderer の authority にしない。
- unsupported glyph、unsupported font table、unsupported writing mode、unsupported math layout は silent fallback にしない。`Result::Err GuiError::Unsupported` または font-specific typed error を返す。
- Layout 用 metrics と rendering 用 rasterization は同じ `GuiFontFace` / variation / size / writing mode / feature set から作る。測定だけ host API、描画だけ別 engine、という分離は禁止する。
- Ruby、縦書き、日本語 text、将来の数式描画は最初から inline layout model の一部として扱う。

## Resource model

Resource path は次で固定する。

```text
fonts/HackGenConsoleNF-Regular.ttf
fonts/HackGen-LICENSE.txt
```

Repository 内の現物は次である。

```text
web/src/fonts/HackGenConsoleNF-Regular.ttf
web/src/fonts/HackGen-LICENSE.txt
```

Web build はこれらを VFS manifest へ登録する。Native build は packaged resource directory または current working directory relative resource root から同じ path を読む。Bare target は filesystem を持たない可能性があるため、font bytes を linked resource blob として渡すか、capability により `GuiError::Unsupported FontResourceLoading` を返す。

```text
FontResourceId:
    path ResourcePath
    content_hash ResourceHash

FontResourceRequest:
    path ResourcePath
    face_index Option i32
    expected_hash Option ResourceHash
    decode_policy FontDecodePolicy

FontDecodePolicy:
    SfntOnly
    SfntAndWoff
    AllSupportedContainers

FontResourceSource:
    Vfs
    FileSystem
    EmbeddedBlob
```

`content_hash` は cache key と reproducibility のために使う。Display name、path suffix、mtime、browser-provided family name は authority にしない。

`face_index` は TTC / OTC collection fonts の face selection に使う。Single-face fonts では `Option::None` または 0 だけを受け付ける。存在しない face index は `FontError::InvalidCollectionHeader` またはより具体的な collection face error として返し、先頭 face へ勝手に切り替えない。

Invalid `face_index` behavior:

- `Option::None` on single-face font means face 0.
- `Option::Some 0` on single-face font means face 0.
- `Option::Some n` where `n < 0` returns `FontError::InvalidFaceIndex`.
- `Option::Some n` where `n >= face_count` returns `FontError::InvalidFaceIndex`.
- `Option::None` on collection font returns `FontError::FaceIndexRequired`.

Target resource behavior:

```text
Web:
    resource source = VFS
    required fixture = fonts/HackGenConsoleNF-Regular.ttf

Native:
    resource source = packaged filesystem or configured resource root
    required fixture path remains fonts/HackGenConsoleNF-Regular.ttf

Bare:
    resource source = EmbeddedBlob
    missing embedded blob returns GuiError::Unsupported FontResourceLoading

Offscreen:
    resource source = the same VFS / filesystem / embedded blob selected by test host
    missing fixture returns ResourceMissing and screenshot is not produced

Headless:
    shaping and layout tests may load font resources through explicit test fixture resources
    present and screenshot remain unsupported
    if no font resource source is configured, font loading returns ResourceMissing or Unsupported FontResourceLoading
```

Headless must not silently switch to fixed-cell measurement for formal GUI font tests. Fixed-cell `MockTextMeasurer` is a separate explicit test measurer.

## Font family and substitution

`HackGenConsoleNF-Regular.ttf` is only the first bundled fixture. Applications must be able to register and use other fonts without changing GUI code.

```text
FontFamilyName:
    name str

FontFamilyEntry:
    family FontFamilyName
    face FontFaceId
    style FontStyleDescriptor
    priority i32

FontStyleDescriptor:
    weight FontWeightRequest
    width FontWidthRequest
    slant FontSlantRequest
    italic bool
    variations FontVariationSet

FontFamilyRegistry:
    entries Vec FontFamilyEntry
```

Font selection request:

```text
FontSelectionRequest:
    families Vec FontFamilyName
    style FontStyleDescriptor
    substitution FontSubstitutionPlan
```

The family list is ordered by the caller. The font engine selects the first registered family/face that satisfies style and glyph requirements under the explicit `FontSubstitutionPlan`.

```text
FontSubstitutionPlan:
    policy FontSubstitutionPolicy
    alternatives Vec FontFamilyName

FontSubstitutionPolicy:
    Disabled
    UseListedFamilies
    UseRegisteredAlternatives
```

Behavior:

- `Disabled`: only the requested first family is used. Missing glyph or style returns `Result::Err FontError::MissingGlyph` or `FontError::MissingFamily`.
- `UseListedFamilies`: use only the ordered `families` in `FontSelectionRequest`.
- `UseRegisteredAlternatives`: use requested families first, then registry alternatives declared by application/theme.

There is no implicit platform font substitution. Browser CSS `font-family` fallback, OS fallback font selection, and fontconfig/CoreText/DirectWrite fallback are not formal authority unless a platform backend exposes the selected font as an explicit registered `GuiFontFace`.

```text
FontSelectionError:
    MissingFamily FontFamilyName
    MissingStyle FontStyleDescriptor
    MissingGlyph UnicodeScalar
    SubstitutionDisabled
```

Theme integration:

```text
GuiTypography:
    body FontSelectionRequest
    monospace FontSelectionRequest
    heading FontSelectionRequest
    math FontSelectionRequest
    ruby FontSelectionRequest
```

This allows examples and user apps to switch away from HackGen by registering a different font resource and changing the family list, not by editing renderer internals.

## Layering

推奨 module tree:

```text
stdlib/
    core/
        gui/
            font/
                id.nepl
                metric.nepl
                glyph.nepl
                writing_mode.nepl
                text_feature.nepl
                error.nepl

    alloc/
        gui/
            font/
                sfnt.nepl
                cmap.nepl
                head.nepl
                hhea.nepl
                hmtx.nepl
                maxp.nepl
                name.nepl
                os2.nepl
                loca.nepl
                glyf.nepl
                kern.nepl
                gsub.nepl
                gpos.nepl
                vhea.nepl
                vmtx.nepl
                vorg.nepl
                font_face.nepl
                glyph_outline.nepl
                glyph_rasterizer.nepl
                glyph_cache.nepl

            text/
                shaped_run.nepl
                line_break.nepl
                bidi.nepl
                ruby.nepl
                vertical.nepl
                inline_box.nepl
                math_inline.nepl
                text_layout_engine.nepl

    std/
        gui/
            font_resource.nepl
            font_registry.nepl
            font_cache.nepl

    platforms/
        gui/
            web/
                font_vfs.nepl
            native/
                font_filesystem.nepl
            bare/
                font_blob.nepl
            headless/
                font_test_resource.nepl
```

`core/gui/font` は id、metrics、writing mode、typed error の value contract だけを持つ。`alloc/gui/font` は TTF/SFNT parsing と rasterization を持つ。`std/gui/font_resource` は resource loading effect を持つ。Platform module は bytes の取得方法だけを実装する。

## Font file support

The font engine must be designed around font containers and outline sources, not around one file extension.

```text
FontContainerKind:
    SfntSingle
    SfntCollection
    Woff
    Woff2

FontOutlineKind:
    TrueTypeGlyf
    OpenTypeCff
    OpenTypeCff2
    BitmapStrike
```

Initial implementation uses uncompressed SFNT single-face TTF because `HackGenConsoleNF-Regular.ttf` is already in the repository. The design must still keep room for the major deployed formats:

```text
.ttf
    SFNT single-face container with TrueType glyf / loca outlines

.otf
    SFNT single-face container, usually CFF or CFF2 outlines

.ttc
    TrueType collection, multiple faces in one file

.otc
    OpenType collection, multiple CFF/CFF2 faces in one file

.woff
    Web Open Font Format container with compressed table payloads

.woff2
    Web Open Font Format 2 container with Brotli-transformed table payloads
```

The first parser phase should reject unsupported container or outline kinds with typed errors while preserving the abstraction boundary. Do not bake `.ttf` assumptions into layout, glyph id, metrics, or cache keys.

Required common SFNT / OpenType tables:

```text
SFNT / OpenType
    common required tables:
        cmap
        head
        maxp
        name
        OS/2

    variable font tables:
        fvar
        gvar
        avar
        HVAR
        VVAR
        MVAR
        STAT

    horizontal layout:
        hhea
        hmtx

    TrueType outlines:
        loca
        glyf

    CFF / CFF2 outlines:
        CFF
        CFF2

    optional layout and metric tables:
        kern
        GSUB
        GPOS
        vhea
        vmtx
        VORG
```

`HackGenConsoleNF-Regular.ttf` is the first required fixture. Parser tests must include:

- table directory validation
- checksum / offset / length bounds
- container kind detection
- face index selection for collection containers
- outline kind detection
- cmap Unicode mapping
- horizontal metrics
- glyph outline loading
- simple glyph rasterization
- composite glyph handling
- missing glyph typed error
- vertical metric table presence / absence handling
- variable font axis detection
- weight axis selection when `wght` is available

If a required table is missing, return `FontError::MissingTable`. If an optional table is missing, do not silently approximate unless the caller explicitly requests an approximation mode. Default formal layout mode returns `FontError::UnsupportedFeature`.

Implementation phases must treat CFF/CFF2、WOFF/WOFF2、and variable font tables as later parser backends behind the same `GuiFontFace` contract. Supporting them must not require changing application layout APIs.

## Metrics contract

Layout engine needs metrics before rasterization.

```text
FontMetrics:
    units_per_em
    ascender
    descender
    line_gap
    cap_height Option i32
    x_height Option i32
    underline_position
    underline_thickness

GlyphMetrics:
    glyph_id
    advance_x
    advance_y
    bearing_x
    bearing_y
    bounds
    vertical_origin Option GuiPoint
```

Scaled metrics:

```text
ScaledFont:
    font_face_id
    px_size
    scale
    variations FontVariationSet
    hinting_mode
    writing_mode
    feature_set
```

```text
FontVariation:
    axis_tag FontAxisTag
    value f32

FontVariationSet:
    items Vec FontVariation

WellKnownFontAxis:
    Weight
    Width
    Slant
    Italic
    OpticalSize
```

Variable font axes are explicit input. If a font exposes `wght`, bold text chooses a `FontVariation Weight value` instead of relying on browser synthetic bold. If the requested axis is absent, return `FontError::UnsupportedVariationAxis` unless a caller-provided style substitution policy is explicit.

Metrics API:

```text
font_face_metrics %fn GuiFontFace FontMetrics
measure_glyph %fn ScaledFont GlyphId Result GlyphMetrics FontError
measure_text_run %fn ShapingContext TextRun Result ShapedRun FontError
rasterize_glyph %fn GlyphRasterContext PositionedGlyph Result GlyphRasterImage FontError
measure_rendered_text %fn TextRenderContext InlineLayoutBox Result RenderedTextMetrics FontError
```

Text layout must not use character count as width except in `MockTextMeasurer` tests. Formal layout uses shaped glyph advances.

Rendered metrics are required for layout decisions after rasterization.

```text
GlyphRasterImage:
    glyph_id
    pixel_width
    pixel_height
    pixel_origin_x
    pixel_origin_y
    advance_x
    advance_y
    ink_bounds
    allocation_bounds
    mask_format

RenderedGlyphMetrics:
    positioned_glyph
    glyph_metrics
    raster_image_bounds
    ink_bounds
    advance_after_hinting

RenderedTextMetrics:
    logical_bounds
    ink_bounds
    allocation_bounds
    line_bounds
    baseline_set
    caret_positions
    hit_test_map
    glyphs Vec RenderedGlyphMetrics
    ruby_boxes Vec RubyLayoutBox
    math_boxes Vec MathLayoutBox
```

Definitions:

- `logical_bounds` comes from shaped advances and inline layout.
- `ink_bounds` is the union of rasterized non-empty pixels after glyph transforms.
- `allocation_bounds` is the pixel area that must be available for rendering without clipping.
- `line_bounds` includes line gap and block progression.
- `baseline_set` includes alphabetic, ideographic, hanging, math axis, ruby annotation, and vertical baselines when available.
- `caret_positions` and `hit_test_map` are computed from resolved visual runs, not raw string byte order.

The layout engine may use `RenderedTextMetrics` to decide widget size, scroll extents, ruby overhang, line spacing, and collision with inline math. It must not infer these values from raw glyph count or unrendered string length.

## Shaping and layout

Text shaping pipeline:

```text
TextBuffer
    -> Unicode scalar stream
    -> script / language / bidi direction runs
    -> configured font substitution decision
    -> cmap glyph mapping
    -> GSUB substitutions
    -> GPOS positioning
    -> kerning
    -> ShapedRun
```

Font substitution must be explicit data. The shaping context uses the `FontSelectionRequest` / `FontSubstitutionPlan` defined in the resource section. If no configured font contains a glyph, shaping returns `FontError::MissingGlyph`. Default policy is `Disabled`.

Bidi and resolved run contract:

```text
TextDirection:
    LeftToRight
    RightToLeft

BidiClass:
    StrongLtr
    StrongRtl
    Weak
    Neutral
    ExplicitControl

ResolvedTextRun:
    source_range
    visual_order
    direction TextDirection
    script
    language
    shaped_run ShapedRun

ResolvedLine:
    logical_range
    visual_runs Vec ResolvedTextRun
    caret_stops Vec CaretStop
    hit_test_segments Vec HitTestSegment

CaretStop:
    source_offset
    visual_x
    visual_y
    affinity CaretAffinity

HitTestSegment:
    visual_bounds
    source_range
    direction TextDirection
```

`ResolvedLine` is the bridge between shaping and layout. Text fields, editors, ruby placement, and inline math must use this resolved visual structure. Raw string reversal is forbidden.

## Ruby and furigana

Ruby is a first-class inline object, not post-processing on already rendered pixels.

```text
RubyRun:
    base InlineText
    ruby InlineText
    position RubyPosition
    alignment RubyAlignment
    overhang RubyOverhangPolicy

RubyPosition:
    Over
    Under
    InterCharacter

RubyAlignment:
    Start
    Center
    SpaceAround
    SpaceBetween

RubyOverhangPolicy:
    None
    AllowAdjacentKana
    AllowPunctuation
```

Horizontal Japanese normally places furigana above the base. Vertical Japanese places ruby on the right side of the base column by default. Ruby layout must return metrics for both base and annotation:

```text
RubyLayoutBox:
    base_box InlineBox
    ruby_box InlineBox
    total_bounds
    baseline
    annotation_baseline
```

If a backend cannot render ruby, it returns `GuiError::Unsupported RubyLayout`. It must not draw ruby as plain inline text without explicit caller opt-in.

## Vertical writing

Writing mode is part of layout context.

```text
WritingMode:
    HorizontalTb
    HorizontalBt
    VerticalRl
    VerticalLr

TextOrientation:
    Mixed
    Upright
    Sideways
```

Vertical layout requirements:

- line progression follows `VerticalRl` / `VerticalLr`.
- Japanese kana / kanji are upright in `Mixed`.
- Latin and digits use `TextOrientation` policy.
- punctuation uses vertical alternates if `vert` / `vrt2` features are available.
- vertical metrics use `vhea` / `vmtx` / `VORG` when present.
- if required vertical substitutions or metrics are unavailable, formal mode returns typed unsupported error.

The renderer receives positioned glyphs; it does not decide writing mode.

Right-to-left and rightward horizontal requirements:

- `HorizontalTb` is the ordinary top-to-bottom line progression with inline direction determined by bidi runs.
- `HorizontalBt` is reserved for bottom-to-top line progression / right-side layout systems that need horizontal glyphs but reverse block progression.
- Arabic, Hebrew, and other right-to-left scripts are represented by bidi-resolved runs, not by reversing raw strings.
- Right-to-left layout must preserve caret, hit testing, selection, and ruby / math inline object order through resolved visual runs.
- Unsupported bidi class, shaping feature, or writing mode returns typed unsupported error.

## Math layout integration

Future math renderer is a NEPL library similar in role to KaTeX, but not dependent on KaTeX.js.

Math layout enters text as an inline object:

```text
InlineObject:
    Text ShapedRun
    Ruby RubyLayoutBox
    Math MathLayoutBox
    Image ImageLayoutBox

MathLayoutBox:
    width
    height
    axis
    ascent
    descent
    children
```

The text layout engine must support:

- baseline alignment between text and math axis
- inline and display math modes
- stretchy operators as a later extension
- font metric access for math italic correction and operator sizes
- deterministic snapshot tests for math boxes

Math renderer can request glyph outlines and metrics from the same font library. It must not duplicate font parsing or text measurement authority.

## Glyph paint and 2D renderer integration

Text rendering uses the same visual style model as the 2D rendering engine.
Formal 2D renderer contract は `doc/neplg2/gui_2d_rendering_design.md` を authority とし、この節は font engine から 2D renderer へ渡す glyph 側の boundary を定義する。

```text
GlyphPaint:
    fill Option Paint
    stroke Option Stroke
    shadows Vec Shadow
    blend_mode BlendMode

TextPaintMode:
    FillOnly
    StrokeOnly
    FillAndStroke
    ShadowOnly
    FillStrokeAndShadow

Shadow:
    offset_x
    offset_y
    blur_radius
    color
```

`GlyphPaint` is not a text-only color model. `Paint`、`Stroke`、`BlendMode`、`Transform2d`、`Clip` are shared with normal 2D rendering commands. This keeps text, vector shapes, ruby annotation, and math glyphs visually consistent.

Rendering order:

```text
shadow layers
fill
stroke
```

If `fill` is `Option::None` and `stroke` is present, the glyph is stroke-only. If `stroke` is `Option::None` and `fill` is present, it is fill-only. If both are present, the renderer draws both in the defined order. Shadow may be applied to fill, stroke, or both depending on `ShadowTarget`; initial implementation may support `ShadowTarget::WholeGlyph` only and return typed unsupported for finer modes.

```text
ShadowTarget:
    WholeGlyph
    FillOnly
    StrokeOnly
```

Glyph outline rasterization therefore has two mask outputs.

```text
GlyphFillMask:
    coverage mask for fill

GlyphStrokeMask:
    coverage mask for stroked outline
```

The 2D renderer and text renderer must share stroke join / cap / miter policy where applicable. If the glyph outline stroke mode is unsupported, return `FontError::UnsupportedGlyphStroke`; do not drop the stroke silently.

## Rendering pipeline

Formal text rendering pipeline:

```text
Font resource bytes
    -> GuiFontFace
    -> ScaledFont
    -> Text shaping
    -> Inline layout
    -> PositionedGlyph stream
    -> Glyph outline rasterizer
    -> Glyph bitmap cache
    -> PixelBuffer compositor
    -> SurfacePresenter
```

`PositionedGlyph`:

```text
PositionedGlyph:
    glyph_id
    font_face_id
    x
    y
    advance
    transform
    paint GlyphPaint
    subpixel_mode
```

Glyph cache key:

```text
GlyphCacheKey:
    font_content_hash
    face_index
    outline_kind
    glyph_id
    px_size
    variation_key
    transform
    hinting_mode
    subpixel_mode
```

Path, display name, CSS family name, or OS font handle are not cache authority.

`variation_key` is a normalized sorted axis-value list. Axis order in user input must not change cache identity.

## Errors

```text
FontError:
    ResourceMissing ResourcePath
    UnsupportedFontContainer
    UnsupportedCollectionFont
    InvalidSfntHeader
    InvalidCollectionHeader
    InvalidFaceIndex
    FaceIndexRequired
    MissingTable FontTableTag
    InvalidTableBounds FontTableTag
    UnsupportedFontFormat
    UnsupportedOutlineFormat
    UnsupportedWoffCompression
    MissingFamily FontFamilyName
    MissingStyle FontStyleDescriptor
    MissingGlyph UnicodeScalar
    SubstitutionDisabled
    UnsupportedGlyphSubstitution
    UnsupportedGlyphPositioning
    UnsupportedGlyphStroke
    UnsupportedGlyphShadow
    UnsupportedVariationAxis
    UnsupportedVerticalLayout
    UnsupportedBidiLayout
    UnsupportedWritingMode
    UnsupportedRubyLayout
    UnsupportedMathLayout
    RasterizationFailed
```

`FontError` maps to future `GuiError::FontError` / `GuiError::TextMeasureFailed` / `GuiError::ResourceMissing` variants after the core GUI error enum is extended. Until those variants exist, platform-facing code must map unsupported font behavior to existing `GuiError::Unsupported` and invalid font data to `GuiError::InvalidCommand` without losing the backend-specific `FontError` in logs or debug data. Human-readable display belongs to `std/gui/error_display`, not to the error itself.

## Test plan

Minimum tests:

```text
font resource tests
    HackGen font is available at fonts/HackGenConsoleNF-Regular.ttf through Web VFS
    another registered font can be selected by FontFamilyName without renderer edits
    FontSubstitutionPolicy::Disabled returns Result::Err on missing glyph
    explicit family alternatives are used only when policy allows them
    native resource resolver maps the same path
    offscreen test host loads the same fixture path
    headless formal font test requires explicit fixture resource
    missing resource returns ResourceMissing

sfnt parser tests
    container kind detection for ttf / otf / ttc / otc / woff / woff2
    collection face_index none / zero / negative / out-of-range behavior
    CFF and CFF2 outline kinds return typed unsupported until implemented
    WOFF and WOFF2 compression returns typed unsupported until decode capability is present
    table directory bounds
    required table presence
    cmap mapping for ASCII and Japanese sample code points
    horizontal metrics for selected glyphs
    composite glyph load
    variable axis detection for fvar
    explicit wght axis selection
    unsupported variation axis returns UnsupportedVariationAxis
    missing glyph typed error
    vertical metric table presence / absence handling

layout tests
    shaped run metrics use glyph advances
    rendered text metrics expose logical bounds, ink bounds, allocation bounds, baselines, caret positions, hit-test map
    ruby horizontal layout
    ruby vertical layout
    vertical Japanese layout with upright kana / kanji
    RTL resolved line uses visual runs instead of raw string reversal
    unsupported bidi / writing mode returns typed unsupported error
    unsupported glyph returns MissingGlyph
    unsupported vertical feature returns UnsupportedVerticalLayout

raster tests
    glyph outline to grayscale mask
    glyph stroke-only mask
    glyph fill-only mask
    glyph fill-and-stroke compositing
    glyph shadow compositing
    glyph mask compositing to Rgba8888 PixelBuffer
    deterministic pixel hash for sample text

math integration tests
    MathLayoutBox baseline alignment
    inline text + math + ruby ordering
```

## Implementation phases

Phase 1:

- Add font resource design docs and source policy tests.
- Register `web/src/fonts/HackGenConsoleNF-Regular.ttf` as `fonts/HackGenConsoleNF-Regular.ttf` in Web VFS.
- Add native/bare/headless resource contract stubs.

Phase 2:

- Implement SFNT single-face table directory, font container kind detection, outline kind detection, and core metrics parser.
- Expose `GuiFontFace`, `FontMetrics`, `GlyphMetrics`.
- Keep current bitmap renderer as temporary renderer until outline rasterizer is ready.

Phase 3:

- Implement `cmap` glyph mapping, `glyf` outline load, simple outline rasterizer.
- Render ASCII and Japanese sample glyphs to grayscale masks.

Phase 3.5:

- Add TTC / OTC collection face index parsing.
- Keep collection face selection explicit in `FontResourceRequest`.
- Return typed unsupported errors for unavailable face index or unsupported outline kind.

Phase 4:

- Implement shaped run with kerning and initial GSUB / GPOS support.
- Replace fixed-cell `TextMeasurer` for GUI with font metrics based measurer.

Phase 5:

- Implement ruby layout and vertical writing.
- Add tests for Japanese horizontal / vertical text and furigana.

Phase 5.5:

- Add OpenType CFF / CFF2 outline parser behind the same glyph outline interface.
- Add WOFF / WOFF2 container support if compression dependencies are available through an explicit resource decode capability.
- Add variable font `fvar` axis parsing and `wght` selection for explicit bold weight requests.

Phase 6:

- Implement math inline object interface and baseline alignment.
- Keep math expression parser / TeX-like front-end as a separate later library, but use this font metrics / glyph engine.

Phase 7:

- Connect font renderer to Web bitmap pixel surface, native framebuffer presenter, offscreen screenshots, and headless typed unsupported behavior.

## Open questions

- Whether first outline rasterizer uses grayscale only or also subpixel LCD masks.
- Whether CFF / OpenType CFF support is needed before non-HackGen fonts.
- How much GSUB / GPOS coverage is required before formal Japanese vertical mode leaves unsupported state.
- Whether ruby overhang defaults should follow JIS X 4051 strictly or expose a smaller portable policy first.
