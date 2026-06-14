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

Web では bundled resource manifest が canonical resource path `fonts/HackGenConsoleNF-Regular.ttf` と license text を登録し、VFS 内部では `/fonts/HackGenConsoleNF-Regular.ttf` と `/fonts/HackGen-LICENSE.txt` に mount する。`/fonts/...` は VFS transport の表現であり、font identity や lookup authority にはしない。Web Playground は startup で mount promise を開始し、`neplg2 run` / Wasm execution の直前に完了を待つ。Mount に失敗した場合は `GuiFontResourceMountError` の typed variant を terminal に表示し、execution は開始しない。Compile-only path は runtime font bytes を必要としないため mount 完了を待たない。

Native では packaged resource directory または configured resource root から同じ canonical path を読む。Native provider は `fonts/...` を root-relative path として扱い、OS font family lookup や current working directory の suffix scan へ逃がさない。Resource root が未設定、または path が存在しない場合は missing resource error を返す。

Bare では embedded blob table を provider とする。Blob table が未設定の環境では filesystem probing を行わず unsupported を返す。Blob table に canonical path が存在しない場合は missing resource error を返す。Headless / offscreen tests は explicit fixture resource provider を渡し、host font API には依存しない。

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

## SFNT name table

F4b は `alloc/gui/font/sfnt/name.nepl` が所有する。`alloc/gui/font/sfnt.nepl` は facade とし、numeric metadata parser は `alloc/gui/font/sfnt/metadata.nepl` に置く。`gui_sfnt_parse_metadata` は `name` table decode を行わず、`gui_sfnt_parse_names` は別 API として `GuiSfntNames` を返す。

```text
GuiSfntNames:
    family Option str
    subfamily Option str
    full_name Option str

GuiSfntNameEncodingKind:
    WindowsUnicodeBmpAscii
    MacintoshRomanAscii
```

Representative name selection is deterministic per nameID:

```text
nameID 1: family
nameID 2: subfamily
nameID 4: full name

rank 400:
    platformID 3
    encodingID 1
    languageID 0x0409
    decode as UTF-16BE ASCII subset

rank 300:
    platformID 3
    other encoding or language
    selected only when rank 400 is absent, then UnsupportedNameEncoding

rank 200:
    platformID 1
    encodingID 0
    languageID 0
    decode as Roman ASCII subset

rank 100:
    platformID 1
    other encoding or language
    selected only when higher ranks are absent, then UnsupportedNameEncoding
```

Other platform IDs are not representative candidates in F4b. This is not a substitute-font mechanism; it is a narrow metadata extraction rule. If no candidate exists for a representative nameID, the field is `Option::None`. If a higher-ranked candidate exists but cannot be decoded by the initial engine, parsing returns typed error rather than silently taking a lower-ranked candidate.

Name table validation:

- table format must be 0; otherwise `UnsupportedNameTableFormat`.
- `count * 12`, record area, and string storage offset must stay inside the `name` table.
- record string ranges are relative to `stringOffset`, not to the file start.
- UTF-16BE representative strings must have even byte length.
- selected representative strings must be non-empty.
- ASCII subset means all decoded scalar values are in byte range `0..127`.

## SFNT cmap table

F4c は `alloc/gui/font/sfnt/cmap.nepl` が所有する。`gui_sfnt_parse_metadata` は `cmap` table の有無を directory summary に記録するだけで、glyph lookup を行わない。`gui_sfnt_lookup_glyph_id` は別 API として font bytes、face index、Unicode code point を受け取り、`GuiGlyphId` を返す。

```text
GuiSfntCmapSubtableRecord:
    platform_id i32
    encoding_id i32
    offset i32

GuiSfntCmapEncodingKind:
    WindowsUnicodeBmpFormat4
```

Subtable selection は次だけを許す。

```text
selected:
    first record where platformID == 3 and encodingID == 1

absent:
    UnsupportedCmapEncoding

selected format != 4:
    UnsupportedCmapTableFormat
```

Platform 0、platform 3 encoding 10、Macintosh record、後続 record は F4c の代替 candidate ではない。format 4 が対応できない code point は `UnsupportedCmapEncoding`、format 4 の BMP 範囲内だが glyph mapping が存在しない場合は `MissingGlyphMapping` である。

Format 4 lookup は OpenType table layout の byte offsets から直接行う。

```text
format                  u16 offset 0
length                  u16 offset 2
language                u16 offset 4
segCountX2              u16 offset 6
searchRange             u16 offset 8
entrySelector           u16 offset 10
rangeShift              u16 offset 12
endCode[segCount]       offset 14
reservedPad             after endCode
startCode[segCount]
idDelta[segCount]
idRangeOffset[segCount]
glyphIdArray[]
```

Validation rules:

- `length` and `language` are readable.
- `length` is at least `16 + 8 * segCount` and remains inside the selected `cmap` table.
- selected subtable offset is not inside the encoding record array.
- `segCountX2` is even and greater than 0.
- `reservedPad` is 0.
- every segment array range is inside the subtable.
- `idRangeOffset == 0` uses `(code_point + idDelta) mod 65536`.
- `idRangeOffset != 0` computes glyph array address from the address of that idRangeOffset word, then adds `2 * (code_point - startCode)`.
- the computed glyph array address must remain inside the subtable.
- raw glyph 0, computed glyph 0, and no matching segment are `MissingGlyphMapping`.

## SFNT horizontal metrics table

F4d は `alloc/gui/font/sfnt/hmtx.nepl` が所有する。`gui_sfnt_parse_metadata` は `hmtx` table の有無を directory summary に記録するだけで、horizontal metric lookup を行わない。`gui_sfnt_lookup_horizontal_metric` は別 API として font bytes、face index、checked `GuiGlyphId` を受け取り、`GuiSfntHorizontalMetric` を返す。

```text
GuiSfntHorizontalMetric:
    glyph GuiGlyphId
    advance_width i32
    left_side_bearing i32
```

`numberOfHMetrics` は `hhea` table offset 34 の u16 である。F4a metadata parser は line metrics のために `hhea.length >= 10` だけを要求するが、F4d metric lookup は `numberOfHMetrics` を読むので `hhea.length >= 36` を要求する。

Validation rules:

- `hmtx` table must exist; otherwise `MissingTable`.
- `hhea.length >= 36` must hold for this lookup; otherwise `MalformedHmtxRecord`.
- `numberOfHMetrics > 0`.
- `numberOfHMetrics <= maxp.numGlyphs`.
- valid public metric lookup range is `1 <= glyphRaw < numGlyphs`; glyph 0 is not a successful renderable glyph in the GUI font contract.
- required declared `hmtx.length` is `numberOfHMetrics * 4 + (numGlyphs - numberOfHMetrics) * 2`.
- all reads use `hmtx.offset + table_relative_offset`, and each relative range must stay inside declared `hmtx.length`.

Lookup layout:

```text
longHorMetric[numberOfHMetrics]:
    advanceWidth u16
    leftSideBearing i16

leftSideBearing[numGlyphs - numberOfHMetrics]:
    i16
```

If `glyphRaw < numberOfHMetrics`, the metric offset is `glyphRaw * 4`. If `glyphRaw >= numberOfHMetrics`, advance width is read from `(numberOfHMetrics - 1) * 4`, and left side bearing is read from `numberOfHMetrics * 4 + (glyphRaw - numberOfHMetrics) * 2`.

`hmtx` does not provide ink bounds or outline geometry. F4d therefore returns `GuiSfntHorizontalMetric` rather than pretending to produce full `GuiGlyphMetrics`. Conversion to `GuiGlyphMetrics` happens after outline / bitmap bounds are available.

## SFNT glyph header bounds

F4e は `alloc/gui/font/sfnt/glyf.nepl` が所有する。`gui_sfnt_parse_metadata` は `loca` / `glyf` table の有無を directory summary に記録するだけで、glyph bounds lookup を行わない。`gui_sfnt_lookup_glyph_bounds` は別 API として font bytes、face index、checked `GuiGlyphId` を受け取り、`GuiSfntGlyphBounds` を返す。

```text
GuiSfntGlyphBounds:
    glyph GuiGlyphId
    x_min i32
    y_min i32
    x_max i32
    y_max i32
```

`indexToLocFormat` は `head` table offset 50 の i16 である。F4a metadata parser は `unitsPerEm` のために `head.length >= 20` だけを要求するが、F4e glyph bounds lookup は `indexToLocFormat` を読むので `head.length >= 52` を要求する。この要件を metadata parser へ持ち込んではならない。

Validation rules:

- `loca` / `glyf` table must exist; otherwise `MissingTable`.
- `head.length >= 52` must hold for this lookup; otherwise `MalformedGlyfRecord`.
- `indexToLocFormat == 0` uses short offsets: `loca[glyph]` and `loca[glyph + 1]` are u16 values multiplied by 2.
- `indexToLocFormat == 1` uses long offsets: `loca[glyph]` and `loca[glyph + 1]` are u32 values constrained to i32 range.
- other `indexToLocFormat` values are `UnsupportedLocaFormat`.
- required declared `loca.length` is `(numGlyphs + 1) * 2` for format 0 and `(numGlyphs + 1) * 4` for format 1.
- valid public glyph bounds lookup range is `1 <= glyphRaw < numGlyphs`; glyph 0 is not a successful renderable glyph in the GUI font contract.
- glyph offset pair must satisfy `start <= end <= glyf.length`.
- `start == end` is `MissingGlyphOutline`.
- `end - start < 10`, inverted x bounds, and inverted y bounds are `MalformedGlyfRecord`.
- all glyph reads use `glyf.offset + table_relative_offset`, and each relative range must stay inside declared `glyf.length`.

Lookup layout:

```text
loca format 0:
    offset[numGlyphs + 1] u16, actual offset = value * 2

loca format 1:
    offset[numGlyphs + 1] u32

glyf glyph header:
    numberOfContours i16
    xMin i16
    yMin i16
    xMax i16
    yMax i16
```

F4e reads `numberOfContours` only as part of the required 10 byte header. Simple glyph contour arrays, composite component recursion, CFF / CFF2 charstrings, fill / stroke rasterization, and `GuiGlyphMetrics` synthesis are later phases.

### SFNT simple glyph topology

F4f extends `alloc/gui/font/sfnt/glyf.nepl` with simple glyph topology. This remains in `glyf.nepl` because it shares the same `head` / `loca` / `glyf` table lookup and declared range validation as F4e. A later point decoder can split into `glyf/simple.nepl` or `outline.nepl` when flags, coordinates, and composite recursion grow beyond header topology.

```text
GuiSfntSimpleGlyphTopology:
    glyph GuiGlyphId
    bounds GuiSfntGlyphBounds
    contour_count i32
    point_count i32
    instruction_length i32
    point_data_offset i32
    point_data_length i32
```

`point_data_offset` is relative to the `glyf` table, not absolute file offset. The file offset is `glyf.offset + point_data_offset`. Keeping topology ranges table-relative lets headless, native, web, and bare providers share the same checked payload without path or host font authority.

Simple glyph layout:

```text
glyf glyph header:
    numberOfContours i16
    xMin i16
    yMin i16
    xMax i16
    yMax i16

simple glyph payload:
    endPtsOfContours[numberOfContours] u16
    instructionLength u16
    instructions[instructionLength] u8
    flags and coordinate stream
```

Validation:

