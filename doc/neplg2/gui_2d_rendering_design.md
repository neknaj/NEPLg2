# NEPLg2 GUI 2D rendering engine design

作成日: 2026-06-13

## 目的

この文書は、GUI の 2D rendering engine を font rendering engine と同時に設計するための仕様と実装計画である。

GUI の見た目は rectangle、path、stroke、fill、shadow、image、SVG、text glyph、ruby、math glyph、button skin などの合成で作られる。Font engine が glyph の metrics / outline / mask を作り、2D renderer がそれを他の図形と同じ `Paint`、`Stroke`、`BlendMode`、`Clip`、`Transform2d` で描画する。この境界を分けることで、font renderer と UI renderer が別々の色、stroke、shadow、clip 規則を持つ不整合を防ぐ。

この文書は `doc/neplg2/gui_font_rendering_design.md` と対になる。Font engine は文字 shaping、glyph positioning、glyph outline / mask generation、text metrics を担当する。2D renderer は path / image / glyph mask / surface compositing を担当する。

## 設計原則

- `core/gui` は DOM、Canvas2D object、SVG DOM、Skia、Direct2D、CoreGraphics、GPU API、OS font handle を知らない。
- Web visible canvas は pixel buffer を `putImageData` で表示するだけであり、正式 renderer は Canvas2D primitive や browser SVG renderer を authority にしない。
- 2D renderer は `Result` を返す。Unsupported SVG feature、unsupported blend mode、unsupported color space、unsupported glyph stroke、missing image resource は代替描画せず typed error にする。
- Path、glyph、image、UI skin、math layout は同じ paint model を共有する。Text 専用の色 model や SVG 専用の shadow model を作らない。
- Layout に使う bounds は描画後の ink / allocation / dirty region と同じ入力から計算する。描画と測定で別 engine を使わない。
- Offscreen、headless、native、bare、Web は同じ command / pixel buffer contract を使う。Headless は surface を持たないため present は `GuiError::Unsupported` を返す。

## 層構造

```text
core/gui/render2d
    paint
    stroke
    path command
    blend mode
    clip
    transform
    render command type
    render error type

alloc/gui/render2d
    path builder
    display list
    software rasterizer
    mask rasterizer
    svg document model
    image reference
    render cache key

alloc/gui/font
    shaped run
    positioned glyph
    glyph outline
    glyph fill/stroke mask
    rendered text metrics

std/gui/resource
    image bytes
    SVG bytes
    font bytes
    VFS / filesystem resource root

platforms/gui/web
    pixel buffer presenter
    VFS resource provider

platforms/gui/native
    framebuffer presenter
    resource provider

platforms/gui/bare
    device pixel surface
    optional flush target

platforms/gui/headless
    virtual event source
    no surface presentation
```

依存方向は renderer command / software rasterizer / presenter の順である。Presenter は renderer を呼んでよいが、renderer は presenter や platform API を呼ばない。

## Shared visual style model

2D renderer と font renderer は次の型を共有する。

```text
Paint:
    Solid Rgba8888
    LinearGradient Gradient
    RadialGradient Gradient
    ImagePattern ImagePattern

Stroke:
    width LogicalPx
    join StrokeJoin
    cap StrokeCap
    miter_limit f32
    dash Option StrokeDash
    paint Paint

Shadow:
    offset GuiVector
    blur_radius LogicalPx
    spread LogicalPx
    paint Paint
    target ShadowTarget

BlendMode:
    SourceOver
    Copy
    Multiply
    Screen
    Overlay

RenderStyle:
    fill Option Paint
    stroke Option Stroke
    shadows Vec Shadow
    blend BlendMode
    opacity f32
```

初期実装は `Solid`、`SourceOver`、矩形、bitmap text mask だけから始めてよい。ただし型と error contract は後から path / SVG / outline glyph / shadow を追加しても変更不要な形にする。

core no_alloc 実装では、この shared stroke model を `GuiStroke`、`GuiStrokeCap`、`GuiStrokeJoin`、`GuiStrokeDash` として表す。`GuiStrokeDash::Solid` は dash なしの明示 policy であり、未対応 dash pattern を solid に近似する fallback ではない。`miter_limit` は shared design と同じ `f32` として保持し、raw integer scale へ暗黙変換しない。

