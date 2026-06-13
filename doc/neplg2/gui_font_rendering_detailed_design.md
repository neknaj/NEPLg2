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