- F4e `loca` / `glyf` range and bounds validation must pass first.
- `numberOfContours < 0` is `UnsupportedGlyphOutlineFormat`.
- `numberOfContours == 0` is `MissingGlyphOutline`.
- endpoint array range and `instructionLength` range must remain inside the selected glyph range.
- endpoints must be strict increasing; `point_count = last_endpoint + 1`.
- `point_count <= 0` or overflow is `MalformedGlyfRecord`.
- `point_data_offset = instruction_start + instructionLength`, `point_data_length = glyph_end - point_data_offset`.
- `numberOfContours > 0` and `point_count > 0` with zero point data length is `MalformedGlyfRecord`.

F4f deliberately does not parse flags or x/y deltas. That means it only proves the topology prefix and non-empty point stream boundary. Later outline phases must validate repeat flags, coordinate stream length, contour closure, and composite recursion.

### SFNT simple glyph point stream

F4g adds a point stream range decoder without constructing point values. The decoder consumes the raw flags stream, expands repeat counts only for counting, and derives the x/y coordinate byte ranges.

```text
GuiSfntSimpleGlyphPointStream:
    topology GuiSfntSimpleGlyphTopology
    flag_data_offset i32
    flag_data_length i32
    x_data_offset i32
    x_data_length i32
    y_data_offset i32
    y_data_length i32
    trailing_data_offset i32
    trailing_data_length i32
```

All offsets are relative to the `glyf` table. `flag_data_offset` equals `topology.point_data_offset`. `flag_data_length` is the raw consumed flag stream length, including repeat-count bytes; it is not the expanded logical point count.

Flag scan state:

```text
logical_point_count
raw_flag_cursor
x_coordinate_byte_count
y_coordinate_byte_count
```

Repeat semantics:

- A flag byte always contributes one logical point.
- If repeat bit 3 is set, the next byte is an additional repeat count.
- The total run is `1 + repeat_count`.
- `repeat_count = 0` is valid and means no additional logical point.
- A run that crosses `point_count` is malformed.
- Missing repeat count byte is malformed.

Coordinate byte length is derived without decoding values:

```text
xShort == 1:
    x bytes = 1
xShort == 0 and xSame == 1:
    x bytes = 0
xShort == 0 and xSame == 0:
    x bytes = 2

yShort == 1:
    y bytes = 1
yShort == 0 and ySame == 1:
    y bytes = 0
yShort == 0 and ySame == 0:
    y bytes = 2
```

When short is set, the same/positive bit controls sign, not byte length. F4g therefore records byte ranges only. Actual delta sign and cumulative coordinate reconstruction are F4h responsibilities.

Range derivation:

```text
flag_data_offset = topology.point_data_offset
x_data_offset = flag_data_offset + flag_data_length
y_data_offset = x_data_offset + x_data_length
trailing_data_offset = y_data_offset + y_data_length
trailing_data_length = glyph_end - trailing_data_offset
```

`trailing_data_length < 0` is `MalformedGlyfRecord`. Non-negative trailing data is returned explicitly, not treated as hidden fallback. Later sanitizer or outline decode phases can decide whether non-zero padding is accepted.

### SFNT simple glyph single point decode

F4h decodes one logical point from an already checked `GuiSfntSimpleGlyphPointStream`. It intentionally does not allocate a point `Vec`; the full outline builder is deferred until allocation failure and owner recovery are specified.

```text
GuiSfntSimpleGlyphPoint:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
    on_curve bool
    end_of_contour bool
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_point:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32
    -> Result GuiSfntSimpleGlyphPoint GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_point_stream_with_tables
    -> validate point_index against topology.point_count
    -> decode flags and coordinate deltas only inside stream ranges
    -> read endpoint array through topology-derived endpoint offset
    -> return GuiSfntSimpleGlyphPoint
```

`point_index < 0` and `point_index >= point_count` return `MissingGlyphOutline`. These conditions are invalid public point requests, not malformed bytes.

Malformed byte structure remains `MalformedGlyfRecord`:

- repeat byte required by a flag is outside `flag_data`
- x/y coordinate read is outside F4g-derived x/y range
- endpoint array read fails despite topology validation
- internal offset arithmetic is inconsistent

Cursor semantics:

```text
flag_cursor = flag_data_offset
x_cursor = x_data_offset
y_cursor = y_data_offset
logical_index = 0
current_x = 0
current_y = 0
```

Each logical point consumes its flag first, then consumes x/y coordinate bytes according to that flag, applies deltas cumulatively, and only then compares `logical_index` with target. When target lies inside a repeated flag run, all earlier repeated points in that same run still consume coordinate bytes and update cumulative coordinates.

Delta formula:

```text
xShort and xPositive: +u8
xShort and not xPositive: -u8
not xShort and xSame: 0
not xShort and not xSame: i16be

yShort and yPositive: +u8
yShort and not yPositive: -u8
not yShort and ySame: 0
not yShort and not ySame: i16be
```

`on_curve` is flag bit 0. `end_of_contour` is true if `point_index` equals any endpoint value. The endpoint array offset is derived from topology:

```text
endpoint_array_length = contour_count * 2
endpoint_array_offset = point_data_offset - instruction_length - 2 - endpoint_array_length
```

F4h ignores `trailing_data_length` except that F4g already proved it is non-negative. It must not consume trailing bytes, require zero padding, call host font APIs, or use fixed-cell fallback.

### SFNT simple glyph contour span lookup

F4i returns one contour's inclusive logical point index span from checked simple glyph topology. It is a no-allocation boundary before the future outline builder, curve segment builder, and mask rasterizer.

```text
GuiSfntSimpleGlyphContourSpan:
    glyph GuiGlyphId
    contour_index i32
    start_point_index i32
    end_point_index i32
    point_count i32
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_contour_span:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32
    -> Result GuiSfntSimpleGlyphContourSpan GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_topology_with_tables
    -> validate contour_index against topology.contour_count
    -> read current endpoint from topology-derived endpoint array
    -> read previous endpoint when contour_index > 0
    -> return GuiSfntSimpleGlyphContourSpan
```

F4i deliberately depends on F4f topology validation only. It must not call `gui_sfnt_glyf_simple_point_stream_with_tables`, `gui_sfnt_lookup_simple_glyph_point_stream`, or `gui_sfnt_lookup_simple_glyph_point`.

Endpoint semantics:

```text
endpoint_array_length = contour_count * 2
endpoint_array_offset = point_data_offset - instruction_length - 2 - endpoint_array_length

previous_endpoint = -1 when contour_index == 0
previous_endpoint = endPtsOfContours[contour_index - 1] otherwise
end_point_index = endPtsOfContours[contour_index]
start_point_index = previous_endpoint + 1
point_count = end_point_index - start_point_index + 1
```

`end_point_index` is inclusive. `contour_index < 0` and `contour_index >= contour_count` return `MissingGlyphOutline`; malformed endpoint data observed through topology validation or endpoint reads returns `MalformedGlyfRecord`.

### SFNT simple glyph contour point lookup

F4j composes the F4i contour span and F4h single point decode. It is the first contour-local coordinate query, but it still does not allocate a point vector, contour vector, curve segment list, or raster mask.

```text
GuiSfntSimpleGlyphContourPoint:
    span GuiSfntSimpleGlyphContourSpan
    contour_point_index i32
    point GuiSfntSimpleGlyphPoint
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_contour_point:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32 -> i32
    -> Result GuiSfntSimpleGlyphContourPoint GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_contour_span_with_tables
    -> validate contour-local contour_point_index against span.point_count
    -> absolute_point_index = span.start_point_index + contour_point_index
    -> gui_sfnt_glyf_simple_point_with_tables
    -> return GuiSfntSimpleGlyphContourPoint
```

`contour_point_index` is contour-local. The nested `point.point_index` is absolute within the glyph and must equal `span.start_point_index + contour_point_index`.

The implementation must validate local point range before calling point decode. Otherwise a local out-of-range request could be hidden behind a coordinate stream error from a malformed point array. For a valid contour span, `contour_point_index < 0` and `contour_point_index >= span.point_count` return `MissingGlyphOutline`.

F4j uses internal table helpers, not public wrappers, after metadata is parsed:

```text
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_glyf_simple_point_with_tables
```

This keeps table unwrap single-pass at the public boundary. F4j must not call platform font APIs, host text measurement, full outline builders, or hidden substitution paths.

### SFNT simple glyph contour edge lookup

F4k composes the F4i contour span and F4j contour-local point lookup into a topology point pair. The result is not a drawable line segment. It does not classify on-curve/off-curve combinations, insert implied on-curve points, build quadratic segments, decide winding, or rasterize pixels.

```text
GuiSfntSimpleGlyphContourEdge:
    start GuiSfntSimpleGlyphContourPoint
    end GuiSfntSimpleGlyphContourPoint
    edge_index i32
    next_contour_point_index i32
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_contour_edge:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32 -> i32
    -> Result GuiSfntSimpleGlyphContourEdge GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_contour_span_with_tables
    -> validate edge_index against span.point_count
    -> next_contour_point_index = wrap(edge_index + 1, span.point_count)
    -> gui_sfnt_glyf_simple_contour_point_with_tables for start
    -> gui_sfnt_glyf_simple_contour_point_with_tables for end
    -> return GuiSfntSimpleGlyphContourEdge
```

`edge_index` is contour-local and must equal `start.contour_point_index`. `next_contour_point_index` must equal `end.contour_point_index`. The nested `start.point.point_index` and `end.point.point_index` remain absolute within the glyph.

The implementation must validate `edge_index` before decoding either endpoint. This keeps the F4k public contract stable: a valid glyph with a missing requested edge returns `MissingGlyphOutline` instead of exposing a later point decode or coordinate error.

One-point contours return an explicit self-wrapping topology edge:

```text
edge_index = 0
next_contour_point_index = 0
start.contour_point_index = 0
end.contour_point_index = 0
start.point.point_index = end.point.point_index
```

This only preserves contour topology. The later segment builder decides whether the self-wrap contributes a visible segment or an unsupported outline condition.

F4k uses internal table helpers, not public wrappers, after metadata is parsed:

```text
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
```

F4k must not allocate `Vec GuiSfntSimpleGlyphContourEdge`, call platform font APIs, use host text measurement, build full outlines, or substitute another rendering path.

### SFNT simple glyph curve segment classification

F4l is the first drawable-shape classification layer above contour topology. It still does not own an outline, allocate a segment list, decide winding, rasterize masks, or call platform text APIs. It classifies exactly one edge start.

The type model uses enum payloads instead of a shared struct with inactive fields:

```text
GuiSfntSimpleGlyphCurveNoSegmentReason:
    SinglePointContour
    OffCurveStart
    MissingLookahead

GuiSfntSimpleGlyphCurveNoSegment:
    edge GuiSfntSimpleGlyphContourEdge
    reason GuiSfntSimpleGlyphCurveNoSegmentReason

GuiSfntSimpleGlyphLineSegment:
    edge GuiSfntSimpleGlyphContourEdge
    start_x2 i32
    start_y2 i32
    end_x2 i32
    end_y2 i32

GuiSfntSimpleGlyphQuadraticSegment:
    edge GuiSfntSimpleGlyphContourEdge
    lookahead GuiSfntSimpleGlyphContourPoint
    start_x2 i32
    start_y2 i32
    control_x2 i32
    control_y2 i32
    end_x2 i32
    end_y2 i32
    end_is_implied bool

GuiSfntSimpleGlyphCurveSegment:
    NoSegment GuiSfntSimpleGlyphCurveNoSegment
    Line GuiSfntSimpleGlyphLineSegment
    Quadratic GuiSfntSimpleGlyphQuadraticSegment
```

Coordinate fields are doubled font units:

```text
point coordinate:
    x2 = x * 2

explicit quadratic end:
    end_x2 = lookahead.x * 2
    end_y2 = lookahead.y * 2

implied quadratic end:
    end_x2 = control.x + lookahead.x
    end_y2 = control.y + lookahead.y
```

The implementation must not compute implied midpoint with integer division. A midpoint such as `(1 + 2) / 2` is representable as `end_x2 = 3`, not rounded to `1` or `2`.

Pure classifier flow:

```text
gui_sfnt_classify_simple_glyph_curve_segment edge lookahead
    -> read start/end/span from edge
    -> if span.point_count == 1
        return NoSegment SinglePointContour
    -> if start is off-curve
        return NoSegment OffCurveStart
    -> if end is on-curve
        return Line with doubled start/end coordinates
    -> if lookahead is None
        return NoSegment MissingLookahead
    -> if lookahead is on-curve
        return Quadratic with explicit doubled lookahead end
    -> otherwise
        return Quadratic with implied doubled midpoint end
```

Byte lookup flow:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_contour_edge_with_tables
    -> read start/end point flags from edge
    -> if start is on-curve and end is off-curve
        -> compute lookahead_contour_point_index = wrap(edge.next_contour_point_index + 1)
        -> gui_sfnt_glyf_simple_contour_point_with_tables for lookahead
        -> gui_sfnt_classify_simple_glyph_curve_segment edge (Some lookahead)
    -> otherwise
        -> gui_sfnt_classify_simple_glyph_curve_segment edge None
```

This deliberate conditional lookahead avoids surfacing unrelated later coordinate corruption for an edge that is already a line, a one-point no-segment, or an off-curve-start no-segment.

`NoSegment` is a successful classification state. Out-of-range `contour_index` / `edge_index` and malformed bytes remain `Result::Err GuiSfntParseError`. The classifier must not convert unsupported shape semantics into silent fallback drawing.

F4l uses internal table helpers, not public wrappers, after metadata is parsed:

```text
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
```

F4l must not allocate `Vec GuiSfntSimpleGlyphCurveSegment`, use integer midpoint division, call platform font APIs, use host text measurement, build full outlines, or rasterize pixels.

### SFNT simple glyph path command projection

F4m is the first sink-facing projection layer above curve segment classification. It still does not allocate a full outline, own a path sink trait, decide winding/fill rules, emit render2d commands, or rasterize pixels. It maps exactly one `GuiSfntSimpleGlyphCurveSegment` to explicit move and draw command values.

The type model uses enum payloads:

```text
GuiSfntSimpleGlyphPathMoveTo:
    contour_index i32
    edge_index i32
    x2 i32
    y2 i32

GuiSfntSimpleGlyphPathLineTo:
    contour_index i32
    edge_index i32
    x2 i32
    y2 i32

GuiSfntSimpleGlyphPathQuadraticTo:
    contour_index i32
    edge_index i32
    control_x2 i32
    control_y2 i32
    end_x2 i32
    end_y2 i32
    end_is_implied bool

GuiSfntSimpleGlyphPathSkipNoSegment:
    contour_index i32
    edge_index i32
    reason GuiSfntSimpleGlyphCurveNoSegmentReason

GuiSfntSimpleGlyphPathCommand:
    MoveTo GuiSfntSimpleGlyphPathMoveTo
    LineTo GuiSfntSimpleGlyphPathLineTo
    QuadraticTo GuiSfntSimpleGlyphPathQuadraticTo
    SkipNoSegment GuiSfntSimpleGlyphPathSkipNoSegment
```

Projection flow:

```text
gui_sfnt_simple_glyph_curve_segment_move_to_command segment
    -> Line: MoveTo line.start_x2 line.start_y2
    -> Quadratic: MoveTo quadratic.start_x2 quadratic.start_y2
    -> NoSegment: SkipNoSegment no_segment

gui_sfnt_simple_glyph_curve_segment_draw_command segment
    -> Line: LineTo line.end_x2 line.end_y2
    -> Quadratic: QuadraticTo control/end doubled coordinates
    -> NoSegment: SkipNoSegment no_segment
```

There is no command index in this API. Returning `GuiSfntSimpleGlyphPathCommand` directly keeps the boundary pure while avoiding an invalid-index state that would otherwise need `Option` or `Result`. The caller chooses the move phase or draw phase explicitly.

`SkipNoSegment` is a typed command, not fallback drawing and not silent ignore. A later streaming sink may count, log, or explicitly skip it by matching `GuiSfntSimpleGlyphPathCommand::SkipNoSegment`.

The command payload must be compact. F4m does not copy the whole edge, line segment, quadratic segment, or no-segment value into the command. It projects those values to the source contour/edge index, doubled coordinates, and no-segment reason that a later sink needs. This keeps the abstraction cheap and prevents deeply nested payloads from becoming the path interface.

F4m must not allocate `Vec GuiSfntSimpleGlyphPathCommand`, call `gui_sfnt_parse_metadata`, call platform font APIs, use host text measurement, import render2d/backend modules, build full glyph outlines, or rasterize pixels.

### SFNT simple glyph path command public lookup

F4n is a thin public composition layer over F4l and F4m. It takes the same byte-backed lookup input shape as the existing curve segment public lookup and returns the path command value needed by the next layer.

```text
gui_sfnt_lookup_simple_glyph_move_to_command bytes face_index glyph contour_index edge_index
    -> gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index
    -> gui_sfnt_simple_glyph_curve_segment_move_to_command segment

gui_sfnt_lookup_simple_glyph_draw_command bytes face_index glyph contour_index edge_index
    -> gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index
    -> gui_sfnt_simple_glyph_curve_segment_draw_command segment
```

The implementation must not call `gui_sfnt_parse_metadata`, `gui_sfnt_glyf_simple_curve_segment_with_tables`, lower point/contour table helpers, renderer APIs, rasterizers, host text measurement, or platform APIs. Those responsibilities already belong to earlier lookup layers or later rendering phases.

Error flow is one-to-one with F4l byte-backed lookup:

```text
Result::Err parse_error
    -> Result::Err parse_error

Result::Ok segment
    -> Result::Ok path_command
```

`NoSegment` remains a successful path command state. Both public F4n helpers map it to `SkipNoSegment`; they do not return `Option::None`, synthesize an empty command, or silently ignore the edge.

### SFNT simple glyph path command pair lookup

F4o is a single-edge pair boundary. It is not a contour stream and does not define a command sequence for a full outline. The pair is an O(1) value that carries the two explicit phases already defined by F4m.

```text
GuiSfntSimpleGlyphPathCommandPair:
    move_command GuiSfntSimpleGlyphPathCommand
    draw_command GuiSfntSimpleGlyphPathCommand
```

Pure projection:

```text
gui_sfnt_simple_glyph_curve_segment_path_command_pair segment
    -> move = gui_sfnt_simple_glyph_curve_segment_move_to_command segment
    -> draw = gui_sfnt_simple_glyph_curve_segment_draw_command segment
    -> GuiSfntSimpleGlyphPathCommandPair move draw
```

Byte-backed public lookup:

```text
gui_sfnt_lookup_simple_glyph_path_command_pair bytes face_index glyph contour_index edge_index
    -> gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index
    -> gui_sfnt_simple_glyph_curve_segment_path_command_pair segment
```

The public byte-backed helper must call `gui_sfnt_lookup_simple_glyph_curve_segment` exactly once. It must not call the separate move and draw public lookup helpers because that would decode the same SFNT edge twice. It must not call `gui_sfnt_parse_metadata`, `gui_sfnt_glyf_simple_curve_segment_with_tables`, lower public lookup helpers, the curve classifier, renderer APIs, rasterizers, host text measurement, or platform APIs.

The pair is not a list. F4o does not expose `command_index`, `count`, `next`, mutable current point state, or `Vec GuiSfntSimpleGlyphPathCommand`. A later contour/path sink can choose how to consume the pair values, but F4o does not define contour closure or off-curve contour-start synthesis.

`NoSegment` remains explicit. Both `move_command` and `draw_command` are `SkipNoSegment`, preserving the reason value for later diagnostic, skip counting, or sink behavior.

### SFNT simple glyph path sink event adapter

F4p is a single-edge adapter from the F4o command pair to the event pair a later contour/path sink can consume. It is still not the real sink. It does not define contour stream traversal, full outline command order, contour closure, off-curve contour-start synthesis, winding, fill rules, rasterization, render2d commands, host text measurement, or platform presentation.

The adapter intentionally reuses the existing compact command enum instead of defining another path representation:

```text
GuiSfntSimpleGlyphPathSinkEvent:
    Command GuiSfntSimpleGlyphPathCommand

GuiSfntSimpleGlyphPathSinkEventPair:
    first_event GuiSfntSimpleGlyphPathSinkEvent
    second_event GuiSfntSimpleGlyphPathSinkEvent
```

The pure command wrapper is total:

```text
gui_sfnt_simple_glyph_path_command_sink_event command
    -> GuiSfntSimpleGlyphPathSinkEvent::Command command
```

The pair adapter uses only the F4o accessors:

```text
gui_sfnt_simple_glyph_path_command_pair_sink_event_pair pair
    -> move_command = gui_sfnt_simple_glyph_path_command_pair_move_command pair
    -> draw_command = gui_sfnt_simple_glyph_path_command_pair_draw_command pair
    -> first_event = gui_sfnt_simple_glyph_path_command_sink_event move_command
    -> second_event = gui_sfnt_simple_glyph_path_command_sink_event draw_command
    -> GuiSfntSimpleGlyphPathSinkEventPair first_event second_event
```

F4p must not return `Option` or `Result` from the pure adapter because a valid `GuiSfntSimpleGlyphPathCommandPair` already contains both commands. It must not call byte-backed lookup helpers, metadata parsers, `*_with_tables` helpers, lower point / contour helpers, or the curve classifier. It must not allocate `Vec GuiSfntSimpleGlyphPathSinkEvent`, expose `command_index`, `count`, `next`, or carry mutable current point state.

`SkipNoSegment` remains a typed event by wrapping the existing `GuiSfntSimpleGlyphPathCommand::SkipNoSegment` value. This keeps later sink behavior explicit without changing parse status.

### SFNT simple glyph path sink event kind classification

F4q adds a dispatch classification boundary above the F4p event wrapper. It is not a replacement path representation and not a compact payload enum. The authority for coordinates, contour index, edge index, and skip source remains the wrapped `GuiSfntSimpleGlyphPathCommand`.

```text
GuiSfntSimpleGlyphPathSinkEventKind:
    MoveTo
    LineTo
    QuadraticTo
    SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason

GuiSfntSimpleGlyphPathSinkEventKindPair:
    first_kind GuiSfntSimpleGlyphPathSinkEventKind
    second_kind GuiSfntSimpleGlyphPathSinkEventKind
```

The event kind helper is total and uses exhaustive `match` on the wrapped command:

```text
gui_sfnt_simple_glyph_path_sink_event_kind event
    -> command = gui_sfnt_simple_glyph_path_sink_event_command event
    -> match command:
        MoveTo _ -> MoveTo
        LineTo _ -> LineTo
        QuadraticTo _ -> QuadraticTo
        SkipNoSegment skip -> SkipNoSegment (reason skip)