`fill` が `Option::None` で `stroke` がある場合は stroke-only、`stroke` が `Option::None` で `fill` がある場合は fill-only、両方ある場合は定義された paint order で両方を描く。どちらもない場合は invalid command として拒否する。

## Render commands

Formal command は platform 非依存である。

```text
Render2dCommand:
    FillRect GuiRect Paint
    StrokeRect GuiRect Stroke
    FillPath PathId Paint FillRule
    StrokePath PathId Stroke
    DrawImage ImageId GuiRect ImageSampling
    DrawGlyphRun GlyphRunId GlyphPaint
    DrawMask MaskId GuiPoint Paint
    PushClip Clip
    PopClip
    PushTransform Transform2d
    PopTransform
    PushLayer LayerConfig
    PopLayer BlendMode
```

Widget layer は button、checkbox、slider などをこの command stream へ展開する。Button の見た目は DOM button ではなく、rounded rect、border stroke、text glyph run、focus ring、pressed/hover state の render command として表す。

SVG は browser SVG renderer へ渡さない。SVG parser は `SvgDocument` を作り、対応済み subset を `Render2dCommand` または `Path` へ変換する。未対応 feature は `Render2dError::UnsupportedSvgFeature` として返す。

## Path and rasterization

Path は次の command を持つ。

```text
PathCommand:
    MoveTo GuiPoint
    LineTo GuiPoint
    QuadTo GuiPoint GuiPoint
    CubicTo GuiPoint GuiPoint GuiPoint
    Close
```

Fill rule:

```text
FillRule:
    NonZero
    EvenOdd
```

Rasterizer は path を scanline / coverage mask へ変換する。Anti-aliasing は coverage alpha として扱い、pixel buffer へ source-over 合成する。Anti-aliasing が無効な target は capability で表し、暗黙の nearest approximation へ落とさない。

Stroke expansion は path geometry の責務であり、join / cap / miter / dash を `Stroke` から計算する。Unsupported dash pattern や miter overflow は typed error にする。

## Font engine integration

Font engine は次を返す。

```text
ShapedRun
PositionedGlyph
GlyphOutline
GlyphFillMask
GlyphStrokeMask
RenderedTextMetrics
```

2D renderer は `PositionedGlyph` を受け取り、`GlyphPaint` を `RenderStyle` と同じ規則で解釈する。

```text
GlyphPaint:
    fill Option Paint
    stroke Option Stroke
    shadows Vec Shadow
    blend BlendMode
```

Layout engine は `RenderedTextMetrics` の `logical_bounds`、`ink_bounds`、`allocation_bounds`、`line_bounds`、`caret_positions`、`hit_test_map` を使う。Renderer が実際に描く glyph mask と別の簡易測定値を使って widget size を決めてはいけない。

Ruby / furigana は inline object として layout される。Ruby text も通常 text と同じ glyph pipeline で metrics と masks を作り、2D renderer が合成する。縦書き、右横書き、variable font axis、math glyph も `PositionedGlyph` と `RenderedTextMetrics` へ反映してから 2D renderer に渡す。

## SVG and image integration

SVG support は段階的に行う。

```text
SvgParseResult:
    Ok SvgDocument
    Err SvgError

SvgRenderPlan:
    commands Vec Render2dCommand
    required_features Vec SvgFeature
```

初期 subset:

- `svg` viewport
- `path`
- `rect`
- `circle`
- solid fill
- solid stroke
- group transform
- clip path

後続 subset:

- gradient
- pattern
- mask
- filter
- text element

未対応要素は描画から消さない。`UnsupportedSvgFeature` として caller に返し、UI は明示的に error state を表示するか、別 resource を選ぶ。

Image は decoded pixel buffer として renderer に入る。PNG、JPEG、WebP、AVIF などの decode は `std/gui/resource` または platform resource provider の責務であり、`core/gui` は encoded format を知らない。Color format 変換は `ColorConvert` contract に従い、unsupported color space は typed error にする。

## Surface and video memory integration

2D renderer の出力は `PixelBuffer` である。

```text
PixelBuffer:
    width
    height
    stride_bytes
    format
    pixels

Render2dOutput:
    pixel_buffer PixelBuffer
    dirty_region DirtyRegion
    metrics RenderMetrics
```

Web backend は `PixelBuffer` を video memory slot に書き、presenter は `ImageData` を作って `putImageData` する。Native backend は window framebuffer へ present する。Bare backend は device framebuffer と optional `FlushTarget` を使う。Offscreen backend は snapshot を返す。Headless backend は layout / update / event replay だけを実行し、present は unsupported error にする。