```

`SkipNoSegment` carries only `GuiSfntSimpleGlyphCurveNoSegmentReason`. That reason is suitable for diagnostics, explicit skip counting, and branch selection, but it is not enough to recover the source contour/edge. Callers that need source indices or coordinates must read the original command payload from `GuiSfntSimpleGlyphPathSinkEvent`.

The kind pair helper must use only F4p event pair accessors and `gui_sfnt_simple_glyph_path_sink_event_kind`:

```text
gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair pair
    -> first_event = gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair
    -> second_event = gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair
    -> first_kind = gui_sfnt_simple_glyph_path_sink_event_kind first_event
    -> second_kind = gui_sfnt_simple_glyph_path_sink_event_kind second_event
    -> GuiSfntSimpleGlyphPathSinkEventKindPair first_kind second_kind
```

F4q must not add `contour_index`, `edge_index`, coordinate fields, control/end fields, a real sink trait, allocation ownership, contour stream traversal, contour closure, off-curve contour-start synthesis, winding, fill rules, rasterization, render2d commands, host text measurement, or platform presentation. The pure helpers must not return `Option` or `Result`, allocate `Vec GuiSfntSimpleGlyphPathSinkEventKind`, expose `command_index`, `count`, `next`, or call byte-backed lookup helpers, metadata parsers, `*_with_tables` helpers, lower point / contour helpers, or the curve classifier.

### SFNT simple glyph path sink event indexed selection

F4r adds typed slot selection over the F4p/F4q two-slot values. It is not a contour iterator, not a command sequence, and not a stream cursor. The goal is to keep the next sink layer from introducing numeric indexes while still letting it choose the first or second event of one already-decoded edge pair.

```text
GuiSfntSimpleGlyphPathSinkEventSlot:
    First
    Second

gui_sfnt_simple_glyph_path_sink_event_pair_event_at pair slot:
    match slot:
        First -> gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair
        Second -> gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair

gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at pair slot:
    match slot:
        First -> gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind pair
        Second -> gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind pair

gui_sfnt_simple_glyph_path_sink_event_pair_kind_at pair slot:
    event = gui_sfnt_simple_glyph_path_sink_event_pair_event_at pair slot
    gui_sfnt_simple_glyph_path_sink_event_kind event
```

The slot enum is the only selector. F4r must not accept an `i32` event index, because a numeric index would reintroduce impossible states that have to be reported at runtime. `First` and `Second` are exhaustive; helpers therefore must not return `Option` or `Result`.

`event_pair_kind_at` deliberately composes `event_at` and the F4q kind helper. It must not duplicate kind classification logic, build a `GuiSfntSimpleGlyphPathSinkEventKindPair`, read path coordinates, or call byte-backed glyph lookup. `kind_pair_kind_at` deliberately uses only the kind pair first / second accessors.

F4r must not allocate `Vec GuiSfntSimpleGlyphPathSinkEvent`, allocate `Vec GuiSfntSimpleGlyphPathSinkEventKind`, use `push`, expose `command_index`, `count`, `next`, mutable current point state, contour traversal, contour closure, off-curve contour-start synthesis, winding, fill rules, rasterization, render2d commands, host text measurement, or platform presentation.

### SFNT simple glyph path contour traversal step

F4s adds the first cursor-shaped traversal boundary above F4r. It advances exactly one sink event in a simple glyph contour. It is still not a full sink trait, not a path builder, not a `Vec` command stream, and not a renderer. Its responsibility is to make the next contour state explicit with enum data so that later sink ownership can remain deterministic and testable.

```text
GuiSfntSimpleGlyphPathContourCursor:
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot

GuiSfntSimpleGlyphPathContourNext:
    Continue GuiSfntSimpleGlyphPathContourCursor
    EndContour

GuiSfntSimpleGlyphPathContourStep:
    cursor GuiSfntSimpleGlyphPathContourCursor
    event GuiSfntSimpleGlyphPathSinkEvent
    kind GuiSfntSimpleGlyphPathSinkEventKind
    next GuiSfntSimpleGlyphPathContourNext
```

The public lookup is:

```text
gui_sfnt_lookup_simple_glyph_path_contour_step bytes face_index cursor
    -> glyph = cursor.glyph
    -> contour_index = cursor.contour_index
    -> edge_index = cursor.edge_index
    -> slot = cursor.slot
    -> span = gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index
    -> pair = gui_sfnt_lookup_simple_glyph_path_command_pair bytes face_index glyph contour_index edge_index
    -> event_pair = gui_sfnt_simple_glyph_path_command_pair_sink_event_pair pair
    -> event = gui_sfnt_simple_glyph_path_sink_event_pair_event_at event_pair slot
    -> kind = gui_sfnt_simple_glyph_path_sink_event_kind event
    -> next = private validated cursor-next helper using span.point_count
    -> GuiSfntSimpleGlyphPathContourStep cursor event kind next
```

The next rule is fixed:

```text
First  -> Continue same glyph / same contour / same edge / Second
Second -> Continue same glyph / same contour / next edge / First
Second on final edge -> EndContour
```

`EndContour` is a successful step state, not an error and not `Option::None`. The lookup returns `Result::Err` only for parse, missing glyph, missing contour, or invalid edge range errors already reported by the lower checked lookups.

The next helper must remain private unless it is changed to return `Result`. As a private helper, it may assume the public lookup has already checked `span_point_count > 0` and `0 <= edge_index < span_point_count`. Exposing a total public helper over unchecked raw cursor data would make invalid states appear statically valid, which conflicts with the static-checking policy.

F4s deliberately leaves off-curve contour-start synthesis unchanged. The current `SkipNoSegment OffCurveStart` event remains a typed event. Contour closure insertion, actual sink ownership, outline allocation, path repair, rasterization, render2d command emission, font fallback, and platform presentation are later phases.

### SFNT simple glyph allocation-free path sink ownership boundary

F4t turns an F4s contour step into a sink-facing ownership decision. It is deliberately still a one-step value transformation. It does not own a real sink trait, allocate a path list, build an outline object, apply fill rules, rasterize, or emit render2d commands.

The policy is split into two independent axes:

```text
GuiSfntSimpleGlyphPathOffCurveStartPolicy:
    KeepTypedSkip
    RejectUnsupported

GuiSfntSimpleGlyphPathClosurePolicy:
    KeepOpen
    EmitCloseAfterFinalEvent

GuiSfntSimpleGlyphPathSinkPolicy:
    off_curve_start_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy
    closure_policy GuiSfntSimpleGlyphPathClosurePolicy
```

`off_curve_start_policy` only applies to `GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment OffCurveStart`. It must not reject `SinglePointContour` or `MissingLookahead`, because those states are already typed F4l/F4s success states and do not mean the sink is being asked to synthesize an implied contour start.

The sink decision is represented as data:

```text
GuiSfntSimpleGlyphPathSinkRejectReason:
    UnsupportedOffCurveStart

GuiSfntSimpleGlyphPathSinkPrimaryAction:
    EmitEvent GuiSfntSimpleGlyphPathSinkEvent
    Reject GuiSfntSimpleGlyphPathSinkRejectReason

GuiSfntSimpleGlyphPathContourClose:
    glyph GuiGlyphId
    contour_index i32

GuiSfntSimpleGlyphPathSinkTailAction:
    NoTailAction
    CloseContour GuiSfntSimpleGlyphPathContourClose

GuiSfntSimpleGlyphPathSinkStep:
    source_step GuiSfntSimpleGlyphPathContourStep
    primary_action GuiSfntSimpleGlyphPathSinkPrimaryAction
    tail_action GuiSfntSimpleGlyphPathSinkTailAction
```

`Reject` is not `GuiSfntParseError`. Parse/range failure is still the responsibility of F4s lookup and remains the only reason the byte-backed F4t helper returns `Result::Err GuiSfntParseError`. Policy rejection stays in the successful payload so caller code can distinguish malformed font data from a configured sink capability refusal.

The tail action depends on both the primary action and the F4s next state. This avoids the ambiguous state where a rejected final event also asks the sink to close the contour.

```text
if primary_action is Reject:
    tail_action = NoTailAction

else if step.next is Continue:
    tail_action = NoTailAction

else if step.next is EndContour and closure_policy is KeepOpen:
    tail_action = NoTailAction

else if step.next is EndContour and closure_policy is EmitCloseAfterFinalEvent:
    tail_action = CloseContour glyph contour_index
```

`CloseContour` uses the glyph and contour index from `step.cursor`. It is a marker for the next sink layer, not a renderer command and not a coordinate-bearing path segment. The future real sink can map it to its own close-path operation or reject unsupported closure semantics with a separate typed result.

The public pure helper is:

```text
gui_sfnt_simple_glyph_path_sink_step_from_contour_step policy step
    -> primary = gui_sfnt_simple_glyph_path_sink_primary_action_from_contour_step policy step
    -> tail = gui_sfnt_simple_glyph_path_sink_tail_action_from_contour_step policy step primary
    -> GuiSfntSimpleGlyphPathSinkStep step primary tail
```

The byte-backed helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index cursor policy
    -> contour_step = gui_sfnt_lookup_simple_glyph_path_contour_step bytes face_index cursor
    -> gui_sfnt_simple_glyph_path_sink_step_from_contour_step policy contour_step
```

It must not re-parse metadata itself, call internal table helpers directly, bypass F4s, allocate `Vec`, call renderer/platform APIs, rasterize, or consult host font fallback. The helper is only the boundary where checked bytes become a checked one-step sink decision.

### SFNT simple glyph path sink action selection projection

F4u projects an F4t sink step into one explicitly selected action. It is a selection/projection layer for a future sink. It does not mutate a sink, store a sink trait object, allocate an outline list, define callback ownership, repair paths, rasterize, emit render2d commands, or present to a platform backend.

The action slot is a different axis from the event slot:

```text
GuiSfntSimpleGlyphPathSinkEventSlot:
    First
    Second

GuiSfntSimpleGlyphPathSinkActionSlot:
    Primary
    Tail
```

`First` / `Second` selects one of the two command events generated for a contour edge. `Primary` / `Tail` selects one of the two F4t actions attached to an already materialized `GuiSfntSimpleGlyphPathSinkStep`. F4u must not collapse these slots into an integer, a shared enum, or a command index.

The unified action value is:

```text
GuiSfntSimpleGlyphPathSinkAction:
    EmitEvent GuiSfntSimpleGlyphPathSinkEvent
    Reject GuiSfntSimpleGlyphPathSinkRejectReason
    CloseContour GuiSfntSimpleGlyphPathContourClose
    NoAction
```

`NoAction` is only the explicit projection of `GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction`. It is not a fallback path, not an unsupported-operation success, and not a hidden no-op. Primary action projection must never return `NoAction`.

Pure projection helpers:

```text
gui_sfnt_simple_glyph_path_sink_primary_action_as_action primary_action
    EmitEvent event -> GuiSfntSimpleGlyphPathSinkAction::EmitEvent event
    Reject reason -> GuiSfntSimpleGlyphPathSinkAction::Reject reason

gui_sfnt_simple_glyph_path_sink_tail_action_as_action tail_action
    NoTailAction -> GuiSfntSimpleGlyphPathSinkAction::NoAction
    CloseContour close -> GuiSfntSimpleGlyphPathSinkAction::CloseContour close

gui_sfnt_simple_glyph_path_sink_step_action_at step slot
    Primary -> primary_action_as_action step.primary_action
    Tail -> tail_action_as_action step.tail_action
```

`gui_sfnt_simple_glyph_path_sink_step_action_at` is intentionally total over the two slot variants. It must not return `Option` or `Result`, expose `command_index`, accept a numeric action index, use a default arm, allocate `Vec`, call byte-backed lookup helpers, or duplicate F4t policy rules. It only reads the existing F4t action values and projects them into a unified action type.

The byte-backed helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action bytes face_index cursor policy slot
    -> sink_step = gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index cursor policy
    -> gui_sfnt_simple_glyph_path_sink_step_action_at sink_step slot
```

The byte-backed helper must call `gui_sfnt_lookup_simple_glyph_path_sink_step` exactly once. It must not call `gui_sfnt_lookup_simple_glyph_path_contour_step`, `gui_sfnt_lookup_simple_glyph_path_command_pair`, lower contour/curve helpers, `gui_sfnt_parse_metadata`, internal `*_with_tables` helpers, renderer/platform APIs, rasterizers, host font measurement, or font fallback. Its only failure channel remains the F4t lookup's `Result::Err GuiSfntParseError`; policy rejection remains `Result::Ok GuiSfntSimpleGlyphPathSinkAction::Reject`.

### SFNT simple glyph path sink action traversal step

F4v turns F4u action selection into a typed traversal step. It is still a contour-local value model, not a real sink. It does not mutate a sink, define callback ownership, allocate a command list, construct a full outline, repair paths, compute winding, rasterize, emit render2d commands, or present to a platform backend.

The traversal cursor combines the existing contour event cursor and the action slot:

```text
GuiSfntSimpleGlyphPathSinkActionCursor:
    contour_cursor GuiSfntSimpleGlyphPathContourCursor
    action_slot GuiSfntSimpleGlyphPathSinkActionSlot
```

The cursor intentionally carries the validated F4s contour cursor. It therefore contains the existing `contour_index` and `edge_index` through that nested value. F4v must not add a new numeric action index, command index, loop index, count field, or ad-hoc traversal counter. The action position is always the enum `GuiSfntSimpleGlyphPathSinkActionSlot`.

The next state is:

```text
GuiSfntSimpleGlyphPathSinkActionNext:
    Continue GuiSfntSimpleGlyphPathSinkActionCursor
    EndContour
```

`EndContour` is a successful terminal state of the contour-local action stream. It is not `Option::None`, not `Result::Err`, and not an implicit ignored event.

The step value is:

```text
GuiSfntSimpleGlyphPathSinkActionStep:
    cursor GuiSfntSimpleGlyphPathSinkActionCursor
    sink_step GuiSfntSimpleGlyphPathSinkStep
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionNext
```

`action` and `next` are separate facts. `action` says what a future sink consumes. `next` says where traversal continues. Next-state computation must not inspect whether the action is `EmitEvent`, `Reject`, `CloseContour`, or `NoAction`.

Pure traversal rules:

```text
gui_sfnt_simple_glyph_path_sink_action_next_from_step sink_step action_slot
    Primary:
        source_step.cursor -> Continue same contour_cursor Tail

    Tail:
        source_step.next = Continue next_cursor
            -> Continue next_cursor Primary

        source_step.next = EndContour
            -> EndContour
```

This means `Primary -> Tail` happens even when the primary action is `Reject`. It also means `Tail -> source_step.next` happens even when the tail action is `NoAction`. Reject handling and no-action handling belong to the consumer of the action payload, not to traversal.

`gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` must compose F4u rather than duplicate it:

```text
gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step sink_step action_slot
    -> cursor = GuiSfntSimpleGlyphPathSinkActionCursor source_step.cursor action_slot
    -> action = gui_sfnt_simple_glyph_path_sink_step_action_at sink_step action_slot
    -> next = gui_sfnt_simple_glyph_path_sink_action_next_from_step sink_step action_slot
    -> GuiSfntSimpleGlyphPathSinkActionStep cursor sink_step action next
```

The byte-backed helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy
    -> contour_cursor = cursor.contour_cursor
    -> action_slot = cursor.action_slot
    -> sink_step = gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index contour_cursor policy
    -> gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step sink_step action_slot
```

The byte-backed helper must call `gui_sfnt_lookup_simple_glyph_path_sink_step` exactly once. It must not call F4s contour-step lookup, path command pair lookup, lower contour/curve helpers, `gui_sfnt_parse_metadata`, internal `*_with_tables` helpers, renderer/platform APIs, rasterizers, host font measurement, or font fallback.

### SFNT simple glyph path sink action start cursor

F4w adds the entry position for the F4v action traversal stream. The start cursor is not a glyph iterator, not a sink, not a first action lookup, and not a renderer command. It is a typed value that names the first action slot of one contour:

```text
contour cursor:
    glyph = input glyph
    contour_index = input contour_index
    edge_index = 0
    event_slot = First

action cursor:
    contour_cursor = contour cursor
    action_slot = Primary
```

The pure constructor has no byte access:

```text
gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
    -> gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index 0 First
    -> gui_sfnt_simple_glyph_path_sink_action_cursor contour_cursor Primary
```

This constructor is intentionally unchecked. It must not claim that the contour exists, that the contour has at least one point, or that a glyph record is well-formed. Those facts belong to the byte-backed entry point:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor bytes face_index glyph contour_index
    -> gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index
    -> gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
```

The byte-backed entry point validates only the contour span boundary by calling `gui_sfnt_lookup_simple_glyph_contour_span` exactly once. It must not call F4v action-step lookup, F4t sink-step lookup, F4s contour-step lookup, point/curve/path-command helpers, policy helpers, full outline allocation, renderer/platform APIs, rasterizers, host font measurement, or font fallback. This keeps F4w as the start-cursor authority and leaves action payload traversal to F4v.

### SFNT simple glyph path sink action start step

F4x adds the first-step entry point for callers that need the first action step rather than only the start cursor. It is a thin composition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy
    -> start_cursor = gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy
```

This helper deliberately does not call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`. The byte-backed start cursor helper validates contour span for cursor-only callers, while the action-step lookup already validates the same contour through the F4v -> F4t -> F4s path. Calling both would duplicate the contour span validation and would make F4x look like a new validation authority. F4x is not a new authority; it only connects the unchecked start cursor value to the existing checked action-step lookup.

Error and policy taxonomy are inherited unchanged:

```text
parse/range/table error
    -> Result::Err GuiSfntParseError

policy reject
    -> Result::Ok step where step.action = Reject reason
```

F4x must not call `gui_sfnt_lookup_simple_glyph_contour_span`, `gui_sfnt_lookup_simple_glyph_path_sink_step`, F4s contour-step lookup, lower point/curve/path helpers, metadata parser, internal table helpers, renderer/platform APIs, rasterizers, host font measurement, or font fallback.

### SFNT simple glyph path sink action step advance

F4y resolves one `GuiSfntSimpleGlyphPathSinkActionNext` value into either a checked next action step or a typed contour terminal state:

```text
GuiSfntSimpleGlyphPathSinkActionStepAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionStep
    EndContour
```

The type is separate from `GuiSfntSimpleGlyphPathSinkActionNext` because `Next` contains only the next cursor, while `StepAdvance` contains the byte-backed lookup result for that cursor. Returning `Option GuiSfntSimpleGlyphPathSinkActionStep` would lose the domain reason for termination, and returning `Result::Err` for `EndContour` would confuse a successful terminal state with malformed font data.

The helper is a one-step state transition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy
    -> next = gui_sfnt_simple_glyph_path_sink_action_step_next step
    -> match next
        Continue cursor:
            gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy
                Err error -> Err error
                Ok next_step -> Ok Continue next_step
        EndContour:
            Ok EndContour
```

F4y does not inspect `step.action`, primary action, tail action, or unified action payload variants. `Reject`, `NoAction`, and `CloseContour` are payloads for a future consumer; they do not change the traversal rule. F4y must not loop, allocate a command list, mutate a sink, repair contours, rasterize, or present to any platform.

### SFNT simple glyph path sink action step item

F4z packages the current action step and the checked advance result into a single value:

```text
GuiSfntSimpleGlyphPathSinkActionStepItem:
    step GuiSfntSimpleGlyphPathSinkActionStep
    advance GuiSfntSimpleGlyphPathSinkActionStepAdvance
```

This is the first consumer-facing item boundary above F4y. It is deliberately not a contour iterator, not a sink trait, and not a command list. A later sink consumer can read `item.step.action` through the existing step accessor and use `item.advance` to decide whether the next item exists, but F4z itself does not interpret the action payload.

The byte-backed helper is a narrow composition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index step policy
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy
        Err error -> Err error
        Ok advance:
            stored_step = *step
            Ok ActionStepItem stored_step advance
```

The explicit `stored_step = *step` copy is part of the contract. The helper must not store a borrowed reference in the item, because the item is a value that can be passed to later consumers without aliasing the caller-owned reference.

F4z must not call start cursor helpers, start step helpers, F4v action step lookup, sink action lookup, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not allocate `Vec`, push into a command list, loop over a contour, inspect `Reject` / `NoAction` / `CloseContour`, or introduce a silent no-op path.

### SFNT simple glyph path sink action start item

F4aa adds a first-item entry point above F4x and F4z. It is useful for a future contour consumer that wants the first item directly, but it remains a narrow composition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy
        Err error -> Err error
        Ok start_step:
            gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy
                Err error -> Err error
                Ok item -> Ok item
```

F4aa does not introduce a new data type. The result type is the F4z `GuiSfntSimpleGlyphPathSinkActionStepItem`, so the current step and checked advance remain the only item payload. This preserves the typed value boundary and avoids a parallel "start item" structure that would duplicate state.

The helper must call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` exactly once and `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item` exactly once. It must not call the pure start-cursor helper, the byte-backed start-cursor helper, F4v action-step lookup, F4y advance helper, sink action lookup, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback.

F4aa itself does not inspect the action payload. `Reject`, `NoAction`, `CloseContour`, and `EndContour` remain typed states inside the F4x/F4z result path. This keeps parse/range/table failures as `Result::Err`, domain terminal states as enum values, and unsupported future behavior out of the helper instead of hiding it through fallback.

### SFNT simple glyph path sink action item next

F4ab resolves the checked advance already stored in an F4z/F4aa action item. It is deliberately a one-item boundary, not a contour iterator and not a real sink. Its only job is to turn the item's stored `GuiSfntSimpleGlyphPathSinkActionStepAdvance` into either the next checked item or the contour terminal state:

```text
GuiSfntSimpleGlyphPathSinkActionItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionStepItem
    EndContour
```

The public helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy
    -> advance = gui_sfnt_simple_glyph_path_sink_action_step_item_advance item
    -> match advance
        Continue next_step:
            gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &next_step policy
                Err error -> Err error
                Ok next_item -> Ok Continue next_item

        EndContour:
            Ok EndContour
```

`EndContour` is not an error. It means that the contour-local action stream reached its successful terminal state. Returning `Option::None` would make this indistinguishable from "no value was produced", and returning `Result::Err` would confuse a valid terminal contour with malformed font data.

F4ab must not inspect `item.step.action` or any nested `EmitEvent` / `Reject` / `NoAction` / `CloseContour` payload. Action payloads are what a future sink consumes; item-next only follows the checked traversal state that F4z already computed. This preserves the separation between "what to consume" and "where traversal continues".

The helper must call `gui_sfnt_simple_glyph_path_sink_action_step_item_advance` exactly once. In the `Continue` branch it must call `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item` exactly once. It must not call start helpers, action-step lookup, action-step advance lookup, sink action lookup, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not allocate `Vec`, push into a command list, loop over a contour, or introduce a hidden no-op path.

### SFNT simple glyph path sink action consumer item

F4ac packages one checked action item for the future sink consumer. F4z stores the checked source item as `step + advance`, and F4ab turns `advance` into `Continue next_item` or `EndContour`. F4ac intentionally does not replace either boundary. It reads the current action from the stored step and composes it with F4ab's checked next state:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItem:
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index item policy:
    stored_step = gui_sfnt_simple_glyph_path_sink_action_step_item_step item
    action = gui_sfnt_simple_glyph_path_sink_action_step_action &stored_step

    match gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy:
        Err error:
            Err error

        Ok next:
            Ok GuiSfntSimpleGlyphPathSinkActionConsumerItem action next