Single shared pixel plane は禁止する。Writer と presenter は slot ownership state を使い、`Free -> Writing -> Published -> Reading -> Free` で移る。

## UI skin integration

Widget は semantic tree と visual command tree を分ける。

```text
ButtonVisualState:
    Normal
    Hovered
    Pressed
    Disabled
    Focused

ButtonSkin:
    background RenderStyle
    border Stroke
    focus_ring Stroke
    label_text TextStyle
```

Button は `ButtonSkin` と `ButtonVisualState` から render commands を作る。Action は `ActionId` として event layer に残し、visual command は behavior を持たない。Accessibility label は semantic tree へ出し、draw command から復元しない。

## Error model

```text
Render2dError:
    InvalidGeometry
    UnsupportedBlendMode
    UnsupportedPaint
    UnsupportedStroke
    UnsupportedSvgFeature SvgFeature
    UnsupportedImageFormat
    UnsupportedColorSpace
    UnsupportedGlyphPaint
    MissingPath PathId
    MissingImage ImageId
    MissingGlyphRun GlyphRunId
    ClipStackUnderflow
    TransformStackUnderflow
    ResourceMissing ResourceId
```

Error 自体は enum data として持つ。Human-readable display、色付け、localized message は `std/gui/error_display` で扱う。

## Cache and performance

Cache key は content と render parameter から作る。

```text
PathCacheKey:
    path_content_hash
    transform_quantized
    fill_rule

GlyphCacheKey:
    font_content_hash
    face_index
    variation_key
    glyph_id
    size
    stroke_key
    hinting_mode

ImageCacheKey:
    resource_content_hash
    decode_options
    color_space
```

File path、CSS family name、OS handle、display name は cache authority にしない。Resource bytes の content hash と明示的な render parameter を authority にする。

Renderer は display list を線形走査する。Clip stack と transform stack は bounded stack とし、overflow は typed error にする。Dirty region は command bounds と ink bounds から計算し、全画面 redraw を標準 contract にしない。

## Implementation phases

Phase 1:

- Web preview の existing bitmap buffer / `putImageData` path を正式な software rasterizer slice に寄せる。
- `FillRect`、`rgba-row`、bitmap font text mask、dirty full / rect を扱う。
- Unsupported text scalar、unsupported command、invalid geometry を typed error にする。

Phase 2:

- `RenderStyle`、`Paint`、`Stroke`、`BlendMode`、`Shadow` の shared type を `core/gui` 側の設計へ反映する。
- Button / checkbox / slider skin を render command へ展開する。

Phase 3:

- Path builder と fill rasterizer を追加する。
- Stroke expansion、clip stack、transform stack を追加する。

Phase 4:

- Font renderer の glyph outline / mask と接続する。
- `RenderedTextMetrics` から layout、dirty region、hit-test を決める。

Phase 5:

- SVG parser subset と SVG-to-render-command lowering を追加する。
- Unsupported SVG feature matrix を test 化する。

Phase 6:

- Offscreen screenshot、headless event replay、native framebuffer presenter、bare framebuffer presenter を同じ output contract へ接続する。

## Tests

必要な test:

- `RenderStyle` の fill-only / stroke-only / fill-and-stroke / shadow の contract test。
- Invalid geometry、empty style、clip stack underflow、transform stack underflow の typed error test。
- Web visible canvas に `fillText`、`fillRect`、`stroke`、`drawImage` が無い source policy test。
- Unsupported glyph scalar が replacement glyph にならず typed error を返す test。
- SVG unsupported feature が消えるのではなく typed error を返す test。
- Button visual state が DOM button ではなく render commands へ展開される test。
- Offscreen snapshot が pixel hash / dimensions / format を返す test。
- Headless present が `Unsupported` を返し、update / layout / event replay は実行できる test。

## Current implementation note

2026-06-13 時点の Web preview implementation は formal 2D engine の最初の slice である。`fill-rect`、`rgba-row`、ASCII bitmap font text-run だけを software bitmap buffer へ rasterize し、visible canvas へは `putImageData` だけで表示する。Path、SVG、outline font、shadow、gradient、image decode、math rendering は未実装だが、この文書の command / error / style contract に沿って追加する。