```

The explicit `stored_step` copy is part of the ownership contract. The consumer item stores values, not references to caller-owned storage. A later real sink can consume `action` and inspect `next` without relying on hidden mutable current-point state.

F4ac is not a real sink, not an iterator, and not a contour-wide consumer. It does not call a callback, mutate a sink, allocate a command list, construct an outline, decide fill rules, rasterize, emit render2d commands, or present to a platform backend. It also does not classify the action payload: `EmitEvent`, `Reject`, `NoAction`, and `CloseContour` remain data in `GuiSfntSimpleGlyphPathSinkAction`.

The helper must call `gui_sfnt_simple_glyph_path_sink_action_step_item_step` exactly once, `gui_sfnt_simple_glyph_path_sink_action_step_action` exactly once, and `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next` exactly once. It must not call F4z action item lookup, F4y action advance lookup, F4v action step lookup, F4x/F4aa start helpers, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not allocate `Vec`, push into a command list, loop over a contour, use numeric action indexes, or introduce hidden fallback.

### SFNT simple glyph path sink action consumer item next

F4ad advances a consumer item by one already checked continuation. It is deliberately above F4ac and below any real sink loop. The current item's action payload is not part of the traversal authority; only `item.next` decides whether a next consumer packet exists.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    EndContour
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next bytes face_index item policy:
    next = gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item

    match next:
        Continue next_item:
            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy:
                Err error:
                    Err error

                Ok next_consumer_item:
                    Ok Continue next_consumer_item

        EndContour:
            Ok EndContour
```

`EndContour` remains a successful terminal domain state. Returning `Option::None` would hide the difference between "no value exists" and "the contour stream completed"; returning `Result::Err` would confuse valid terminal state with malformed font data.

F4ad is not a loop and not a sink. It does not consume actions, does not decide whether `Reject` stops rendering, does not turn `NoAction` into skip, and does not map `CloseContour` to a renderer command. Those decisions belong to a later real sink or explicit consumer policy.

The helper must call `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next` exactly once and `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item` exactly once in the `Continue` branch. It must not call F4ab item-next lookup, F4z item lookup, F4y action advance lookup, F4v action step lookup, F4x/F4aa start helpers, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not read `item.action`, match action payload variants, allocate `Vec`, push into a command list, loop over a contour, or introduce hidden fallback.

### SFNT simple glyph path sink action apply state

F4ae is the first boundary that consumes the current action payload. F4ad intentionally keeps traversal authority separate and does not inspect `item.action`; F4ae accepts one `GuiSfntSimpleGlyphPathSinkAction` value and records what was consumed in a pure value state.

```text
GuiSfntSimpleGlyphPathSinkActionApplyStatus:
    EmittedEvent GuiSfntSimpleGlyphPathSinkEvent
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    ClosedContour GuiSfntSimpleGlyphPathContourClose
    NoAction

GuiSfntSimpleGlyphPathSinkActionApplyState:
    emitted_event_count i32
    reject_count i32
    close_contour_count i32
    no_action_count i32

GuiSfntSimpleGlyphPathSinkActionApplyStep:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
```

```text
gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action:
    match action:
        EmitEvent event:
            state.emitted_event_count += 1
            status = EmittedEvent event

        Reject reason:
            state.reject_count += 1
            status = Rejected reason

        CloseContour close:
            state.close_contour_count += 1
            status = ClosedContour close

        NoAction:
            state.no_action_count += 1
            status = NoAction
```

Only one counter changes per call. The counter state is not a cursor, not a continuation token, and not a replacement for F4ac/F4ad checked next state. A future contour-wide consumer may pair F4ad traversal with F4ae consumption, but this phase does not own that loop.

`Rejected` is a domain status, not `Result::Err`. It comes from a sink policy action selected from valid contour data and must stay distinguishable from malformed SFNT bytes. `NoAction` is also a consumed status, not an implicit skip or hidden no-op. Tests must match both variants explicitly.

The implementation must stay allocation-free and side-effect-free. It must not call lookup helpers, parse metadata, allocate an outline, push render commands, inspect current point state, rasterize, call a renderer, call platform APIs, or perform host text measurement.

### SFNT simple glyph path sink action consumer apply step

F4af composes F4ac and F4ae without taking over F4ad traversal. A consumer item already contains the current action and the checked item-level continuation. F4af applies the current action to an apply state, then stores the resulting apply step next to the already stored continuation.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionApplyStep
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item:
    action = gui_sfnt_simple_glyph_path_sink_action_consumer_item_action item
    next = gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item
    apply_step = gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action
    GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep apply_step next
```

The helper is total because it receives typed values that have already crossed the byte-backed parsing boundary. Adding `Result` here would confuse malformed SFNT data with `Rejected` / `NoAction` domain status.

`next` is the stored `GuiSfntSimpleGlyphPathSinkActionItemNext` value from the current consumer item. F4af must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` and must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, because that would resolve the next consumer packet and move F4ad's byte-backed traversal responsibility into this phase.

F4af must not match `GuiSfntSimpleGlyphPathSinkAction` variants directly. Payload interpretation belongs to F4ae. It also must not allocate `Vec`, push a command, loop over a contour, track current point state, perform lower SFNT lookup, parse metadata, rasterize, render, call a platform backend, or call host text measurement.

### SFNT simple glyph path sink action consumer apply terminal

F4ag turns a single F4af apply step into a pure terminal classification for a future consumer loop. It deliberately does not become that loop. It also does not resolve the next byte-backed consumer item.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
```

The helper reads the inner apply status first:

```text
status = gui_sfnt_simple_glyph_path_sink_action_apply_step_status step.apply_step

match status:
    Rejected reason:
        Rejected reason

    otherwise:
        match step.next:
            Continue _:
                Continue step

            EndContour:
                EndContour step
```

`Rejected` has priority over stored next state because a policy rejection is a domain terminal. It is not a parse error and must not be wrapped as `Result::Err`. `EndContour` is also a domain terminal, but successful. `Continue` keeps the already computed F4af apply step so the future loop can inspect status and counts without recomputing payload interpretation.

`NoAction` must not be treated as an implicit skip. Its terminal state is decided only by the stored next value. This preserves the distinction between "nothing was emitted by this action" and "the traversal has ended".

F4ag must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`, must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, and must not call byte-backed lower lookup helpers. It must not allocate `Vec`, push commands, loop over contour data, inspect current point state, parse metadata, rasterize, render, call platform APIs, or call host text measurement.

### SFNT simple glyph path sink action consumer apply advance

F4ah is the first post-apply one-step advance boundary. It combines F4ag's terminal classification with F4ac lookup through the stored `GuiSfntSimpleGlyphPathSinkActionItemNext`. It is not a direct F4ad call, because F4ag/F4af intentionally no longer carry the original `GuiSfntSimpleGlyphPathSinkActionConsumerItem`.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index step policy:
    terminal = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step

    match terminal:
        Rejected reason:
            Ok Rejected reason

        EndContour _:
            Ok EndContour

        Continue continue_step:
            next = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next continue_step

            match next:
                Continue next_item:
                    gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index next_item policy
                        |> map Continue

                EndContour:
                    Ok EndContour
```

`Rejected` remains a domain terminal, not a parse error. `EndContour` remains a successful terminal. The only byte-backed lookup in this phase is the F4ac consumer item lookup for the stored `next_item` in the `Continue` branch.

The apparent `Continue + EndContour` branch is defensive against future representation changes and keeps the function total over its input type. It must still return successful `EndContour`, not panic or silently skip.

F4ah must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, because that helper requires the original consumer item and would obscure the F4ag/F4af ownership boundary. F4ah must not re-apply action payloads, match `GuiSfntSimpleGlyphPathSinkAction` variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, call lower contour/curve lookup helpers directly, rasterize, render, call platform APIs, or call host text measurement.

### SFNT simple glyph path sink action consumer consume once

F4ai composes F4af and F4ah into the smallest useful future-loop step. It consumes exactly one already-checked consumer item and resolves the immediate post-apply continuation, but it still does not own a loop and does not become a sink.

The result must preserve both sides of the operation:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

`apply_step` is not redundant. It carries the updated apply state and the consumed action status. Returning only `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` would hide whether the current item emitted an event, rejected, closed a contour, or consumed an explicit `NoAction`.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state item policy:
    apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item

    match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index apply_step policy:
        Err error:
            Err error

        Ok advance:
            Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep apply_step advance
```

F4ai must not call F4ag directly. F4ah owns terminal classification and stored-next advancement. F4ai must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` because it would bypass the F4af/F4ah split and would also lose the structured apply result unless wrapped again.

F4ai must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, call lower contour/curve lookup helpers directly, rasterize, render, call platform APIs, or call host text measurement.

### SFNT simple glyph path sink action start consumer item

F4aj is the start boundary for the future consumer loop. It converts a glyph contour start into the first checked `GuiSfntSimpleGlyphPathSinkActionConsumerItem`, using the existing start-item and consumer-item contracts.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy:
    match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy:
        Err error:
            Err error

        Ok item:
            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index item policy:
                Err error:
                    Err error

                Ok consumer_item:
                    Ok consumer_item
```

F4aj intentionally does not create a new value type. The result type is the existing F4ac `GuiSfntSimpleGlyphPathSinkActionConsumerItem`, because the responsibility is only to provide a byte-backed entry point for the first consumer packet.

“No advance” in F4aj means no F4ad consumer-item-next call, no F4af apply, no F4ah post-apply advance, and no F4ai consume-once call. F4ac still resolves the checked `GuiSfntSimpleGlyphPathSinkActionItemNext` needed to construct a consumer item. That resolution remains inside F4ac and does not make F4aj a traversal authority.

F4aj must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`, must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, and must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once`. It must not call F4af/F4ah/F4ab/F4z/F4y/F4v/lower lookup helpers directly, inspect action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action start consume once

F4ak is the first start-to-consume boundary. It takes a byte-backed glyph contour start and an existing apply state, creates the first consumer item through F4aj, and consumes exactly that item through F4ai.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy:
    match gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy:
        Err error:
            Err error

        Ok consumer_item:
            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state consumer_item policy:
                Err error:
                    Err error

                Ok consume_step:
                    Ok consume_step
```

F4ak deliberately returns `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep`, not only `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`. The start boundary must preserve the same diagnostic and future-loop information as F4ai: the consumed action's apply state/status and the post-consume advance.

F4ak is not a contour loop and not a real sink. It must not call F4aa/F4ac/F4ad/F4af/F4ah/F4ab/F4z/F4y/F4v/lower lookup helpers directly. It must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`, inspect action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume step apply summary

F4al adds stable public projections over `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep`. The future loop needs the updated apply state and the consumed action status, but it should not depend on the nested storage layout of F4ai and F4af.

The state helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step:
    consumer_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step
    inner_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step consumer_apply_step
    gui_sfnt_simple_glyph_path_sink_action_apply_step_state inner_apply_step
```

The status helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step:
    consumer_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step
    inner_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step consumer_apply_step
    gui_sfnt_simple_glyph_path_sink_action_apply_step_status inner_apply_step
```

F4al deliberately does not read `advance`. The existing `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance` accessor remains the explicit API for terminal / next item state. Separating apply summary and advance keeps future loops from conflating consumed action diagnostics with traversal state.

F4al must not call byte-backed lookup helpers, consumer item next, consume-once, start helpers, or lower contour/curve lookup helpers. It must not allocate `Vec`, push commands, own a loop, inspect action payload variants, parse metadata, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume summary

F4am adds a flat public value above F4al. The future loop needs three values after each consume step: the updated apply state, the status of the consumed action, and the already-computed post-consume advance enum. Reading these three values independently would repeatedly expose the `ConsumeStep -> ApplyStep -> inner apply step` storage path to future code. F4am makes that read boundary explicit without taking ownership of traversal.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

The conversion helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step step:
    state = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step
    status = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step
    advance = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance step
    gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary state status advance
```

This differs from F4al intentionally. F4al does not read `advance`, because it only exposes the apply side. F4am does read `advance`, because its contract is to assemble the full consume-step summary. The source policy must keep these two contracts separate: F4al still forbids `advance`, while F4am requires exactly one call to the existing advance accessor.

F4am must not call byte-backed lookup helpers, consumer item next lookup, consume-once, start helpers, lower contour/curve lookup helpers, or metadata parser. It must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. It must not reinterpret the advance enum; `Continue`, `Rejected`, and `EndContour` remain the F4ah domain states.

### SFNT simple glyph path sink action consumer consume summary terminal

F4an adds the next pure projection above F4am. F4am creates a flat value and deliberately does not interpret the stored advance. The future loop still needs one stable API that reads the summary and returns traversal control without depending on the storage name or the F4ah enum directly. F4an provides that boundary.

The value shape is:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

The helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary:
    advance = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance summary
    match advance:
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance Continue item:
            GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal Continue item
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance Rejected reason:
            GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal Rejected reason
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance EndContour:
            GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal EndContour
```

Although the public enum name ends with `Terminal`, it is the future loop's traversal control state and includes `Continue`. This avoids forcing loop code to re-match the lower stored advance enum, while keeping the operation allocation-free and deterministic.

F4an must read `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance` exactly once. It must not call byte-backed lookup helpers, consumer item next lookup, consume-once, start helpers, lower contour/curve lookup helpers, metadata parser, or `*_with_tables`. It must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. Its only match target is `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`.

### SFNT simple glyph path sink action start consume summary

F4ap creates the initial consume summary for a future contour consumer. It is a thin composition of F4ak and F4am: F4ak finds and consumes the first action, and F4am projects the resulting consume step into the stable summary value used by F4ao.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary bytes face_index state glyph contour_index policy:
    start = gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy
    match start:
        Err error:
            Err error
        Ok consume_step:
            summary = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step consume_step
            Ok summary
```

F4ap does not own traversal beyond the first consumed action. It does not call F4ao, does not loop, and does not allocate an outline or event list. The only byte-backed lookup it may call directly is `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once`.

F4ap must call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once` exactly once and `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step` exactly once in the success branch. It must not call start item helpers, start consumer item helpers, consumer item consume-once, summary advance-once, consumer item next lookup, lower contour/curve lookup helpers, metadata parser, or `*_with_tables`. It must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume summary advance once

F4ao is the first byte-backed boundary above the F4am/F4an summary projection. It advances from one completed consume summary to the next completed consume summary, but it still advances only one action. It is not a contour-wide loop and does not own sink mutation or outline storage.

The result value is:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once bytes face_index summary policy:
    state = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary
    terminal = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary
    match terminal:
        Continue item:
            consume_once = gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state item policy
            match consume_once:
                Err error:
                    Err error
                Ok consume_step:
                    next_summary = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step consume_step
                    Ok Continue next_summary
        Rejected reason:
            Ok Rejected reason
        EndContour:
            Ok EndContour
```

`Rejected` and `EndContour` are domain terminals. They must remain `Result::Ok` values, because they do not mean that the SFNT bytes failed to parse. Only the Continue branch can call the existing consume-once byte-backed helper and therefore only that branch can produce a parse error from byte lookup.

F4ao must call `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state` exactly once, `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal` exactly once, `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once` exactly once in the Continue branch, and `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step` exactly once after a successful Continue consume. It must not call start helpers, consumer item next lookup, lower contour/curve lookup helpers, metadata parser, or `*_with_tables`. It must not match action payload variants, allocate `Vec`, push commands, own a full loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume summary drain budget

F4aq is the first bounded traversal boundary above F4ap/F4ao. It exists so a caller can advance a contour action consumer without allocating an outline, building a command list, mutating a real sink, or relying on an unbounded recursive traversal. The result is still a typed value, not a renderer side effect.

The drain result carries the summary at which traversal stopped:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain:
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    StepBudgetExhausted GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
```

`StepBudgetExhausted` is not fallback. It is an explicit domain terminal that says the caller provided no more action steps for this slice. Both `remaining_steps == 0` and `remaining_steps < 0` produce the same typed terminal when the current summary still has `Continue`.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget bytes face_index summary policy remaining_steps:
    terminal = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary
    match terminal:
        Rejected reason:
            Ok Rejected reason current_summary
        EndContour:
            Ok EndContour current_summary
        Continue:
            if remaining_steps <= 0:
                Ok StepBudgetExhausted current_summary
            else:
                advance = gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once bytes face_index summary policy
                match advance:
                    Err error:
                        Err error
                    Ok Continue next_summary:
                        gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget bytes face_index next_summary policy remaining_steps - 1
                    Ok Rejected reason:
                        Ok Rejected reason current_summary
                    Ok EndContour:
                        Ok EndContour current_summary
```

The current F4ao contract normally returns `Continue next_summary` when its input terminal is `Continue`. The F4ao `Rejected` and `EndContour` branches are still handled in F4aq as a compatibility-preserving domain branch. In those branches F4aq stores the same current summary that was passed to F4ao; it does not invent or reparse a new terminal summary.

The start helper composes F4ap and F4aq only:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary_drain_budget bytes face_index state glyph contour_index policy remaining_steps:
    start = gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary bytes face_index state glyph contour_index policy
    match start:
        Err error:
            Err error
        Ok summary:
            gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget bytes face_index summary policy remaining_steps
```

F4aq must not allocate `Vec`, push commands, match action payload variants, call lower contour/curve helpers directly, parse metadata, call `*_with_tables`, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. It is the final bounded traversal boundary before later phases decide owner recovery, outline storage, and command emission.

## SFNT simple glyph outline storage capacity and owner recovery boundary

F5a is the value-only boundary between bounded path traversal and future outline storage allocation. It consumes only checked topology values and capacity limits. It does not consume font bytes, does not allocate an outline, does not mutate a sink, and does not render.

The data flow is:

```text
F4aq drain result
    EndContour summary
        -> summary owner remains with caller
        -> caller may plan contour/outline storage from topology

    StepBudgetExhausted summary
        -> summary owner remains with caller
        -> caller must request another work slice
        -> capacity planning is not successful and not attempted for that unfinished contour

    Rejected rejected
        -> summary owner remains with rejected payload
        -> caller reports policy/domain rejection
        -> outline storage is not attempted
```

F5a does not call F4aq. The caller owns the scheduling boundary and only passes `GuiSfntSimpleGlyphTopology` to F5a after it has a complete contour traversal. This keeps time slicing separate from capacity planning and prevents a capacity helper from hiding unbounded byte-backed work.

Capacity calculation is pure:

```text
capacity_from_topology topology:
    contour_count = topology.contour_count
    point_count = topology.point_count

    if contour_count <= 0:
        InvalidTopology topology
    else if point_count <= 0:
        InvalidTopology topology
    else if contour_count > point_count:
        InvalidTopology topology
    else if point_count > 1073741823:
        CommandCountOverflow topology
    else:
        Fits capacity:
            glyph = topology.glyph
            contour_count = contour_count
            point_count = point_count
            edge_count = point_count
            path_command_pair_count = point_count
            path_command_count = point_count * 2
```

Limit comparison is also pure:

```text
check_limit capacity limit:
    if limit.max_contours <= 0 or capacity.contour_count > limit.max_contours:
        Rejected ContourCapacityExceeded capacity limit
    else if limit.max_points <= 0 or capacity.point_count > limit.max_points:
        Rejected PointCapacityExceeded capacity limit
    else if limit.max_edges <= 0 or capacity.edge_count > limit.max_edges:
        Rejected EdgeCapacityExceeded capacity limit
    else if limit.max_path_commands <= 0 or capacity.path_command_count > limit.max_path_commands:
        Rejected CommandCapacityExceeded capacity limit
    else:
        Fits capacity
```

`InvalidTopology` and `CommandCountOverflow` do not carry a capacity value because capacity cannot be trusted. Capacity exceeded carries both `capacity` and `limit` so a later owner-taking allocation API can return the original owner plus enough data to present a precise typed error or retry with a different limit.

F5a source policy:

- `Vec`, `push`, point list, contour list, path command list, raster mask, render command, platform API, host text measurement, and font substitute logic are forbidden.
- No byte-backed lookup, metadata parser, `*_with_tables`, F4aq drain helper, lower contour helper, or point decoder is called from capacity helpers.
- `StepBudgetExhausted` is a continuation-required state, not capacity success.
- All public states are enum / struct values, not strings.

## SFNT simple glyph outline storage owner boundary

F5b is the first allocation boundary after F5a. It converts a trusted outline capacity into one empty owner-backed scalar slot table. The storage is deliberately narrower than a complete outline representation:

```text
GuiSfntSimpleGlyphOutlineStorage:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    scalar_slots Vec i32
    scalar_slot_count i32
```

F5b does not decode point coordinates, does not synthesize implied on-curve points, does not push path commands, does not rasterize, and does not call a renderer or host text API. Those operations start in later phases after owner recovery and time slicing are stable.

The scalar slot count is:

```text
scalar_slot_count =
    contour_count
    + point_count
    + point_count
    + edge_count
    + path_command_count
```

The five terms reserve contour endpoint slots, x coordinate slots, y coordinate slots, edge slots, and path command tag slots. The `Vec` is allocated with that capacity and `len = 0`; later builders populate it under their own mutation contract.

Forged capacity values must not be converted into F5a `Rejected` payloads. A rejected payload means "trusted capacity exceeds caller limit", so F5b validates the capacity shape before limit comparison:

```text
shape_is_valid capacity:
    contour_count > 0
    point_count > 0
    contour_count <= point_count
    edge_count == point_count
    path_command_pair_count == point_count
    point_count <= 1073741823
    path_command_count == point_count * 2
```

The `point_count <= 1073741823` check is evaluated before `point_count * 2`. If shape validation fails, allocation returns `InvalidCapacity` and `capacity_check = None`. This is the only branch where the error has no capacity check value.

After shape validation, F5b calls `check_limit capacity limit`. `Rejected` is returned as `CapacityRejected` with `capacity_check = Some checked`. `Fits` proceeds to scalar slot count overflow checking. Unexpected `InvalidTopology` or `CommandCountOverflow` results are treated as `InvalidCapacity` with `Some checked`, because they cannot be produced by a valid direct limit check and indicate an invariant break.

Scalar slot count overflow is checked by staged residual subtraction from `2147483647`:

```text
remaining = 2147483647
subtract contour_count
subtract point_count
subtract point_count
subtract edge_count
subtract path_command_count
```

If any term is larger than the remaining capacity, the result is `ScalarSlotCountOverflow`. Only after all terms fit does F5b compute the final count using prefix `add` and call `Vec` allocation exactly once.

Allocation failure returns `ScalarSlotStorageAllocFailed` with `capacity_check = Some checked`. Since F5b owns only one `Vec i32`, there is no partially allocated multi-owner aggregate to recover. `gui_sfnt_simple_glyph_outline_storage_free` consumes the storage and calls `vec::free` once.

## SFNT simple glyph outline scalar slot mutation boundary

F5c is the first mutation boundary for `GuiSfntSimpleGlyphOutlineStorage`. It only appends an i32 scalar slot value to the owner-backed `scalar_slots` table. It does not decide which logical region the value belongs to, and it does not decode point bytes, synthesize contour closure, emit path commands, rasterize, render, or call host/platform APIs.

The public mutation result is:

```text
push_scalar_slot storage value:
    Result GuiSfntSimpleGlyphOutlineStorage GuiSfntSimpleGlyphOutlineStoragePushError
```

The error payload is owner-preserving:

```text
GuiSfntSimpleGlyphOutlineStoragePushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    scalar_value i32
    error StdErrorKind
```

`scalar_value` is Copy, but it is still stored explicitly in the error payload. This keeps the API shape consistent with `VecPushError`: failure returns both the input owner and the rejected item, rather than relying on caller-local variables or hidden cleanup.

Implementation order:

```text
capacity = storage.capacity
scalar_slot_count = storage.scalar_slot_count
scalar_slots = storage.scalar_slots
push_result = vec::push scalar_slots value

Ok next_slots:
    Ok Storage capacity next_slots scalar_slot_count

Err e:
    error_kind = vec_push_error_kind &e
    returned_slots = vec_push_error_vec e
    returned_storage = Storage capacity returned_slots scalar_slot_count
    Err PushError returned_storage value error_kind
```

The error kind must be read before consuming `e` with `vec_push_error_vec`. F5c does not call `vec::with_capacity`, `vec::free`, `vec::filled`, `vec::replace`, or `vec::pop`; allocation growth remains inside `vec::push`, and cleanup remains the caller's explicit responsibility through the F5b free helper.

The recovery API has two forms:

```text
push_error_storage error -> storage owner
push_error_with error callback -> callback storage scalar_value error_kind
```

The consuming storage accessor is convenient for Copy scalar payloads. The eliminator preserves the pattern used by owner-bearing collection APIs and is the preferred shape if later push errors carry non-Copy payloads.

## SFNT simple glyph outline scalar region cursor boundary

F5d adds typed cursor movement over the F5b scalar slot storage. The boundary is deliberately still below point decode and path command synthesis. It only answers "which fixed scalar region is being filled next" and "is the storage owner synchronized with that cursor".

The region order is fixed:

```text
contour endpoints
x coordinates
y coordinates
edges
path command tags
```

The unchecked boundary helper is internal. Public cursor construction is fail-closed:

```text
try_from_capacity capacity region:
    if not shape_is_valid capacity:
        Err InvalidOperation
    else:
        match scalar_slot_count_check capacity:
            Fits expected:
                Ok from_valid_capacity capacity region
            Overflow:
                Err CapacityExceeded
```

This ordering matters because region start/end uses i32 addition. A forged capacity must not reach addition before shape validation and scalar slot count overflow checking have succeeded.

`push_region_scalar` validates storage and cursor before mutation:

```text
capacity = storage.capacity
scalar_slot_count = storage.scalar_slot_count
scalar_slots_len = len storage.scalar_slots
scalar_slots_cap = cap storage.scalar_slots

if not shape_is_valid capacity:
    Err StorageCapacityInvalid storage cursor value None
else:
    match scalar_slot_count_check capacity:
        Fits expected:
            if scalar_slot_count != expected:
                Err StorageCapacityInvalid storage cursor value None
            else if scalar_slots_cap != scalar_slot_count:
                Err StorageCapacityInvalid storage cursor value None
            else if not cursor_is_well_formed cursor:
                Err CursorInvalid storage cursor value None
            else if not cursor matches from_valid_capacity capacity cursor.region:
                Err CursorRegionMismatch storage cursor value None
            else if scalar_slots_len != cursor.next_index:
                Err StorageCursorMismatch storage cursor value None
            else if cursor.next_index >= cursor.end:
                Err RegionFull storage cursor value None
            else:
                call F5c push_scalar_slot exactly once
        Overflow:
            Err StorageCapacityInvalid storage cursor value None
```

`scalar_slots_cap == scalar_slot_count` is checked before the F5c call. F5c is a general owner-preserving push helper, but F5d is a fixed outline-region boundary; allowing Vec growth here would hide a broken storage invariant. `scalar_slots_len == cursor.next_index` is checked before `RegionFull` so that forged full cursors over shorter storage are classified as `StorageCursorMismatch`, not as a legitimately full region.

Both success and failure payloads return the storage owner. `GuiSfntSimpleGlyphOutlineRegionPush` and `GuiSfntSimpleGlyphOutlineRegionPushError` must not implement `Clone` or `Copy`. The cursor and error kind are value-only and may implement `Clone` / `Copy`.

## SFNT simple glyph contour endpoint population boundary

F5e is the first semantic population boundary over the F5d contour endpoint region. It accepts a typed endpoint slot:

```text
GuiSfntSimpleGlyphContourEndpointSlot:
    contour_index i32
    end_point_index i32
```

The helper does not read `glyf` bytes. It is intentionally usable with synthetic endpoint values in doctests so the owner/cursor contract can be stabilized before byte-backed endpoint-array decoding is wired in.

F5e keeps three invariants separate:

```text
storage capacity invariant
cursor position invariant
endpoint sequence invariant
```

The validation order is fail-closed:

```text
capacity = storage.capacity

if not shape_is_valid capacity:
    Err StorageCapacityInvalid
else if scalar_slot_count_check capacity is not Fits:
    Err StorageCapacityInvalid
else:
    contour_count = capacity.contour_count
    point_count = capacity.point_count

    if not cursor_is_well_formed cursor:
        Err CursorInvalid
    else if cursor.region != ContourEndpoint:
        Err CursorRegionMismatch
    else if endpoint.contour_index != cursor.next_index:
        Err ContourIndexMismatch
    else if endpoint.contour_index < 0 or endpoint.contour_index >= contour_count:
        Err ContourIndexMismatch
    else if endpoint.end_point_index < 0 or endpoint.end_point_index >= point_count:
        Err EndpointOutOfRange
    else:
        previous must satisfy 0 <= previous < point_count - 1 when present
        end_point_index must be greater than previous when present
        validate final or non-final endpoint
        commit through F5d region push exactly once
```

The previous endpoint contract is:

```text
None:
    contour_index must be 0

Some previous:
    contour_index must be greater than 0
    previous must satisfy 0 <= previous < point_count - 1
    end_point_index must be greater than previous
```

The final endpoint contract is:

```text
if contour_index + 1 == contour_count:
    end_point_index == point_count - 1
else:
    end_point_index < point_count - 1
```

The `point_count - 1` arithmetic happens only after capacity shape and scalar slot count validation. This prevents forged capacity values from reaching semantic endpoint checks before the lower storage contract has accepted the shape.

F5e wraps F5d region push failure without losing ownership:

```text
RegionPushFailed:
    storage = recovered storage from F5d error
    region_error_kind = Some F5d error kind
    push_error_kind = F5d underlying push_error_kind
```

Validation failures set `region_error_kind = None` and `push_error_kind = None`, because no lower region push was attempted. The success and error payloads own `GuiSfntSimpleGlyphOutlineStorage`, so neither implements `Clone` or `Copy`.

## SFNT simple glyph contour endpoint byte reader bridge

F5f connects the already checked `glyf` endpoint-array reader to the F5e contour endpoint storage push. It is intentionally a bridge, not a full outline builder. It reads one endpoint from bytes and either returns a read error before mutation or delegates to F5e exactly once.

The bridge keeps these failure domains separate:

```text
byte read failure
    Result::Err ReadFailed
    parse_error = Some GuiSfntParseError
    endpoint = None
    no F5e push was attempted
    storage is the original storage owner

storage push failure
    Result::Err PushFailed
    parse_error = None
    endpoint = Some read endpoint slot
    push_error_kind = Some F5e error kind
    region_error_kind = F5d error kind when present
    storage_push_error_kind = F5c error kind when present
    storage is recovered from the F5e error owner
```

The helper order is fixed:

```text
match gui_sfnt_glyf_read_contour_endpoint bytes glyf topology contour_index:
    Err parse_error:
        return ReadFailed with original storage and cursor

    Ok end_point_index:
        endpoint = GuiSfntSimpleGlyphContourEndpointSlot contour_index end_point_index
        match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint previous_endpoint:
            Ok pushed:
                return storage, advanced cursor, previous endpoint

            Err push_error:
                endpoint_value = endpoint from push_error
                push_kind = kind from push_error
                region_kind = region_error_kind from push_error
                storage_push_kind = push_error_kind from push_error
                returned_storage = storage from push_error
                return PushFailed with returned_storage and metadata
```

The lower error metadata must be read before `returned_storage = storage from push_error`, because that accessor consumes the owner-bearing error. This ordering preserves both the byte-read endpoint value and the recovered storage owner.

F5f must not call point stream construction, point coordinate decode, path generation, rasterization, render2d, platform APIs, host text measurement, or direct `Vec` APIs. The only allowed byte-side call in the bridge body is `gui_sfnt_glyf_read_contour_endpoint`.

## SFNT simple glyph point x coordinate population boundary

F5g adds the first coordinate-specific storage boundary. It accepts a typed x coordinate slot and appends its scalar value into the F5d `PointX` region.

```text
GuiSfntSimpleGlyphPointXSlot:
    point_index i32
    x i32
```

F5g does not read bytes. The point stream and x delta decoder already exist for older lookup APIs, but this phase deliberately stabilizes the storage mutation contract before wiring byte-backed x decode into the outline builder.

The main risk is confusing scalar storage index with glyph logical point index. For a two-contour, four-point glyph, the `PointX` cursor starts at scalar slot 2:

```text
ContourEndpoint region: [0, 2)
PointX region:          [2, 6)
```

Therefore:

```text
logical_point_index = cursor.next_index - cursor.start
```

This subtraction is valid only after:

```text
capacity shape is valid
scalar slot count is Fits
cursor is well formed
cursor region is PointX
cursor boundaries match the checked capacity
```

The validation order is fail-closed:

```text
capacity = storage.capacity

if not shape_is_valid capacity:
    Err StorageCapacityInvalid
else if scalar_slot_count_check capacity is not Fits:
    Err StorageCapacityInvalid
else:
    point_count = capacity.point_count

    if not cursor_is_well_formed cursor:
        Err CursorInvalid
    else if cursor.region != PointX:
        Err CursorRegionMismatch
    else if cursor does not match checked capacity:
        Err CursorRegionMismatch
    else:
        logical_point_index = cursor.next_index - cursor.start

        if point.point_index != logical_point_index:
            Err PointIndexMismatch
        else if point.point_index < 0 or point.point_index >= point_count:
            Err PointIndexOutOfRange
        else:
            commit through F5d region push exactly once
```

F5g wraps F5d region push failure without losing ownership:

```text
RegionPushFailed:
    storage = recovered storage from F5d error
    region_error_kind = Some F5d error kind
    push_error_kind = F5d underlying push_error_kind
```

The F5d error kind and F5c push error kind must be read before consuming the owner-bearing F5d error via its storage accessor.

F5g must not call byte readers, `gui_sfnt_glyf_*` helpers, point stream construction, coordinate decode, path generation, rasterization, render2d, platform APIs, host text measurement, or direct `Vec` APIs. The only mutation call in the commit helper is `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`.

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
