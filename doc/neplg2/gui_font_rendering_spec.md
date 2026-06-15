# NEPLg2 GUI font rendering specification

作成日: 2026-06-13

## 目的

この文書は GUI font rendering の public contract を定義する。詳細な table parsing、rasterization、layout algorithm は `gui_font_rendering_detailed_design.md` に置き、段階的な作業順序は `gui_font_rendering_implementation_plan.md` に置く。

NEPLg2 GUI は Web、native、bare、offscreen、headless で同じ application code を動かす。そのため、font は browser font object、DOM、Canvas2D text API、CoreText、DirectWrite、fontconfig を標準 API の authority にしない。Host は font bytes と resource access を提供し、NEPL 側の font/layout/rendering library が metrics、shaping、rasterization、描画後 bounds を計算する。

## Zenn 方針との対応

- platform 依存 resource access は `std/gui` と `platforms/gui/*` に閉じ込める。
- `core/gui` は no_alloc value、enum、struct、Result-friendly contract だけを持つ。
- 欠損や未対応は fallback せず、`Option` または typed `Result` として扱う。
- metrics、writing mode、font selection、glyph paint は raw number や string sentinel ではなく enum / struct で表す。
- error value と error display は分離する。
- 契約と現在の実装を文書で分ける。

## 必須 contract

### Font identity

Font identity は表示名や path suffix ではなく、登録済み resource と face index から作る。

```text
GuiFontFaceId:
    raw i32

GuiGlyphId:
    raw i32

GuiFontResourceId:
    raw i32

GuiResourceHash:
    raw i32

GuiFontResourcePath:
    path str

GuiFontResourceRequest:
    path GuiFontResourcePath
    face_index Option i32
    expected_hash Option GuiResourceHash
    decode_policy GuiFontDecodePolicy
```

`face_index` は collection font の face 選択を明示する。存在しない face index は先頭 face へ fallback せず error にする。

この文書で public name として使う font identity は `GuiFontFaceId`、`GuiGlyphId`、`GuiFontResourceId`、`GuiResourceHash`、`GuiFontResourcePath` に統一する。古い設計文書に残る `FontFaceId`、`GlyphId`、`ResourceHash` は同じ概念の旧表記であり、新規実装では使わない。

`GuiFontResourcePath` の `path str` は typed resource path string であり、display name、family name、path suffix、browser-provided font name の authority ではない。

`GuiFontResourcePath` の canonical 表記は leading slash を持たない resource path である。初期 bundled fixture は `fonts/HackGenConsoleNF-Regular.ttf` と表す。Web VFS の内部 file path はこの canonical path に `/` を 1 つだけ前置した `/fonts/HackGenConsoleNF-Regular.ttf` であり、public font identity ではない。Resource lookup は canonical path 全体の一致で行い、`HackGenConsoleNF-Regular.ttf` のような suffix match、font family name、browser-provided display name を authority として使ってはならない。

Resource path の normalization は platform boundary で一度だけ行う。空 path、absolute path、backslash、empty segment、`.`、`..` は typed error として拒否し、別 path へ推測変換しない。

### SFNT representative names

SFNT `name` table から得る display 用 metadata は、path suffix、browser-provided display name、OS font family lookup ではなく、font bytes 内の record だけを authority とする。

初期 slice が返す representative name は次に限定する。

```text
GuiSfntNames:
    family Option str      nameID 1
    subfamily Option str   nameID 2
    full_name Option str   nameID 4
```

選択順位は nameID ごとに次で固定する。

1. platformID 3, encodingID 1, languageID 0x0409: Windows Unicode BMP, UTF-16BE ASCII subset
2. platformID 3, any other encoding/language: selected if no rank 1 exists, then `UnsupportedNameEncoding`
3. platformID 1, encodingID 0, languageID 0: Macintosh Roman ASCII subset
4. platformID 1, any other encoding/language: selected if no higher rank exists, then `UnsupportedNameEncoding`

その他 platform の record は representative candidate ではない。nameID 1 / 2 / 4 の candidate が存在しない場合、その field は `Option::None` である。

`name` table が存在しない場合は `MissingTable`、format 0 以外は `UnsupportedNameTableFormat`、record range や UTF-16BE 奇数 length は `MalformedNameRecord`、ASCII subset 外の文字は `UnsupportedNameCharacter` で返す。空文字の selected representative は layout / UI display の metadata として不正なので `MalformedNameRecord` とする。

### SFNT cmap glyph mapping

SFNT `cmap` table から得る character-to-glyph mapping は、font bytes 内の mapping table だけを authority とする。Host font substitution、OS character map、browser text API、resource path suffix は使わない。

初期 slice は Unicode BMP の最小実用経路だけを扱う。

```text
gui_sfnt_lookup_glyph_id:
    bytes &ByteBuf
    face_index Option i32
    code_point i32
    -> Result GuiGlyphId GuiSfntParseError
```

Subtable selection は deterministic である。

1. `cmap` encoding record のうち platformID 3 / encodingID 1 を選択する。
2. 同じ platformID 3 / encodingID 1 が複数ある場合は最初に出現した record を使う。
3. それ以外の record は F4c の candidate ではない。対象 record がなければ `UnsupportedCmapEncoding` を返す。
4. 選択 record の subtable format が 4 でなければ `UnsupportedCmapTableFormat` を返す。別 record へ切り替えない。

`cmap` table 自体がない場合は `MissingTable` である。code point が BMP 外、つまり `0..65535` の範囲外の場合は、format 4 では表現できないので `UnsupportedCmapEncoding` で返す。BMP 内だが segment が存在しない場合、computed glyph id が 0 の場合、または glyphIdArray entry が 0 の場合は `MissingGlyphMapping` で返す。Glyph 0 を成功値として返してはならない。

Format 4 parser は `cmap` table header が declared table length 内に収まること、選択 subtable offset が encoding record array より後ろを指すこと、`length` が `cmap` table record 内に収まること、`segCountX2` が 0 でなく偶数であること、`reservedPad` が 0 であること、endCode / startCode / idDelta / idRangeOffset の各 array が subtable 内に収まること、idRangeOffset の指す glyphIdArray entry がその idRangeOffset word の位置から計算して subtable 内に収まることを検査する。不正な範囲は `MalformedCmapRecord` として返す。

### SFNT horizontal metrics

SFNT `hmtx` table から得る horizontal metrics は、font bytes 内の `hhea` / `maxp` / `hmtx` table だけを authority とする。Host text measurement、browser text API、fixed-cell test utility、glyph name、family name、path suffix は使わない。

初期 slice は horizontal writing の glyph advance と left side bearing だけを扱う。

```text
GuiSfntHorizontalMetric:
    glyph GuiGlyphId
    advance_width i32
    left_side_bearing i32

gui_sfnt_lookup_horizontal_metric:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    -> Result GuiSfntHorizontalMetric GuiSfntParseError
```

`GuiGlyphId` の public constructor は 1 以上の raw value だけを成功にする。SFNT metric lookup ではさらに `maxp.numGlyphs` に対して `1 <= glyphRaw < numGlyphs` を要求する。この project contract では glyph 0 を renderable glyph として成功させないため、glyph 0 や `glyphRaw >= numGlyphs` は `MissingGlyphMetric` である。防御的に forged value が 0 以下として渡された場合も `MissingGlyphMetric` とする。

`gui_sfnt_parse_metadata` は `hmtx` table record を directory summary に記録してよいが、`hmtx` decode を成功条件にしてはならない。F4a metadata の `hhea` 最小長は line metrics 用の 10 byte のままである。`hhea.numberOfHMetrics` を読むための `hhea.length >= 36` は `gui_sfnt_lookup_horizontal_metric` だけの契約である。

`hmtx` validation は declared table length に対して table-relative に行う。

- `hmtx` table がなければ `MissingTable`。
- `hhea.length < 36`、`numberOfHMetrics <= 0`、`numberOfHMetrics > numGlyphs` は `MalformedHmtxRecord`。
- 必要な `hmtx` declared length は `numberOfHMetrics * 4 + (numGlyphs - numberOfHMetrics) * 2` である。これを満たさない場合は `MalformedHmtxRecord`。
- `glyphRaw < numberOfHMetrics` の場合は `longHorMetric[glyphRaw]` から `advanceWidth u16` と `lsb i16` を読む。
- `glyphRaw >= numberOfHMetrics` の場合は最後の `longHorMetric` の `advanceWidth` と、後続 leftSideBearing array の `glyphRaw - numberOfHMetrics` 番目を読む。

File 末尾に余分な byte があっても、declared `hmtx.length` が不足している場合は成功にしてはならない。

### SFNT glyph header bounds

SFNT `loca` / `glyf` table から得る glyph bounds は、font bytes 内の `head` / `maxp` / `loca` / `glyf` table だけを authority とする。Host text measurement、browser text API、fixed-cell test utility、glyph name、family name、path suffix、別 glyph への置換は使わない。

初期 slice は glyph header の bounding box だけを扱う。contour flags、coordinate array、composite component、CFF / CFF2 charstring、rasterization は後続 phase で扱う。

```text
GuiSfntGlyphBounds:
    glyph GuiGlyphId
    x_min i32
    y_min i32
    x_max i32
    y_max i32

gui_sfnt_lookup_glyph_bounds:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    -> Result GuiSfntGlyphBounds GuiSfntParseError
```

`gui_sfnt_parse_metadata` は `loca` / `glyf` table record を directory summary に記録してよいが、glyph bounds lookup を成功条件にしてはならない。F4a metadata の `head` 最小長は `unitsPerEm` 用の 20 byte のままである。`head.indexToLocFormat` を読むための `head.length >= 52` は `gui_sfnt_lookup_glyph_bounds` だけの契約である。

`loca` / `glyf` validation は declared table length に対して table-relative に行う。

- `loca` または `glyf` table がなければ `MissingTable`。
- `head.length < 52` は `MalformedGlyfRecord`。
- `head.indexToLocFormat == 0` は short loca offset として u16 value を 2 倍する。
- `head.indexToLocFormat == 1` は long loca offset として u32 value を読む。u32 value が i32 範囲外なら `MalformedGlyfRecord`。
- `head.indexToLocFormat` が 0 / 1 以外なら `UnsupportedLocaFormat`。
- 必要な `loca` declared length は format 0 で `(numGlyphs + 1) * 2`、format 1 で `(numGlyphs + 1) * 4` である。これを満たさない場合は `MalformedGlyfRecord`。
- `glyphRaw <= 0` または `glyphRaw >= maxp.numGlyphs` は `MissingGlyphOutline`。
- glyph offset pair は `start <= end <= glyf.length` を満たす必要がある。`start > end` または `end > glyf.length` は `MalformedGlyfRecord`。
- `start == end` は empty glyph outline なので `MissingGlyphOutline`。
- glyph header は 10 byte 必須であり、`end - start < 10` は `MalformedGlyfRecord`。
- header の `xMin <= xMax` と `yMin <= yMax` が成り立たない場合は `MalformedGlyfRecord`。

File 末尾に余分な byte があっても、declared `loca.length` や `glyf.length` を越えて成功にしてはならない。

### SFNT simple glyph topology

SFNT `glyf` simple glyph の topology は、後続の flags / coordinate decode と rasterization の入力である。ここでは point stream 自体を解析せず、glyph header 後の contour endpoint array、instruction length、point data range だけを typed value として返す。

```text
GuiSfntSimpleGlyphTopology:
    glyph GuiGlyphId
    bounds GuiSfntGlyphBounds
    contour_count i32
    point_count i32
    instruction_length i32
    point_data_offset i32
    point_data_length i32

gui_sfnt_lookup_simple_glyph_topology:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    -> Result GuiSfntSimpleGlyphTopology GuiSfntParseError
```

`point_data_offset` は file absolute offset ではなく、`glyf` table-relative offset である。後続 decoder は `glyf.offset + point_data_offset` を file offset として使う。`point_data_length` も declared glyph range 内の相対長であり、file 末尾の余分な byte を使って補完してはならない。

Validation rules:

- F4e の `loca` / `glyf` validation と glyph header bounds validation を先に満たす。
- `numberOfContours < 0` は composite glyph または未対応 outline format なので `UnsupportedGlyphOutlineFormat`。
- `numberOfContours == 0` は成功値として使える outline がないため `MissingGlyphOutline`。
- `numberOfContours > 0` の場合、`endPtsOfContours[numberOfContours]` 全体が glyph range 内になければ `MalformedGlyfRecord`。
- endpoint は strict increasing でなければならない。
- `point_count = last_endpoint + 1` とし、`point_count > 0` でなければならない。overflow は `MalformedGlyfRecord`。
- `instructionLength` は u16 として読む。`instruction_length_offset + 2 + instructionLength <= glyph_end` を満たさない場合は `MalformedGlyfRecord`。
- `point_data_offset = instruction_start + instructionLength`、`point_data_length = glyph_end - point_data_offset` とする。
- `numberOfContours > 0` かつ `point_count > 0` なのに `point_data_length == 0` なら `MalformedGlyfRecord`。

F4f は flags / coordinate stream が「十分な長さを持つか」までは判定しない。flag repeat、x/y coordinate delta、contour point decode は後続 phase の責務である。ただし point stream が空の glyph を success にしてはならない。

### SFNT simple glyph point stream

SFNT simple glyph point stream は、flags の repeat 展開と x/y coordinate byte range を検査する段階である。ここでは coordinate value を復元せず、後続 decoder が読むべき raw byte range だけを typed value として返す。

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

gui_sfnt_lookup_simple_glyph_point_stream:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    -> Result GuiSfntSimpleGlyphPointStream GuiSfntParseError
```

すべての offset は file absolute offset ではなく `glyf` table-relative offset である。`flag_data_offset = topology.point_data_offset` であり、`flag_data_length` は expanded logical flag count ではなく、repeat count byte を含む raw flag stream の consumed byte length である。

Repeat semantics:

- repeat flag bit が立っている flag byte 自身は 1 point 分の flag である。
- 次 byte の repeat count は「追加で同じ flag を繰り返す point 数」である。
- `repeat_count = 0` は current flag 1 個だけを意味する。
- `logical_count + 1 + repeat_count > point_count` は `MalformedGlyfRecord`。
- `point_count` に達する前に glyph range が尽きる、または repeat count byte が glyph range 外なら `MalformedGlyfRecord`。
- `point_count` に達した直後の byte は flags ではなく x coordinate data の先頭として扱う。

Coordinate byte length:

- xShort bit が 1 の場合、x coordinate delta は 1 byte である。xSame / positive bit は sign として扱い、F4g では byte length には影響しない。
- xShort bit が 0 かつ xSame bit が 1 の場合、x coordinate delta は 0 byte である。
- xShort bit が 0 かつ xSame bit が 0 の場合、x coordinate delta は signed 16-bit なので 2 byte である。
- yShort / ySame も同じ規則を y coordinate に適用する。

Offset derivation:

```text
x_data_offset = flag_data_offset + flag_data_length
y_data_offset = x_data_offset + x_data_length
trailing_data_offset = y_data_offset + y_data_length
trailing_data_length = glyph_end - trailing_data_offset
```

`trailing_data_length < 0` は coordinate byte overrun なので `MalformedGlyfRecord`。`trailing_data_length >= 0` は success とし、padding / unused bytes として明示値で返す。後続 phase はこの trailing bytes を zero padding として要求するか、font sanitizer policy として扱うかを別途決める。F4g は trailing bytes を暗黙に fallback 消費しない。

### SFNT simple glyph single point decode

F4h は checked point stream range から 1 点の coordinate と flag state を復元する段階である。全点 `Vec` や outline builder は allocation failure と owner recovery の contract を別に設計してから実装する。F4h は allocation なしで 1 点を decode し、renderer や platform API には依存しない。

```text
GuiSfntSimpleGlyphPoint:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
    on_curve bool
    end_of_contour bool

gui_sfnt_lookup_simple_glyph_point:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    point_index i32
    -> Result GuiSfntSimpleGlyphPoint GuiSfntParseError
```

`point_index < 0` または `point_index >= topology.point_count` は、valid glyph に要求された point が存在しないことを表すため `MissingGlyphOutline` とする。font byte 構造の破損ではない。

一方、flag repeat byte 欠落、F4g で検証された `flag_data` / `x_data` / `y_data` range 外への read、endpoint array read failure、internal range inconsistency は `MalformedGlyfRecord` とする。

F4h は必ず F4g の point stream validation path を通り、そこから得た range 内だけを読む。public wrapper か内部共有 helper かは実装詳細だが、F4h が独自に unchecked flags / coordinate path を持ってはならない。

coordinate cursor は次のように定義する。

```text
flag_cursor = stream.flag_data_offset
x_cursor = stream.x_data_offset
y_cursor = stream.y_data_offset
current_x = 0
current_y = 0
```

logical point `0..point_index` を順番に処理し、それぞれの flag に応じて x/y cursor を進め、delta を `current_x` / `current_y` に累積する。repeat run の途中が target であっても、同じ run の target より前の repeated point の delta はすべて累積する。

delta decode:

```text
xShort == 1 and xPositive == 1: x_delta = u8
xShort == 1 and xPositive == 0: x_delta = -u8
xShort == 0 and xSame == 1: x_delta = 0
xShort == 0 and xSame == 0: x_delta = i16be

yShort == 1 and yPositive == 1: y_delta = u8
yShort == 1 and yPositive == 0: y_delta = -u8
yShort == 0 and ySame == 1: y_delta = 0
yShort == 0 and ySame == 0: y_delta = i16be
```

`on_curve` は flag bit 0 から得る。`end_of_contour` は topology の endpoint array と `point_index` の一致で判定する。F4h は trailing bytes を読まず、zero padding も要求しない。

### SFNT simple glyph contour span lookup

F4i は checked simple glyph topology から 1 つの contour が参照する logical point index range を返す段階である。full outline `Vec` や curve segment builder は allocation failure と owner recovery の contract を別に設計してから実装する。F4i は allocation なしで動作し、point stream decode や coordinate decode には依存しない。

```text
GuiSfntSimpleGlyphContourSpan:
    glyph GuiGlyphId
    contour_index i32
    start_point_index i32
    end_point_index i32
    point_count i32

gui_sfnt_lookup_simple_glyph_contour_span:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    -> Result GuiSfntSimpleGlyphContourSpan GuiSfntParseError
```

`end_point_index` は inclusive endpoint である。`point_count` は `end_point_index - start_point_index + 1` として定義する。

F4i は F4f の `gui_sfnt_glyf_simple_topology_with_tables` validation path にだけ依存する。F4g の point stream range や F4h の single point decode を呼ばない。contour 0 の `start_point_index` は 0 であり、contour n の start は contour n-1 の endpoint + 1 である。endpoint array read failure や topology validation で検出される endpoint 不整合は `MalformedGlyfRecord` とする。

`contour_index < 0` または `contour_index >= topology.contour_count` は、valid glyph に要求された contour が存在しないことを表すため `MissingGlyphOutline` とする。font byte 構造の破損ではない。

### SFNT simple glyph contour point lookup

F4j は checked contour span と checked point decode を合成し、contour-local point index から 1 点を復元する段階である。full point `Vec`、full contour `Vec`、curve segment builder、rasterizer は後続 phase の責務である。

```text
GuiSfntSimpleGlyphContourPoint:
    span GuiSfntSimpleGlyphContourSpan
    contour_point_index i32
    point GuiSfntSimpleGlyphPoint

gui_sfnt_lookup_simple_glyph_contour_point:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    contour_point_index i32
    -> Result GuiSfntSimpleGlyphContourPoint GuiSfntParseError
```

`contour_point_index` は contour-local index である。一方、nested `point.point_index` は glyph 全体での absolute logical point index である。absolute point index は次の式で定義する。

```text
absolute_point_index = span.start_point_index + contour_point_index
```

F4j は必ず次の順序で処理する。

```text
contour span lookup
    -> validate contour_point_index
    -> compute absolute_point_index
    -> point decode
```

`contour_point_index < 0` または `contour_point_index >= span.point_count` は `MissingGlyphOutline` とする。local index validation は point decode より先に行う。F4i / F4h 由来の byte 構造不整合は、それぞれの typed error をそのまま伝播する。

### SFNT simple glyph contour edge lookup

F4k は contour-local edge index から、contour 上で隣り合う 2 点を取得する段階である。この edge は topology 上の隣接 point pair であり、描画される直線 segment ではない。TrueType simple glyph では on-curve / off-curve の組み合わせにより quadratic curve や implied on-curve point が生じるため、curve segment classification、implied point 挿入、winding、rasterization は後続 phase の責務である。

```text
GuiSfntSimpleGlyphContourEdge:
    start GuiSfntSimpleGlyphContourPoint
    end GuiSfntSimpleGlyphContourPoint
    edge_index i32
    next_contour_point_index i32

gui_sfnt_lookup_simple_glyph_contour_edge:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphContourEdge GuiSfntParseError
```

`edge_index` は contour-local edge start index である。`start.contour_point_index == edge_index` であり、`end.contour_point_index == next_contour_point_index` である。nested `start.point.point_index` と `end.point.point_index` は glyph 全体での absolute logical point index のままである。

F4k は必ず次の順序で処理する。

```text
contour span lookup
    -> validate edge_index
    -> compute next_contour_point_index
    -> decode start contour point
    -> decode end contour point
```

`edge_index < 0` または `edge_index >= span.point_count` は `MissingGlyphOutline` とする。`next_contour_point_index` は次の式で定義する。

```text
next_contour_point_index =
    if edge_index + 1 == span.point_count then 0 else edge_index + 1
```

`span.point_count == 1` の contour は topology として有効な自己 wrap edge を返す。この場合、`edge_index == 0`、`next_contour_point_index == 0`、`start.point.point_index == end.point.point_index` である。ただし、これを描画可能な線分と見なしてはならない。後続の curve builder が renderability を判定する。

F4k は full edge `Vec`、full contour `Vec`、curve segment builder、rasterizer を作らない。F4i / F4j / F4h 由来の byte 構造不整合は、それぞれの typed error をそのまま伝播する。

### SFNT simple glyph curve segment classification

F4l は F4k の topology edge を、TrueType simple glyph の on-curve / off-curve 規則で 1 つの curve segment state に分類する段階である。この段階はまだ full outline `Vec`、streaming contour sink、winding、rasterization を作らない。分類結果は drawable な line / quadratic だけでなく、valid topology だが edge start からは drawable segment を出さない `NoSegment` も enum payload として返す。

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

`*_x2` / `*_y2` は font unit の 2 倍である。通常の decoded point は `x * 2` / `y * 2` として保持する。2 つの off-curve point の midpoint が implied on-curve endpoint になる場合、`end_x2 = control.x + lookahead.x`、`end_y2 = control.y + lookahead.y` とする。これにより odd midpoint も exact に表現し、整数除算による丸めや fallback を行わない。

Pure classifier:

```text
gui_sfnt_classify_simple_glyph_curve_segment:
    edge GuiSfntSimpleGlyphContourEdge
    lookahead Option GuiSfntSimpleGlyphContourPoint
    -> GuiSfntSimpleGlyphCurveSegment
```

Byte lookup:

```text
gui_sfnt_lookup_simple_glyph_curve_segment:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphCurveSegment GuiSfntParseError
```

分類規則は次である。

```text
span.point_count == 1
    -> NoSegment SinglePointContour

start.on_curve == false
    -> NoSegment OffCurveStart

start.on_curve == true and end.on_curve == true
    -> Line

start.on_curve == true and end.on_curve == false and lookahead is None
    -> NoSegment MissingLookahead

start.on_curve == true and end.on_curve == false and lookahead.on_curve == true
    -> Quadratic with explicit on-curve end

start.on_curve == true and end.on_curve == false and lookahead.on_curve == false
    -> Quadratic with implied midpoint end
```

`gui_sfnt_lookup_simple_glyph_curve_segment` は `edge.end` が off-curve で、かつ `edge.start` が on-curve の場合だけ lookahead point を読む。line、single point contour、off-curve start では不要な lookahead decode を行わない。これにより、関係しない後続 coordinate corruption を現在 edge の error として露出しない。

`NoSegment` は parse error ではない。`edge_index` や `contour_index` が範囲外の場合、または必要な byte range が壊れている場合だけ `Result::Err GuiSfntParseError` とする。

### SFNT simple glyph path command projection

F4m は F4l の `GuiSfntSimpleGlyphCurveSegment` を、後続の outline / path sink が読む typed command に写す段階である。この段階でも full outline `Vec`、streaming sink trait、winding、fill rule、rasterization、2D renderer command への変換は行わない。

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

Projection API:

```text
gui_sfnt_simple_glyph_curve_segment_move_to_command:
    segment &GuiSfntSimpleGlyphCurveSegment
    -> GuiSfntSimpleGlyphPathCommand

gui_sfnt_simple_glyph_curve_segment_draw_command:
    segment &GuiSfntSimpleGlyphCurveSegment
    -> GuiSfntSimpleGlyphPathCommand
```

`gui_sfnt_simple_glyph_curve_segment_move_to_command` は `Line` / `Quadratic` を segment start の `MoveTo` に写す。`gui_sfnt_simple_glyph_curve_segment_draw_command` は `Line` を `LineTo`、`Quadratic` を `QuadraticTo` に写す。`NoSegment` はどちらの関数でも `SkipNoSegment` に写す。

この API は command index を受け取らず、`Option` や `Result` も返さない。invalid index という状態を API から消し、caller は move phase と draw phase を明示的に選ぶ。これにより「1 segment から暗黙の current point state を持つ command list が出る」ような設計を避ける。

`MoveTo` は current subpath の開始位置を明示する command であり、これ単体では線や mask を描画しない。`LineTo` と `QuadraticTo` は F4l で確定済みの doubled coordinate をそのまま使う。`SkipNoSegment` は fallback drawing ではなく、valid topology だが現在 edge から path command を発行しないことを後続 sink に伝える typed command である。

Path command payload は元の `GuiSfntSimpleGlyphContourEdge` / `GuiSfntSimpleGlyphLineSegment` / `GuiSfntSimpleGlyphQuadraticSegment` 全体を再保持しない。後続 sink に必要な source contour/edge index、doubled coordinate、no-segment reason だけを保持する。これは full topology value の再コピーを避け、projection 層を小さな値の列として扱えるようにするためである。

F4m は `Vec GuiSfntSimpleGlyphPathCommand` を作らない。caller は `move_to_command` と `draw_command` を明示的に呼び分け、必要な command を 1 つずつ取得する。これにより headless / bare / web / native のどの backend でも同じ pure projection を使える。

### SFNT simple glyph path command public lookup

F4n は SFNT byte input から contour-local edge の path command を 1 つ読む public lookup layer である。この段階でも full outline `Vec`、command list、sink trait、winding、fill rule、rasterization、2D renderer command への変換は行わない。

```text
gui_sfnt_lookup_simple_glyph_move_to_command:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathCommand GuiSfntParseError

gui_sfnt_lookup_simple_glyph_draw_command:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathCommand GuiSfntParseError
```

F4n は `gui_sfnt_lookup_simple_glyph_curve_segment` を呼び、成功した `GuiSfntSimpleGlyphCurveSegment` を F4m の `move_to_command` または `draw_command` に渡す。`gui_sfnt_parse_metadata`、`*_with_tables` helper、point / contour decode helper、curve classification logic を F4n で重複実装してはならない。

`gui_sfnt_lookup_simple_glyph_curve_segment` が `Result::Err` を返した場合、F4n は同じ `GuiSfntParseError` を伝播する。`NoSegment` は parse error ではないため、F4n でも `Result::Ok (SkipNoSegment ...)` として返す。`Option::None`、empty command、silent no-op、fallback drawing にはしない。

### SFNT simple glyph path command pair lookup

F4o は同じ contour-local edge について、move command と draw command を O(1) の pair value として取得する層である。これは contour stream、command sequence、full outline、sink trait ではない。command index、count、next pointer、current point state は導入しない。

```text
GuiSfntSimpleGlyphPathCommandPair:
    move_command GuiSfntSimpleGlyphPathCommand
    draw_command GuiSfntSimpleGlyphPathCommand
```

Pure projection API:

```text
gui_sfnt_simple_glyph_curve_segment_path_command_pair:
    segment &GuiSfntSimpleGlyphCurveSegment
    -> GuiSfntSimpleGlyphPathCommandPair
```

Byte-backed public lookup:

```text
gui_sfnt_lookup_simple_glyph_path_command_pair:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathCommandPair GuiSfntParseError
```

F4o の public lookup は `gui_sfnt_lookup_simple_glyph_curve_segment` を 1 回だけ呼び、成功した segment を `gui_sfnt_simple_glyph_curve_segment_path_command_pair` に渡す。F4n の move lookup と draw lookup を別々に呼ぶと同じ SFNT edge decode が 2 回走るため、pair lookup は後続 sink が single-edge boundary を効率よく読むための API である。

`NoSegment` は pair 内の `move_command` と `draw_command` の両方で `SkipNoSegment` になる。これは parse error、empty command、silent no-op ではない。

F4o は `Vec GuiSfntSimpleGlyphPathCommand` を作らない。`gui_sfnt_parse_metadata`、`*_with_tables` helper、lower public lookup、curve classifier、renderer、rasterizer、platform API を F4o の public helper に混ぜてはならない。

### SFNT simple glyph path sink event adapter

F4p は `GuiSfntSimpleGlyphPathCommandPair` を、後続の contour/path sink が読む event pair へ写す single-edge adapter である。これは F5 の contour stream ではなく、glyph outline 全体の command sequence でもない。F4p が定義する順序は 1 pair 内の `first_event` と `second_event` だけであり、contour closure、off-curve contour-start synthesis、winding、fill rule、rasterizer、render2d command、platform API は扱わない。

```text
GuiSfntSimpleGlyphPathSinkEvent:
    Command GuiSfntSimpleGlyphPathCommand

GuiSfntSimpleGlyphPathSinkEventPair:
    first_event GuiSfntSimpleGlyphPathSinkEvent
    second_event GuiSfntSimpleGlyphPathSinkEvent
```

`GuiSfntSimpleGlyphPathSinkEvent` は新しい path command 表現ではない。F4m/F4o の compact `GuiSfntSimpleGlyphPathCommand` を sink-facing event boundary として包むだけである。したがって `MoveTo`、`LineTo`、`QuadraticTo`、`SkipNoSegment` の payload を再定義しない。

```text
gui_sfnt_simple_glyph_path_command_sink_event command
    -> GuiSfntSimpleGlyphPathSinkEvent::Command command

gui_sfnt_simple_glyph_path_command_pair_sink_event_pair pair
    -> first = gui_sfnt_simple_glyph_path_command_sink_event pair.move_command
    -> second = gui_sfnt_simple_glyph_path_command_sink_event pair.draw_command
    -> GuiSfntSimpleGlyphPathSinkEventPair first second
```

pure projection は total であり、`Option` や `Result` を返さない。valid pair value に対して first / second event は必ず存在する。`SkipNoSegment` も `Command (SkipNoSegment ...)` として保持し、empty event や silent skip にはしない。

F4p は `Vec GuiSfntSimpleGlyphPathSinkEvent` を作らない。`command_index`、`count`、`next`、mutable current point state、`push`、`gui_sfnt_lookup_simple_glyph_path_command_pair`、`gui_sfnt_lookup_simple_glyph_curve_segment`、metadata parser、`*_with_tables` helper、lower point / contour helper、curve classifier、renderer、rasterizer、platform API を F4p の pure helper に混ぜてはならない。

### SFNT simple glyph path sink event kind classification

F4q は F4p の `GuiSfntSimpleGlyphPathSinkEvent` を、後続 sink の dispatch 用分類値へ写す段階である。これは path command payload の軽量版ではなく、描画座標、source contour / edge、rasterization state、current point state の authority ではない。実 payload は常に `GuiSfntSimpleGlyphPathSinkEvent` から `GuiSfntSimpleGlyphPathCommand` を読む側に残す。

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

`SkipNoSegment` kind は diagnostics、skip counting、branch selection のために reason だけを保持する。これは source contour / edge を復元する値ではない。`contour_index`、`edge_index`、`x2`、`y2`、`control_x2`、`end_x2` などが必要な caller は kind ではなく既存 command payload を読む。

```text
gui_sfnt_simple_glyph_path_sink_event_kind event
    -> match event.command:
        MoveTo -> GuiSfntSimpleGlyphPathSinkEventKind::MoveTo
        LineTo -> GuiSfntSimpleGlyphPathSinkEventKind::LineTo
        QuadraticTo -> GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo
        SkipNoSegment skip -> GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment skip.reason

gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair pair
    -> first_kind = gui_sfnt_simple_glyph_path_sink_event_kind pair.first_event
    -> second_kind = gui_sfnt_simple_glyph_path_sink_event_kind pair.second_event
    -> GuiSfntSimpleGlyphPathSinkEventKindPair first_kind second_kind
```

F4q の pure helper は total であり、`Option` や `Result` を返さない。valid event は必ず既存 command を包んでいるため kind も必ず決まる。F4q は `Vec GuiSfntSimpleGlyphPathSinkEventKind`、`push`、command index、count、next、current point state、contour closure、winding、fill rule、byte lookup、metadata parser、`*_with_tables` helper、curve classifier、renderer、rasterizer、platform API を導入しない。

### SFNT simple glyph path sink event indexed selection

F4r は F4p/F4q の two-slot pair から、後続 sink が `First` または `Second` を O(1) に選択する境界である。これは numeric index を受け取る iterator ではなく、contour stream、command count、next pointer、current point state、contour closure を定義しない。

```text
GuiSfntSimpleGlyphPathSinkEventSlot:
    First
    Second

gui_sfnt_simple_glyph_path_sink_event_pair_event_at:
    pair GuiSfntSimpleGlyphPathSinkEventPair
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> GuiSfntSimpleGlyphPathSinkEvent

gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at:
    pair GuiSfntSimpleGlyphPathSinkEventKindPair
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> GuiSfntSimpleGlyphPathSinkEventKind

gui_sfnt_simple_glyph_path_sink_event_pair_kind_at:
    pair GuiSfntSimpleGlyphPathSinkEventPair
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> GuiSfntSimpleGlyphPathSinkEventKind
```

slot は enum なので、存在しない third event や負の index は型として表現できない。したがって F4r の selection helper は `Option` / `Result` を返さない。`event_pair_event_at` は slot を明示的に `match` し、`First` なら first event accessor、`Second` なら second event accessor だけを使う。`kind_pair_kind_at` も kind pair の first / second accessor だけを使う。`event_pair_kind_at` は `event_pair_event_at` と `gui_sfnt_simple_glyph_path_sink_event_kind` の合成だけであり、kind pair を作らずに single slot だけ dispatch したい caller のための total helper である。

F4r は numeric `i32` index、`Option`、`Result`、`Vec`、`push`、command index、count、next、current point state、contour traversal、contour closure、off-curve contour-start synthesis、byte lookup、metadata parser、`*_with_tables` helper、curve classifier、renderer、rasterizer、render2d、platform API を導入しない。

### SFNT simple glyph path contour traversal step

F4s は F4r の typed slot selection の上に、1 contour 内の 1 sink event step だけを読む境界である。これは full outline builder ではなく、`Vec` による command list、mutable current point state、rasterizer、render2d、platform API へ進まない。cursor は glyph / contour / edge / slot を持ち、step は cursor、event、kind、next state を返す。

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

gui_sfnt_lookup_simple_glyph_path_contour_step:
    bytes &ByteBuf
    face_index Option i32
    cursor GuiSfntSimpleGlyphPathContourCursor
    -> Result GuiSfntSimpleGlyphPathContourStep GuiSfntParseError
```

public lookup は `gui_sfnt_lookup_simple_glyph_contour_span` で contour と point count を検証し、`gui_sfnt_lookup_simple_glyph_path_command_pair` で cursor の edge を 1 回だけ path command pair に変換する。成功した pair は `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair` に渡し、F4r の `gui_sfnt_simple_glyph_path_sink_event_pair_event_at` と F4q の `gui_sfnt_simple_glyph_path_sink_event_kind` を使って event / kind を得る。

next state は domain enum であり、contour の終端を `Option::None` や error で表さない。`slot First -> same edge Second`、`slot Second -> edge + 1 First or EndContour` が契約である。range 不正、glyph 欠落、table 破損だけが `Result::Err GuiSfntParseError` になる。成功した final event は `Result::Ok step` であり、`step.next = EndContour` で終端を表す。

next 計算の pure helper は public contract ではなく、public lookup が `span_point_count > 0` と `0 <= edge_index < span_point_count` を検証した後だけ呼ぶ private helper である。検証されていない raw cursor に対して total public helper を見せてはならない。

F4s は off-curve contour-start synthesis、contour closure command insertion、real path sink ownership、full outline allocation、font fallback、renderer command generation を扱わない。off-curve start は既存の `SkipNoSegment OffCurveStart` として typed event に残り、後続 phase が synthesis policy を決める。

### SFNT simple glyph allocation-free path sink ownership boundary

F4t は F4s の `GuiSfntSimpleGlyphPathContourStep` を、実際の path sink が 1 step ずつ消費できる ownership boundary へ写す段階である。これは real sink trait、full outline builder、`Vec` command stream、rasterizer、renderer command ではない。F4t の責務は、F4s step を「primary action」と「tail action」に分け、off-curve contour start と contour close の policy を enum data として明示することである。

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

policy reject は `GuiSfntParseError` ではない。byte-backed helper の `Result::Err GuiSfntParseError` は F4s lookup 由来の parse / range error だけを表す。`RejectUnsupported` による拒否は `Result::Ok GuiSfntSimpleGlyphPathSinkStep` の `primary_action = Reject UnsupportedOffCurveStart` として返す。

`GuiSfntSimpleGlyphPathOffCurveStartPolicy` が作用するのは `GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment OffCurveStart` だけである。`SinglePointContour` と `MissingLookahead` は F4l/F4s の typed success であり、`RejectUnsupported` でも `EmitEvent` のまま保持する。これにより unsupported off-curve contour-start synthesis を、他の no-segment reason に誤って広げない。

tail action の規則は次で固定する。

```text
primary = Reject _
    -> tail_action = NoTailAction

primary = EmitEvent _ and step.next = Continue _
    -> tail_action = NoTailAction

primary = EmitEvent _ and step.next = EndContour and closure_policy = KeepOpen
    -> tail_action = NoTailAction

primary = EmitEvent _ and step.next = EndContour and closure_policy = EmitCloseAfterFinalEvent
    -> tail_action = CloseContour glyph contour_index
```

つまり reject と close contour は同時に発生しない。`CloseContour` は final event 後だけの tail marker であり、途中の `Continue` step では絶対に発行しない。

public pure helper と byte-backed helper は次である。

```text
gui_sfnt_simple_glyph_path_sink_step_from_contour_step:
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    step &GuiSfntSimpleGlyphPathContourStep
    -> GuiSfntSimpleGlyphPathSinkStep

gui_sfnt_lookup_simple_glyph_path_sink_step:
    bytes &ByteBuf
    face_index Option i32
    cursor GuiSfntSimpleGlyphPathContourCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkStep GuiSfntParseError
```

byte-backed helper は `gui_sfnt_lookup_simple_glyph_path_contour_step` を呼び、成功した F4s step を pure helper に渡すだけである。metadata unwrap、table helper bypass、renderer、platform API、font fallback、rasterization、full outline allocation は行わない。

### SFNT simple glyph path sink action selection projection

F4u は F4t の `GuiSfntSimpleGlyphPathSinkStep` から、future sink が順に消費する action を typed value として選択する projection である。これは real sink mutation、callback、full outline allocation、`Vec` command stream、renderer command、rasterizer ではない。F4u の責務は、F4t で分離した primary / tail action を同一の `GuiSfntSimpleGlyphPathSinkAction` 型に写し、slot による選択を enum / match で固定することである。

```text
GuiSfntSimpleGlyphPathSinkActionSlot:
    Primary
    Tail

GuiSfntSimpleGlyphPathSinkAction:
    EmitEvent GuiSfntSimpleGlyphPathSinkEvent
    Reject GuiSfntSimpleGlyphPathSinkRejectReason
    CloseContour GuiSfntSimpleGlyphPathContourClose
    NoAction
```

`GuiSfntSimpleGlyphPathSinkActionSlot` は F4r/F4s の `GuiSfntSimpleGlyphPathSinkEventSlot` とは別の軸である。`GuiSfntSimpleGlyphPathSinkEventSlot::First` / `Second` は contour edge 内の command event を選ぶ。`GuiSfntSimpleGlyphPathSinkActionSlot::Primary` / `Tail` は F4t sink step 内の action を選ぶ。両者を同じ enum や数値 index に統合してはならない。

`NoAction` は `GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction` の明示的な projection だけを表す。fallback、silent no-op、unsupported feature の握りつぶしではない。primary action projection は `NoAction` を返さず、必ず `EmitEvent` または `Reject` を返す。

public pure helper と byte-backed helper は次である。

```text
gui_sfnt_simple_glyph_path_sink_primary_action_as_action:
    action &GuiSfntSimpleGlyphPathSinkPrimaryAction
    -> GuiSfntSimpleGlyphPathSinkAction

gui_sfnt_simple_glyph_path_sink_tail_action_as_action:
    action &GuiSfntSimpleGlyphPathSinkTailAction
    -> GuiSfntSimpleGlyphPathSinkAction

gui_sfnt_simple_glyph_path_sink_step_action_at:
    step &GuiSfntSimpleGlyphPathSinkStep
    slot GuiSfntSimpleGlyphPathSinkActionSlot
    -> GuiSfntSimpleGlyphPathSinkAction

gui_sfnt_lookup_simple_glyph_path_sink_action:
    bytes &ByteBuf
    face_index Option i32
    cursor GuiSfntSimpleGlyphPathContourCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    slot GuiSfntSimpleGlyphPathSinkActionSlot
    -> Result GuiSfntSimpleGlyphPathSinkAction GuiSfntParseError
```

`gui_sfnt_simple_glyph_path_sink_step_action_at` は `Primary` / `Tail` の網羅的 `match` だけで分岐する。`Option`、`Result`、数値 `command_index`、default branch は使わない。

byte-backed helper は `gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び、成功した step に `gui_sfnt_simple_glyph_path_sink_step_action_at` を適用する。下位の contour / curve / table helper、metadata unwrap、`*_with_tables` bypass、font fallback、renderer、platform API、rasterization、full outline allocation は行わない。

### SFNT simple glyph path sink action traversal step

F4v は F4u の 1 action projection を、contour 内で順に読むための traversal step へ拡張する段階である。これは real sink mutation、callback、full outline allocation、`Vec` command stream、renderer command、rasterizer ではない。F4v の責務は、`Primary -> Tail -> F4s source next` の遷移を enum data として固定することである。

```text
GuiSfntSimpleGlyphPathSinkActionCursor:
    contour_cursor GuiSfntSimpleGlyphPathContourCursor
    action_slot GuiSfntSimpleGlyphPathSinkActionSlot

GuiSfntSimpleGlyphPathSinkActionNext:
    Continue GuiSfntSimpleGlyphPathSinkActionCursor
    EndContour

GuiSfntSimpleGlyphPathSinkActionStep:
    cursor GuiSfntSimpleGlyphPathSinkActionCursor
    sink_step GuiSfntSimpleGlyphPathSinkStep
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionNext
```

`GuiSfntSimpleGlyphPathSinkActionCursor` は既存の checked `GuiSfntSimpleGlyphPathContourCursor` と F4u の action slot を合成した cursor である。新しい数値 action index、command index、loop counter、ad-hoc traversal counter は導入しない。既存 contour cursor が持つ `contour_index` / `edge_index` は、F4s で検証される contour event traversal の authority として保持する。

next の規則は action payload と独立している。

```text
action_slot = Primary
    -> Continue same contour_cursor Tail

action_slot = Tail and sink_step.source_step.next = Continue next_cursor
    -> Continue next_cursor Primary

action_slot = Tail and sink_step.source_step.next = EndContour
    -> EndContour
```

`Primary -> Tail` は primary action が `EmitEvent` でも `Reject` でも同じである。`Tail -> source_step.next` は tail action が `CloseContour` でも `NoAction` でも同じである。action value は future sink が何を消費するかを表し、next value はどこへ進むかだけを表す。

public pure helper と byte-backed helper は次である。

```text
gui_sfnt_simple_glyph_path_sink_action_next_from_step:
    sink_step &GuiSfntSimpleGlyphPathSinkStep
    action_slot GuiSfntSimpleGlyphPathSinkActionSlot
    -> GuiSfntSimpleGlyphPathSinkActionNext

gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step:
    sink_step &GuiSfntSimpleGlyphPathSinkStep
    action_slot GuiSfntSimpleGlyphPathSinkActionSlot
    -> GuiSfntSimpleGlyphPathSinkActionStep

gui_sfnt_lookup_simple_glyph_path_sink_action_step:
    bytes &ByteBuf
    face_index Option i32
    cursor GuiSfntSimpleGlyphPathSinkActionCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStep GuiSfntParseError
```

`gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` は action selection を F4u の `gui_sfnt_simple_glyph_path_sink_step_action_at` に委譲し、primary / tail action の中身を再分類しない。

byte-backed helper は action cursor から contour cursor と action slot を読み、`gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び、成功した step に pure action step projection を適用する。下位の contour / curve / table helper、metadata unwrap、`*_with_tables` bypass、font fallback、renderer、platform API、rasterization、full outline allocation は行わない。

### SFNT simple glyph path sink action start cursor

F4w は F4v の action traversal に、contour-local action stream の開始 cursor を与える段階である。これは glyph outline の列挙、sink mutation、action payload lookup、policy evaluation、allocation、rasterization ではない。開始位置は必ず contour edge `0`、event slot `First`、action slot `Primary` である。

public helper は pure constructor と byte-backed validated entry point に分ける。

```text
gui_sfnt_simple_glyph_path_sink_action_start_cursor:
    glyph GuiGlyphId
    contour_index i32
    -> GuiSfntSimpleGlyphPathSinkActionCursor

gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    -> Result GuiSfntSimpleGlyphPathSinkActionCursor GuiSfntParseError
```

`gui_sfnt_simple_glyph_path_sink_action_start_cursor` は unchecked value constructor である。`GuiSfntSimpleGlyphPathContourCursor` を `edge_index = 0` / `GuiSfntSimpleGlyphPathSinkEventSlot::First` で作り、それを `GuiSfntSimpleGlyphPathSinkActionSlot::Primary` と合成する。byte 妥当性、contour 存在、point 数、span 範囲は検査しない。

`gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor` は byte-backed entry point であり、`gui_sfnt_lookup_simple_glyph_contour_span` を 1 回だけ呼ぶ。成功した場合にだけ pure start cursor helper へ委譲する。最初の action payload は読まず、F4v action step lookup、F4t sink step lookup、F4s contour step lookup、point / curve / path command helper、sink policy、renderer、rasterizer、platform font API は呼ばない。

この分離により、開始位置の型構成は cheap test で確認でき、byte 妥当性は既存 contour span contract に集約される。pure constructor が contour の存在や byte 妥当性を証明するものとして document してはならない。

### SFNT simple glyph path sink action start step

F4x は F4w の start cursor と F4v の action step lookup を接続し、contour の first action step を読む convenience entry point を追加する段階である。これは contour stream、real sink、full outline allocation、command list、renderer、rasterizer ではない。

public helper は次である。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_step:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStep GuiSfntParseError
```

この helper は `gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index` で unchecked start cursor を作り、`gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy` を 1 回だけ呼ぶ。byte-backed start cursor helper は呼ばない。理由は、`gui_sfnt_lookup_simple_glyph_path_sink_action_step` が既に F4t/F4s 経由で contour span 検証、edge range 検証、path command lookup、policy decision を行うためである。ここで byte-backed start cursor helper を先に呼ぶと contour span 検証が二重になる。

`Result::Err` は下位 action step lookup の parse/range error をそのまま伝播する。policy reject は `Result::Err` ではなく `Result::Ok GuiSfntSimpleGlyphPathSinkActionStep` の `action = Reject` payload として残る。F4x は error taxonomy を変更してはならない。

F4x は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`、`gui_sfnt_lookup_simple_glyph_contour_span`、`gui_sfnt_lookup_simple_glyph_path_sink_step`、F4s/F4t より下位の lookup helper を直接呼ばない。検証と payload construction の authority は F4v action step lookup に集約する。

### SFNT simple glyph path sink action step advance

F4y は F4v の `GuiSfntSimpleGlyphPathSinkActionStep.next` を、byte-backed lookup 済みの次 step または contour 終端へ 1 段だけ進める段階である。これは loop traversal、iterator、real sink、full outline allocation、renderer、rasterizer ではない。

terminal state は `Option::None` や `Result::Err` ではなく、専用 enum で表す。

```text
GuiSfntSimpleGlyphPathSinkActionStepAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionStep
    EndContour

gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance:
    bytes &ByteBuf
    face_index Option i32
    step &GuiSfntSimpleGlyphPathSinkActionStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepAdvance GuiSfntParseError
```

helper は `gui_sfnt_simple_glyph_path_sink_action_step_next step` を読み、その enum だけを `match` する。

```text
next = Continue cursor
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step

next = EndContour
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour
```

`Result::Err` は `Continue cursor` の下位 action step lookup から来る parse/range/table error だけを伝播する。`EndContour` は successful terminal state であり、error ではない。policy reject は次 step の `action = Reject` payload として残り、F4y が `Reject` / `NoAction` / `CloseContour` を見て traversal を変えることは禁止する。

F4y は start cursor helper、start step helper、sink action lookup、sink step lookup、contour step lookup、下位 point / curve / path helper、metadata parser、`*_with_tables` helper、`Vec` / `push`、renderer、rasterizer、platform API を直接呼ばない。

### SFNT simple glyph path sink action step item

F4z は現在の `GuiSfntSimpleGlyphPathSinkActionStep` と、F4y で byte-backed lookup 済みになった `GuiSfntSimpleGlyphPathSinkActionStepAdvance` を 1 つの typed item に束ねる段階である。これは後続の real sink / contour stream が読む 1 action 分の安定した入力単位であり、loop traversal、real sink mutation、callback、`Vec` command list、full outline allocation、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphPathSinkActionStepItem:
    step GuiSfntSimpleGlyphPathSinkActionStep
    advance GuiSfntSimpleGlyphPathSinkActionStepAdvance

gui_sfnt_lookup_simple_glyph_path_sink_action_step_item:
    bytes &ByteBuf
    face_index Option i32
    step &GuiSfntSimpleGlyphPathSinkActionStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntParseError
```

helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy` にだけ委譲する。`Result::Err` はそのまま伝播し、`Result::Ok advance` なら現在 step を明示コピーして `GuiSfntSimpleGlyphPathSinkActionStepItem` に格納する。

F4z helper は action payload を見ない。`Reject`、`NoAction`、`CloseContour` の処理、start step composition、contour-wide traversal、event emission、sink mutation、allocation failure recovery は後続 phase の責務である。F4z は start cursor/start step helper、F4v action step lookup、sink action lookup、sink step lookup、contour step lookup、下位 point / curve / path helper、metadata parser、`*_with_tables` helper、`Vec` / `push`、renderer、rasterizer、platform API を直接呼ばない。

### SFNT simple glyph path sink action start item

F4aa は F4x の first action step lookup と F4z の action step item lookup を接続し、contour の最初の action item を読む public helper を追加する段階である。これは contour-wide traversal、iterator、real sink、callback、command list、full outline allocation、renderer、rasterizer ではない。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_item:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntParseError
```

helper は次の 2 段だけを行う。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy
    Err error -> Err error
    Ok start_step:
        gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy
            Err error -> Err error
            Ok item -> Ok item
```

`gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` は 1 回だけ呼び、`gui_sfnt_lookup_simple_glyph_path_sink_action_step_item` も 1 回だけ呼ぶ。F4aa helper 自体は start cursor を作らず、action payload を読まず、`Reject` / `NoAction` / `CloseContour` で traversal を変えない。contour 終端は F4z item 内の `GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` に残り、`Option::None` や hidden no-op へ変換しない。

F4aa は action start cursor helper、F4v action step lookup、F4y advance helper、sink action lookup、sink step lookup、contour step lookup、下位 point / curve / path helper、metadata parser、`*_with_tables` helper、`Vec` / `push`、renderer、rasterizer、platform API を直接呼ばない。検証と action payload construction の authority は F4x/F4z の既存境界に残す。

### SFNT simple glyph path sink action item next

F4ab は F4z/F4aa で得た action item の checked advance を、次の action item または contour terminal state へ 1 段だけ解決する段階である。これは contour-wide loop、iterator owner、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphPathSinkActionItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionStepItem
    EndContour
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_item_next:
    bytes &ByteBuf
    face_index Option i32
    item &GuiSfntSimpleGlyphPathSinkActionStepItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntParseError
```

helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_advance item` を 1 回だけ読み、`Continue next_step` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &next_step policy` を 1 回だけ呼ぶ。`EndContour` は successful terminal state なので `Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` として返す。

F4ab は action payload を読まない。`Reject`、`NoAction`、`CloseContour` は future sink consumer が消費する payload であり、次 item の有無を決める authority ではない。`EndContour` を `Result::Err`、`Option::None`、silent no-op、fallback branch に変換してはならない。

F4ab は start cursor / start step / start item helper、F4v action step lookup、F4y advance helper、sink action lookup、sink step lookup、contour step lookup、下位 point / curve / path helper、metadata parser、`*_with_tables` helper、`Vec` / `push`、renderer、rasterizer、platform API を直接呼ばない。検証と next step construction の authority は F4z item が保持する checked advance と F4z step item lookup に残す。

### SFNT simple glyph path sink action consumer item

F4ac は F4z/F4aa の action item から、future sink consumer が 1 action 分として読む packet を作る段階である。F4ab が「どこへ進むか」だけを返すのに対し、F4ac は「今回何を消費するか」と「次にどこへ進むか」を同じ typed value に束ねる。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItem:
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item:
    bytes &ByteBuf
    face_index Option i32
    item &GuiSfntSimpleGlyphPathSinkActionStepItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError
```

helper は `item.step` を current action の copy のためだけに読み、`gui_sfnt_simple_glyph_path_sink_action_step_action` で action を取得する。次状態は `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next` を 1 回だけ呼んで得る。`Result::Err` はそのまま伝播し、`Result::Ok next` の場合だけ `GuiSfntSimpleGlyphPathSinkActionConsumerItem action next` を返す。

F4ac は real sink、iterator、contour-wide consumer、callback、mutable sink state、command list、full outline allocation、renderer、rasterizer ではない。`EmitEvent`、`Reject`、`NoAction`、`CloseContour` の payload は解釈せず、packet の `action` に保持する。unsupported や terminal を hidden fallback、silent no-op、`Option::None` に変換してはならない。

F4ac は F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables` helper、`Vec` / `push`、loop、renderer、rasterizer、platform API、host text measurement、font fallback を直接呼ばない。後続の real sink はこの packet を consume するが、この phase では consume policy、sink owner、allocation recovery、contour closure、winding / fill rule をまだ定義しない。

### SFNT simple glyph path sink action consumer item next

F4ad は F4ac の consumer item を 1 段だけ進め、次の consumer item または contour terminal state を返す段階である。これは contour-wide loop、iterator owner、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    EndContour
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next:
    bytes &ByteBuf
    face_index Option i32
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItemNext GuiSfntParseError
```

helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item` を 1 回だけ読む。`Continue next_item` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy` を 1 回だけ呼び、`Result::Ok next_consumer_item` を `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer_item` として返す。`EndContour` は successful terminal state なので `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour` として返す。

F4ad は current action を読まず、`EmitEvent`、`Reject`、`NoAction`、`CloseContour` payload を解釈しない。F4ad の authority は F4ac packet が保持する checked next state と、次 packet を構成する F4ac helper に限定する。terminal state を `Result::Err`、`Option::None`、silent no-op、fallback branch に変換してはならない。

F4ad は F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables` helper、`Vec` / `push`、loop、current point state、renderer、rasterizer、platform API、host text measurement、font fallback を直接呼ばない。後続の contour-wide consumer は F4ad を繰り返し呼べるが、この phase では反復制御、sink owner、allocation recovery、winding / fill rule をまだ定義しない。

### SFNT simple glyph path sink action apply state

F4ae は F4ac/F4ad が運ぶ `GuiSfntSimpleGlyphPathSinkAction` を 1 個だけ消費し、domain status と diagnostic count を返す pure boundary である。これは real sink、contour-wide loop、outline builder、renderer、rasterizer、platform API ではない。

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
gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    action GuiSfntSimpleGlyphPathSinkAction
    -> GuiSfntSimpleGlyphPathSinkActionApplyStep
```

helper は action を `match` し、`EmitEvent` は `EmittedEvent event` と `emitted_event_count + 1`、`Reject` は `Rejected reason` と `reject_count + 1`、`CloseContour` は `ClosedContour close` と `close_contour_count + 1`、`NoAction` は `NoAction` と `no_action_count + 1` を返す。増える count は常に 1 種類だけである。

`Reject` は malformed font を表す parse error ではなく、policy が返した typed domain status なので `Result::Err` に変換しない。`NoAction` は silent no-op ではなく、「明示的に `NoAction` を消費した」status として保持する。

F4ae の count は test / diagnostic / contract 検査用の集計であり、cursor、next state、traversal authority ではない。走査位置と次 item の authority は F4ac/F4ad の consumer item next に残す。F4ae は `Vec` / `push`、loop、current point state、outline allocation、lower lookup、metadata parser、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer apply step

F4af は F4ac の consumer item を 1 action 分だけ F4ae の apply state に適用し、apply result と保存済み checked continuation を同じ typed value に束ねる段階である。これは advance helper、byte-backed lookup、contour-wide loop、real sink、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionApplyStep
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    -> GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
```

helper は `item.action` と `item.next` をそれぞれ 1 回だけ読む。`item.action` は `gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action` へ 1 回だけ渡し、返った `apply_step` と `item.next` の copy を `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` に束ねる。

F4af は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` も呼ばない。次 consumer item を byte-backed に解決する authority は F4ad に残す。F4af の `next` は F4ac packet に保存されている `GuiSfntSimpleGlyphPathSinkActionItemNext` であり、新しい traversal decision ではない。

F4af は action payload を直接 `match` しない。`Reject`、`CloseContour`、`NoAction` の解釈は F4ae apply helper に委譲する。`Result`、`Option`、`Vec` / `push`、loop、current point state、outline allocation、lower lookup、metadata parser、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer apply terminal

F4ag は F4af の `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` を future consumer loop が扱いやすい terminal 判定へ分類する段階である。これは loop、next lookup、sink mutation、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
```

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step:
    step &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    -> GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal
```

判定順は固定である。まず `step.apply_step.status` が `Rejected reason` なら `Rejected reason` を返す。これは malformed SFNT bytes ではなく sink policy の domain terminal なので `Result::Err` に変換しない。reject でなければ、F4af が保存している `step.next` を読む。`Continue item` なら `Continue step`、`EndContour` なら `EndContour step` を返す。

`NoAction` は silent skip ではないが、それだけで terminal にはしない。`NoAction + Continue` は `Continue`、`NoAction + EndContour` は `EndContour` である。`ClosedContour` status も同様に保存済み `next` に従う。これにより、action payload の意味と traversal authority を混ぜない。

F4ag は `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼ばず、`GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` も作らない。保存済み `GuiSfntSimpleGlyphPathSinkActionItemNext` だけを読む。`Vec` / `push`、loop、current point state、outline allocation、lower lookup、metadata parser、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer apply advance

F4ah は F4ag の terminal 判定を使い、apply 後の consumer stream を 1 step だけ進める byte-backed boundary である。これは contour-wide loop、iterator、real sink mutation、outline builder、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance:
    bytes &ByteBuf
    face_index Option i32
    step &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntParseError
```

helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step` を 1 回だけ呼ぶ。`Rejected reason` は `Result::Ok Rejected reason`、`EndContour` は `Result::Ok EndContour` として返す。どちらも malformed SFNT bytes ではないので `Result::Err` にはしない。

`Continue continue_step` の場合だけ、`continue_step` に保存された `GuiSfntSimpleGlyphPathSinkActionItemNext` を読む。`Continue next_item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy` を 1 回だけ呼び、成功時は `Continue next_consumer_item` として返す。`EndContour` が出た場合は `EndContour` を成功 terminal として返す。

F4ah は original `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を要求しない。F4af が保存した checked next を authority として使う。したがって `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` は呼ばない。これは F4ad の direct wrapper ではなく、F4ag terminal と保存済み `ActionItemNext` から F4ac lookup へ接続する 1 step boundary である。

F4ah は action payload を直接 `match` せず、apply をやり直さない。`Vec` / `push`、loop、current point state、outline allocation、lower lookup、metadata parser、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer consume once

F4ai は 1 consumer item を「apply してから advance する」境界である。ただし、単に `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` だけを返してはいけない。F4af の apply step には更新後の apply state と status が含まれ、future loop や diagnostics がこれを読むためである。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once:
    bytes &ByteBuf
    face_index Option i32
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError
```

helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item` を 1 回だけ呼び、得られた `apply_step` を `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index &apply_step policy` へ 1 回だけ渡す。advance が `Result::Err` なら malformed SFNT parse/range failure としてそのまま返す。advance が `Result::Ok advance` なら、同じ `apply_step` と `advance` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` に束ねて返す。

F4ai は F4ag を直接呼ばない。terminal classification は F4ah の責務である。F4ai は `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`、F4ad/F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables` を直接呼ばない。action payload を直接 `match` せず、`Vec` / `push`、loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action start consumer item

F4aj は contour start から future sink consumer の初期 packet を作る byte-backed boundary である。これは F4aa の start item と F4ac の consumer item を合成するだけで、consume、apply、post-apply advance、contour-wide loop、real sink mutation は行わない。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item:
    bytes &ByteBuf
    face_index Option i32
    glyph GuiGlyphId
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError
```

helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。`Result::Err error` なら parse/range/table error としてそのまま返す。`Result::Ok item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &item policy` を 1 回だけ呼び、その結果をそのまま返す。

ここで「advance しない」とは、F4ad の consumer item next、F4af apply、F4ah post-apply advance、F4ai consume once を呼ばないという意味である。F4ac は consumer item を作る契約上、checked `GuiSfntSimpleGlyphPathSinkActionItemNext` を内部で解決する。それは F4ac の責務であり、F4aj が新しい traversal authority を持つことではない。

F4aj は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once`、F4af/F4ah/F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables` を直接呼ばない。action payload を直接 `match` せず、`Vec` / `push`、loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action start consume once

F4ak は contour start から first consumer item を作り、その 1 item だけを consume する byte-backed boundary である。これは F4aj と F4ai の薄い合成であり、contour-wide loop、iterator、real sink mutation、full outline builder、renderer、rasterizer ではない。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once:
    bytes &ByteBuf
    face_index Option i32
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    glyph GuiGlyphId
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError
```

helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。`Result::Err error` なら parse/range/table error としてそのまま返す。`Result::Ok consumer_item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &consumer_item policy` を 1 回だけ呼び、その結果をそのまま返す。

F4ak の戻り値は F4ai と同じ `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` である。これにより、最初の action を consume した後の apply state / status と post-consume advance を両方保持する。`GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` だけへ縮約してはいけない。

F4ak は F4aa/F4ac/F4ad/F4af/F4ah/F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables` を直接呼ばない。`GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、action payload を直接 `match` せず、`Vec` / `push`、loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer consume step apply summary

F4al は `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` から、consume 後の apply state と消費済み action status を安定した public helper で読む段階である。これは future loop が内部の `consume_step -> consumer_apply_step -> inner_apply_step` layout へ直接依存しないようにするための pure projection であり、loop、iterator、real sink、byte lookup、renderer、rasterizer ではない。

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state:
    step &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionApplyState

gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status:
    step &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionApplyStatus
```

両 helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step` を 1 回だけ呼び、得られた `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` から `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step &consumer_apply_step` を 1 回だけ呼ぶ。state helper は `gui_sfnt_simple_glyph_path_sink_action_apply_step_state &inner_apply_step` を 1 回だけ呼び、status helper は `gui_sfnt_simple_glyph_path_sink_action_apply_step_status &inner_apply_step` を 1 回だけ呼ぶ。

F4al は consume step の `advance` を読まない。`Result` / `Option`、byte-backed lookup、consumer item next、consume-once、start helper、action payload direct match、`Vec` / `push`、loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer consume summary

F4am は `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` を、future loop が直接扱うための flat summary value へ変換する段階である。F4al が apply side だけを読む helper であるのに対し、F4am は F4al の state / status と既存 `advance` accessor を 1 value に束ねる。これは loop、iterator、real sink、byte lookup、renderer、rasterizer ではなく、すでに計算済みの state / status / advance を読むだけの pure projection である。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step:
    step &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
```

`summary_from_step` は次の 3 helper をそれぞれ 1 回だけ呼ぶ。

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance step
```

F4am は `advance` enum を新しく解釈しない。`Continue`、`Rejected`、`EndContour` の意味は F4ah の `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` contract に従う。F4am は `Result` / `Option`、byte-backed lookup、consumer item next、consume-once、start helper、action payload direct match、`Vec` / `push`、loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。

### SFNT simple glyph path sink action consumer consume summary terminal

F4an は `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` に保持された `advance` を、future loop が読む traversal control state へ写す段階である。名前は `Terminal` だが `Continue` も含むため、contour 終端だけを表す型ではない。F4am が summary value を作るだけで `advance` を解釈しないのに対し、F4an は stored advance を 1 回だけ読み、loop 本体から `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` の storage detail を隠す。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal:
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    -> GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal
```

`summary_terminal` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance summary` を 1 回だけ呼び、次の同型写像だけを行う。

```text
ApplyAdvance Continue item -> SummaryTerminal Continue item
ApplyAdvance Rejected reason -> SummaryTerminal Rejected reason
ApplyAdvance EndContour -> SummaryTerminal EndContour
```

F4an は `Result` / `Option`、byte-backed lookup、consumer item next、consume-once、start helper、metadata parser、lower glyf lookup、`*_with_tables`、action payload direct match、`Vec` / `push`、loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。match 対象は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` だけであり、action payload enum を覗かない。

### SFNT simple glyph path sink action start consume summary

F4ap は F4ak の start consume-once と F4am の consume summary projection を薄く合成し、future loop が最初に読む initial summary を返す段階である。F4ao は summary から次 summary へ進めるが、最初の summary を作る責務は持たない。F4ap がその初期境界を提供する。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary:
    bytes &ByteBuf
    face_index Option i32
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    glyph GuiGlyphId
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntParseError
```

helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy` を 1 回だけ呼ぶ。`Result::Err error` はそのまま伝播する。`Result::Ok consume_step` の場合だけ `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を 1 回だけ呼び、`Result::Ok summary` を返す。

F4ap が直接呼んでよい byte-backed lookup は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once` だけである。start item、start consumer item、consumer item consume-once、summary advance-once、consumer item next lookup、lower glyf lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec` / `push`、full loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback は直接使わない。

### SFNT simple glyph path sink action consumer consume summary advance once

F4ao は F4am/F4an で作った summary boundary を、byte-backed future loop の 1 step advance boundary へ接続する段階である。これは full loop、iterator owner、real sink mutation、outline allocation、renderer、rasterizer ではない。`Continue` のときだけ次 consumer item を 1 つ消費し、次の summary を返す。`Rejected` と `EndContour` は parse error ではなく、`Result::Ok` の domain terminal として返す。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once:
    bytes &ByteBuf
    face_index Option i32
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance GuiSfntParseError
```

helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary` と `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` をそれぞれ 1 回だけ読む。`Continue item` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &item policy` を 1 回呼び、成功した `consume_step` を `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` で次 summary へ変換する。

F4ao が直接呼んでよい byte-backed lookup は Continue branch の `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once` だけである。start helper、consumer item next lookup、lower glyf lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec` / `push`、full loop、current point state、outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback は直接使わない。

### SFNT simple glyph path sink action consumer consume summary drain budget

F4aq は F4ap の initial summary と F4ao の advance-once をつなぎ、contour action consumer を explicit budget 内で domain terminal まで進める boundary である。これは full outline builder ではなく、`Vec` command list、real sink mutation、renderer、rasterizer、platform API を持たない。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain:
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    StepBudgetExhausted GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget:
    bytes &ByteBuf
    face_index Option i32
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain GuiSfntParseError
```

helper は最初に `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を 1 回だけ読む。`Rejected reason` と `EndContour` は parse error ではなく、budget を消費せず `Result::Ok` の domain result として current summary と一緒に返す。`Continue` かつ `remaining_steps <= 0` の場合は `StepBudgetExhausted current_summary` を返す。これは hidden fallback ではなく、呼び出し側が次の work slice を要求するための typed terminal である。

`Continue` かつ `remaining_steps > 0` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once` を 1 回呼ぶ。`Result::Err error` はそのまま伝播する。`Continue next_summary` の場合は `remaining_steps - 1` で同じ drain helper へ再帰する。F4ao が保守上 `Rejected` または `EndContour` を返した場合は、F4ao に渡した current summary を drain result に入れる。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary_drain_budget:
    bytes &ByteBuf
    face_index Option i32
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    glyph GuiGlyphId
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain GuiSfntParseError
```

start helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary` を 1 回だけ呼び、成功時だけ drain budget helper へ 1 回渡す。start helper 自体は F4ao、consumer item next lookup、lower glyf lookup、metadata parser、`*_with_tables` を直接呼ばない。

F4aq は action payload direct match、`Vec` / `push`、full outline allocation、renderer、rasterizer、platform API、host text measurement、font fallback を直接使わない。`remaining_steps == 0` と `remaining_steps < 0` はどちらも `StepBudgetExhausted` であり、暗黙に描画や終端成功へ置換しない。

### SFNT simple glyph outline storage capacity

F5a は F4aq の bounded drain result から後続 outline storage の必要量を計算する境界である。ここでは contour / point / edge / path command の個数だけを扱い、point list、contour list、path command list、mask、renderer command、raster output、platform resource は作らない。

F5a の入力は `GuiSfntSimpleGlyphTopology` と caller が選んだ `GuiSfntSimpleGlyphOutlineStorageLimit` である。`GuiSfntSimpleGlyphTopology` の単位は glyph 内の logical contour / logical point count であり、pixel、font unit scale、layout px、device px ではない。F5a の出力は allocation-free な value enum であり、成功・容量不足・topology 不正・command count overflow を文字列や panic に変換しない。

```text
GuiSfntSimpleGlyphOutlineStorageCapacity:
    glyph GuiGlyphId
    contour_count i32
    point_count i32
    edge_count i32
    path_command_pair_count i32
    path_command_count i32
```

`edge_count` は simple glyph contour を閉じた contour edge stream として読むため `point_count` と同じである。`path_command_pair_count` は各 edge が move / draw の pair を持つので `point_count` と同じである。`path_command_count` は `point_count * 2` である。`point_count > 1073741823` は i32 command count に入らないので `CommandCountOverflow` とする。

```text
GuiSfntSimpleGlyphOutlineStorageLimit:
    max_contours i32
    max_points i32
    max_edges i32
    max_path_commands i32
```

limit の各値は 1 以上を許可容量として扱う。0 以下は unlimited ではなく capacity exceeded である。これは caller の設定ミスを silent no-op にしないためである。

```text
GuiSfntSimpleGlyphOutlineCapacityRejectReason:
    ContourCapacityExceeded
    PointCapacityExceeded
    EdgeCapacityExceeded
    CommandCapacityExceeded
```

```text
GuiSfntSimpleGlyphOutlineCapacityRejected:
    reason GuiSfntSimpleGlyphOutlineCapacityRejectReason
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    limit GuiSfntSimpleGlyphOutlineStorageLimit
```

```text
GuiSfntSimpleGlyphOutlineCapacityCheck:
    Fits GuiSfntSimpleGlyphOutlineStorageCapacity
    InvalidTopology GuiSfntSimpleGlyphTopology
    CommandCountOverflow GuiSfntSimpleGlyphTopology
    Rejected GuiSfntSimpleGlyphOutlineCapacityRejected
```

`InvalidTopology` は `contour_count <= 0`、`point_count <= 0`、`contour_count > point_count` を表す。これは byte parse error ではなく、capacity planning が受け取った topology value の domain 不正である。byte-backed topology lookup から来る通常経路では既に検査済みであるが、test / virtual event / future headless harness が value を直接作るため、この境界でも enum として明示する。

`GuiSfntSimpleGlyphOutlineCapacityRejectReason` は capacity と limit を比較した後の limit exceed だけを表す。`InvalidTopology` と `CommandCountOverflow` は trusted capacity が作れない状態なので、`GuiSfntSimpleGlyphOutlineCapacityRejected` には入れず、`GuiSfntSimpleGlyphOutlineCapacityCheck` の独立 variant として返す。limit check は contour、point、edge、path command の順で最初の exceeded reason を返す。

F4aq の `StepBudgetExhausted` は capacity success ではない。caller は同じ summary owner を保持したまま次の time slice を要求し、`EndContour` まで進めた後に capacity planning を呼ぶ。`Rejected` は policy/domain terminal なので、outline storage へ進まない。

owner recovery contract:

- F5a は owner を受け取らない pure value layer であり、入力 topology と limit を破棄しない。
- 後続の owner-taking outline storage API は success / capacity exceeded / unsupported / invalid topology を enum で返し、失敗時には input owner と capacity check を返す。
- 失敗 branch で point buffer、contour buffer、sink owner、font face owner を黙って捨ててはいけない。
- 未対応 outline format、host font substitute、tofu glyph substitute、silent success への置換は禁止する。未対応 feature は typed unsupported として返す。

### SFNT simple glyph outline storage owner

F5b は F5a の trusted capacity から、後続 outline builder が使う empty scalar slot storage を確保する最初の owner boundary である。F5b は contour point を復元せず、Bezier command を発行せず、mask / bitmap / render2d command / platform resource を作らない。

F5b の storage は 1 つの `Vec i32` owner だけを持つ。複数 Vec owner の部分確保失敗をこの段階に持ち込まず、失敗時の owner recovery surface を単純に保つためである。

```text
GuiSfntSimpleGlyphOutlineStorage:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    scalar_slots Vec i32
    scalar_slot_count i32
```

`scalar_slot_count` は次の fixed formula である。

```text
scalar_slot_count =
    contour_count
    + point_count
    + point_count
    + edge_count
    + path_command_count
```

内訳は contour endpoint slots、x coordinate slots、y coordinate slots、edge slots、path command tag slots である。F5b の時点では slot の意味だけを予約し、slot へ値を push しない。`Vec` の `len` は 0、`cap` は `scalar_slot_count` でなければならない。

F5b は public constructor で作られた forged capacity も検出する。invalid capacity は capacity exceeded より先に返す。

```text
GuiSfntSimpleGlyphOutlineStorageAllocErrorKind:
    InvalidCapacity
    CapacityRejected
    ScalarSlotCountOverflow
    ScalarSlotStorageAllocFailed
```

```text
GuiSfntSimpleGlyphOutlineStorageAllocError:
    kind GuiSfntSimpleGlyphOutlineStorageAllocErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    limit GuiSfntSimpleGlyphOutlineStorageLimit
    capacity_check Option GuiSfntSimpleGlyphOutlineCapacityCheck
```

`capacity_check = None` は `InvalidCapacity` のみである。`CapacityRejected`、`ScalarSlotCountOverflow`、`ScalarSlotStorageAllocFailed` は `Some checked` を持つ。これにより、capacity shape が不正な場合に misleading な `CapacityRejected` payload を作らない。

F5b の allocation order:

```text
alloc capacity limit:
    if not shape_is_valid capacity:
        Err InvalidCapacity capacity limit None
    else:
        checked = check_limit capacity limit
        match checked:
            Rejected:
                Err CapacityRejected capacity limit Some checked
            Fits:
                match scalar_slot_count_check capacity:
                    Overflow:
                        Err ScalarSlotCountOverflow capacity limit Some checked
                    Fits scalar_slot_count:
                        vec::with_capacity scalar_slot_count
                            Ok slots:
                                Ok storage capacity slots scalar_slot_count
                            Err:
                                Err ScalarSlotStorageAllocFailed capacity limit Some checked
            other:
                Err InvalidCapacity capacity limit Some checked
```

shape validation checks `point_count <= 1073741823` before comparing `path_command_count == point_count * 2`。`scalar_slot_count` は i32 上限 `2147483647` から staged residual guard で contour、x、y、edge、path command の順に差し引いて検査する。overflow する場合は allocation を試みない。

### SFNT simple glyph outline scalar slot mutation

F5c は F5b の storage owner に scalar value を 1 件追加する mutation boundary である。F5c は storage の slot region をまだ解釈しない。つまり、contour endpoint slot、x slot、y slot、edge slot、path command tag slot のどれへ入る値かは後続 builder phase が決める。

F5c の責務は次だけである。

```text
GuiSfntSimpleGlyphOutlineStorage + i32
    -> Result GuiSfntSimpleGlyphOutlineStorage GuiSfntSimpleGlyphOutlineStoragePushError
```

push failure は storage owner と rejected scalar value を同時に返す。

```text
GuiSfntSimpleGlyphOutlineStoragePushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    scalar_value i32
    error StdErrorKind
```

`scalar_value` は Copy な i32 であるが、error payload に明示的に残す。caller は cleanup、retry、diagnostic 変換のどれを選ぶ場合でも、storage owner と rejected scalar の両方を同じ `match` branch で扱える。

F5c の push helper は `Vec` の failure surface を隠さず次の順序を守る。

```text
push_scalar_slot storage value:
    capacity = storage.capacity
    scalar_slot_count = storage.scalar_slot_count
    scalar_slots = storage.scalar_slots
    match vec::push scalar_slots value:
        Ok next_slots:
            Ok Storage capacity next_slots scalar_slot_count
        Err e:
            error = vec_push_error_kind e
            returned_slots = vec_push_error_vec e
            returned_storage = Storage capacity returned_slots scalar_slot_count
            Err PushError returned_storage value error
```

`vec_push_error_kind` は `vec_push_error_vec` が error owner を消費する前に読む。F5c は `vec::push` を 1 回だけ呼び、`vec::with_capacity`、`vec::free`、`vec::filled`、`vec::replace`、`vec::pop` は呼ばない。storage cleanup は F5b の `gui_sfnt_simple_glyph_outline_storage_free` を caller が明示的に呼ぶ。

### SFNT simple glyph outline scalar region cursor

F5d は F5b/F5c の 1 本の scalar slot storage に typed region cursor を重ねる。storage 自体は `Vec i32` owner のまま保ち、region は次の固定範囲だけを表す。

```text
GuiSfntSimpleGlyphOutlineScalarRegion:
    ContourEndpoint
    PointX
    PointY
    Edge
    PathCommandTag

GuiSfntSimpleGlyphOutlineScalarRegionCursor:
    region GuiSfntSimpleGlyphOutlineScalarRegion
    start i32
    end i32
    next_index i32
```

region boundary は、F5b の trusted capacity から次の順序で決まる。

```text
ContourEndpoint  0 .. contour_count
PointX           contour_count .. contour_count + point_count
PointY           contour_count + point_count .. contour_count + point_count + point_count
Edge             contour_count + point_count + point_count .. contour_count + point_count + point_count + edge_count
PathCommandTag   contour_count + point_count + point_count + edge_count .. scalar_slot_count
```

unchecked boundary constructor は public API にしない。公開 API は `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity` であり、capacity shape と scalar slot count overflow を検査してから cursor を返す。検査に失敗した場合は `Result::Err StdErrorKind::InvalidOperation` または `Result::Err StdErrorKind::CapacityExceeded` を返し、境界加算へ進まない。

region push は storage owner、cursor、scalar value を受け取り、成功時は storage owner と進んだ cursor を返す。

```text
GuiSfntSimpleGlyphOutlineRegionPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphOutlineRegionPushErrorKind:
    StorageCapacityInvalid
    CursorInvalid
    CursorRegionMismatch
    StorageCursorMismatch
    RegionFull
    StoragePushFailed

GuiSfntSimpleGlyphOutlineRegionPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    scalar_value i32
    kind GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphOutlineRegionPush` と `GuiSfntSimpleGlyphOutlineRegionPushError` は storage owner を持つため `Clone` / `Copy` にしない。

`gui_sfnt_simple_glyph_outline_storage_push_region_scalar` は次の順序を守る。

```text
1. capacity、scalar_slot_count、scalar_slots_len、scalar_slots_cap を storage から読む
2. capacity shape を検査する
3. scalar_slot_count_check が Fits であることを検査する
4. storage.scalar_slot_count == expected_scalar_slot_count を検査する
5. scalar_slots_cap == scalar_slot_count を検査する
6. cursor が start <= next_index <= end を満たすか検査する
7. cursor の region/start/end が checked capacity 由来の region boundary と一致するか検査する
8. scalar_slots_len == cursor.next_index を検査する
9. cursor.next_index < cursor.end を検査する
10. F5c の gui_sfnt_simple_glyph_outline_storage_push_scalar_slot を 1 回だけ呼ぶ
```

`scalar_slots_len == cursor.next_index` は `RegionFull` より先に検査する。これにより、empty storage に full cursor を渡すような forged input は `RegionFull` ではなく `StorageCursorMismatch` として扱う。`scalar_slots_cap == scalar_slot_count` も F5c push より先に検査し、fixed-capacity region boundary の外で Vec growth が起きる実装にはしない。

F5d は contour endpoint や x/y coordinate の意味をまだ解釈しない。byte-backed lookup、point decode、path command generation、rasterizer、renderer、platform API、host text API へは進まない。

### SFNT simple glyph contour endpoint population

F5e は F5d の contour endpoint region cursor を使い、simple glyph の contour endpoint slot を owner-preserving に追加する。ここでは byte-backed `glyf` endpoint array をまだ読まない。caller が typed endpoint value を渡し、F5e は capacity、cursor、endpoint sequence の contract を検査する。

```text
GuiSfntSimpleGlyphContourEndpointSlot:
    contour_index i32
    end_point_index i32
```

success payload は storage owner、advanced cursor、次の endpoint validation に使う previous endpoint を返す。

```text
GuiSfntSimpleGlyphContourEndpointPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    previous_endpoint i32
```

error payload も storage owner を返す。

```text
GuiSfntSimpleGlyphContourEndpointPushErrorKind:
    StorageCapacityInvalid
    CursorInvalid
    CursorRegionMismatch
    ContourIndexMismatch
    PreviousEndpointMismatch
    EndpointOutOfRange
    EndpointNotIncreasing
    FinalEndpointMismatch
    RegionPushFailed

GuiSfntSimpleGlyphContourEndpointPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    endpoint GuiSfntSimpleGlyphContourEndpointSlot
    previous_endpoint Option i32
    kind GuiSfntSimpleGlyphContourEndpointPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphContourEndpointPush` と `GuiSfntSimpleGlyphContourEndpointPushError` は storage owner を持つため `Clone` / `Copy` にしない。

`gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint` は次の順序を守る。

```text
1. storage から capacity を読む
2. capacity shape を検査する
3. scalar_slot_count_check が Fits であることを検査する
4. ここで初めて contour_count と point_count を読む
5. cursor が well-formed であることを検査する
6. cursor region が ContourEndpoint であることを検査する
7. endpoint.contour_index == cursor.next_index を検査する
8. 0 <= endpoint.contour_index < contour_count を検査する
9. 0 <= endpoint.end_point_index < point_count を検査する
10. previous_endpoint が None なら contour_index == 0 を検査する
11. previous_endpoint が Some なら contour_index > 0 を検査する
12. previous endpoint 自体が 0 <= previous < point_count - 1 を満たすか検査する
13. end_point_index > previous を検査する
14. final contour なら end_point_index == point_count - 1 を検査する
15. non-final contour なら end_point_index < point_count - 1 を検査する
16. F5d の gui_sfnt_simple_glyph_outline_storage_push_region_scalar を 1 回だけ呼ぶ
```

capacity validation は `contour_count` / `point_count` の使用や `+ 1` / `- 1` arithmetic より先に行う。cursor well-formed validation は `cursor.next_index` を contour semantic に使うより先に行う。previous endpoint range は `end_point_index > previous` より先に検査する。

F5e は endpoint array の意味だけを扱い、x/y coordinate、flag decode、edge generation、path command generation、rasterizer、renderer、platform API、host text API へは進まない。

### SFNT simple glyph contour endpoint byte reader bridge

F5f は既存の checked `glyf` endpoint array reader と F5e の contour endpoint storage boundary を接続する。ここで初めて byte-backed `endPtsOfContours` を読むが、x/y coordinate、flag decode、edge/path command generation、rasterizer、renderer、platform API、host text API へは進まない。

F5f の中心 contract は、byte lookup と storage mutation を同じ失敗状態に潰さないことである。

```text
GuiSfntSimpleGlyphContourEndpointReadPushErrorKind:
    ReadFailed
    PushFailed
```

success payload は storage owner、advanced cursor、次の endpoint validation に使う previous endpoint を返す。

```text
GuiSfntSimpleGlyphContourEndpointReadPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    previous_endpoint i32
```

error payload も storage owner を返す。`ReadFailed` では parse error を保持し、endpoint は `None` である。`PushFailed` では読めた endpoint value と F5e/F5d/F5c の lower error metadata を保持し、parse error は `None` である。

```text
GuiSfntSimpleGlyphContourEndpointReadPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    contour_index i32
    previous_endpoint Option i32
    kind GuiSfntSimpleGlyphContourEndpointReadPushErrorKind
    parse_error Option GuiSfntParseError
    endpoint Option GuiSfntSimpleGlyphContourEndpointSlot
    push_error_kind Option GuiSfntSimpleGlyphContourEndpointPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphContourEndpointReadPush` と `GuiSfntSimpleGlyphContourEndpointReadPushError` は storage owner を持つため `Clone` / `Copy` にしない。

`gui_sfnt_glyf_read_push_contour_endpoint` は次の順序を守る。

```text
1. gui_sfnt_glyf_read_contour_endpoint を 1 回だけ呼ぶ
2. read failure なら F5e push を呼ばず ReadFailed を返す
3. read success なら GuiSfntSimpleGlyphContourEndpointSlot を作る
4. gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint を 1 回だけ呼ぶ
5. push success なら storage / cursor / previous_endpoint を返す
6. push failure なら endpoint、F5e error kind、F5d error kind、F5c storage push error kind を読む
7. lower error data を読んだ後で storage owner を回収する
8. PushFailed を返す
```

read failure では storage mutation が起きていないため、元の storage owner をそのまま返す。push failure では F5e が owner recovery を担当するため、F5f は F5e error から回収した storage owner を返す。どちらも silent fallback や no-op 成功にしない。

### SFNT simple glyph point x coordinate population

F5g は F5d の `PointX` region cursor を使い、simple glyph の x coordinate scalar を owner-preserving に追加する。ここでは byte-backed flag stream や x delta decode をまだ読まない。caller が typed x coordinate value を渡し、F5g は capacity、cursor、logical point index の contract を検査する。

`PointX` region は contour endpoint region の後ろにあるため、cursor の `next_index` は scalar storage index であり、そのまま glyph logical point index ではない。

```text
logical_point_index = cursor.next_index - cursor.start
```

この変換は cursor が well-formed であり、cursor/capacity boundary が checked capacity と一致することを確認してからだけ行う。

```text
GuiSfntSimpleGlyphPointXSlot:
    point_index i32
    x i32
```

success payload は storage owner と advanced cursor を返す。

```text
GuiSfntSimpleGlyphPointXPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
```

error payload も storage owner を返す。

```text
GuiSfntSimpleGlyphPointXPushErrorKind:
    StorageCapacityInvalid
    CursorInvalid
    CursorRegionMismatch
    PointIndexMismatch
    PointIndexOutOfRange
    RegionPushFailed

GuiSfntSimpleGlyphPointXPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    point GuiSfntSimpleGlyphPointXSlot
    kind GuiSfntSimpleGlyphPointXPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphPointXPush` と `GuiSfntSimpleGlyphPointXPushError` は storage owner を持つため `Clone` / `Copy` にしない。`GuiSfntSimpleGlyphPointXSlot` と error kind は value-only なので `Clone` / `Copy` でよい。

`gui_sfnt_simple_glyph_outline_storage_push_point_x` は次の順序を守る。

```text
1. storage から capacity を読む
2. capacity shape を検査する
3. scalar_slot_count_check が Fits であることを検査する
4. ここで初めて point_count を読む
5. cursor が well-formed であることを検査する
6. cursor region が PointX であることを検査する
7. cursor boundary が checked capacity と一致することを検査する
8. logical_point_index = cursor.next_index - cursor.start を計算する
9. point.point_index == logical_point_index を検査する
10. 0 <= point.point_index < point_count を検査する
11. F5d の gui_sfnt_simple_glyph_outline_storage_push_region_scalar を 1 回だけ呼ぶ
```

F5g は x coordinate region の storage contract だけを扱い、byte decode、y coordinate、edge generation、path command generation、rasterizer、renderer、platform API、host text API へは進まない。

### SFNT simple glyph point x byte reader bridge

F5h は checked `GuiSfntSimpleGlyphPointStream` から 1 logical point の x coordinate だけを読み、F5g の `PointX` storage helper へ接続する。ここでは y coordinate、endpoint array、contour span、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

F5h は x-only boundary である。したがって forged stream の y range が壊れていても F5h は検査しない。y range validation は PointY / full point phase の責務である。同様に endpoint array failure は contour endpoint phase の責務であり、F5h の error domain へ混ぜない。

success payload は storage owner と advanced PointX cursor を返す。

```text
GuiSfntSimpleGlyphPointXReadPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
```

error payload も storage owner を返す。

```text
GuiSfntSimpleGlyphPointXReadPushErrorKind:
    ReadFailed
    PushFailed

GuiSfntSimpleGlyphPointXReadPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    point_index i32
    point Option GuiSfntSimpleGlyphPointXSlot
    kind GuiSfntSimpleGlyphPointXReadPushErrorKind
    parse_error Option GuiSfntParseError
    push_error_kind Option GuiSfntSimpleGlyphPointXPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphPointXReadPush` と `GuiSfntSimpleGlyphPointXReadPushError` は storage owner を持つため `Clone` / `Copy` にしない。error kind は value-only なので `Clone` / `Copy` でよい。

`gui_sfnt_glyf_read_push_point_x` は次の順序を守る。

```text
1. point_index が stream topology の point_count 内であることを検査する
2. flag stream と x delta stream だけを読み、target point までの累積 x を得る
3. read failure なら storage mutation を呼ばず ReadFailed を返す
4. read success なら GuiSfntSimpleGlyphPointXSlot を作る
5. F5g の gui_sfnt_simple_glyph_outline_storage_push_point_x を 1 回だけ呼ぶ
6. push failure なら point、F5g error kind、F5d error kind、F5c storage push error kind を読む
7. lower error data を読んだ後で storage owner を回収する
8. PushFailed を返す
```

F5h の x-only reader helper は bounded flag reads と `gui_sfnt_glyf_decode_x_delta` だけを使う。`gui_sfnt_glyf_decode_y_delta`、full point decode state、endpoint read、contour span read、path/raster/render/platform/host API は使わない。

### SFNT simple glyph point y coordinate population

F5i は F5d の `PointY` region cursor を使い、simple glyph の y coordinate scalar を owner-preserving に追加する。F5g と同じ storage contract を使うが、対象 region は `PointY` である。ここでは byte-backed flag stream や y delta decode をまだ読まない。

`PointY` region は `ContourEndpoint` と `PointX` region の後ろにある。2 contours / 4 points の場合、scalar region は次の順である。

```text
ContourEndpoint [0, 2)
PointX          [2, 6)
PointY          [6, 10)
```

したがって `PointY` に push する前に、endpoint とすべての `PointX` slot が既に追加済みでなければならない。`PointY` cursor の `next_index` は scalar storage index であり、そのまま glyph logical point index ではない。

```text
logical_point_index = cursor.next_index - cursor.start
```

この変換は cursor が well-formed であり、cursor/capacity boundary が checked capacity と一致することを確認してからだけ行う。

```text
GuiSfntSimpleGlyphPointYSlot:
    point_index i32
    y i32

GuiSfntSimpleGlyphPointYPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphPointYPushErrorKind:
    StorageCapacityInvalid
    CursorInvalid
    CursorRegionMismatch
    PointIndexMismatch
    PointIndexOutOfRange
    RegionPushFailed

GuiSfntSimpleGlyphPointYPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    point GuiSfntSimpleGlyphPointYSlot
    kind GuiSfntSimpleGlyphPointYPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphPointYPush` と `GuiSfntSimpleGlyphPointYPushError` は storage owner を持つため `Clone` / `Copy` にしない。`GuiSfntSimpleGlyphPointYSlot` と error kind は value-only なので `Clone` / `Copy` でよい。

`gui_sfnt_simple_glyph_outline_storage_push_point_y` は F5g と同じ順序を守る。ただし cursor region は `PointY` でなければならず、最後に F5d の `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、y scalar を追加する。

F5i は y coordinate region の storage contract だけを扱い、byte decode、x coordinate、edge generation、path command generation、rasterizer、renderer、platform API、host text API へは進まない。

### SFNT simple glyph point y byte reader bridge

F5j は checked `GuiSfntSimpleGlyphPointStream` から 1 logical point の y coordinate だけを読み、F5i の `PointY` storage helper へ接続する。ここでは x coordinate、endpoint array、contour span、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

F5j は y-only boundary である。したがって forged stream の x range が壊れていても F5j は検査しない。x range validation は PointX / full point phase の責務である。同様に endpoint array failure は contour endpoint phase の責務であり、F5j の error domain へ混ぜない。

success payload は storage owner と advanced PointY cursor を返す。

```text
GuiSfntSimpleGlyphPointYReadPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
```

error payload も storage owner を返す。

```text
GuiSfntSimpleGlyphPointYReadPushErrorKind:
    ReadFailed
    PushFailed

GuiSfntSimpleGlyphPointYReadPushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    point_index i32
    point Option GuiSfntSimpleGlyphPointYSlot
    kind GuiSfntSimpleGlyphPointYReadPushErrorKind
    parse_error Option GuiSfntParseError
    push_error_kind Option GuiSfntSimpleGlyphPointYPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
```

`GuiSfntSimpleGlyphPointYReadPush` と `GuiSfntSimpleGlyphPointYReadPushError` は storage owner を持つため `Clone` / `Copy` にしない。error kind は value-only なので `Clone` / `Copy` でよい。

`gui_sfnt_glyf_read_push_point_y` は次の順序を守る。

```text
1. point_index が stream topology の point_count 内であることを検査する
2. flag stream と y delta stream だけを読み、target point までの累積 y を得る
3. read failure なら storage mutation を呼ばず ReadFailed を返す
4. read success なら GuiSfntSimpleGlyphPointYSlot を作る
5. F5i の gui_sfnt_simple_glyph_outline_storage_push_point_y を 1 回だけ呼ぶ
6. push failure なら point、F5i error kind、F5d error kind、F5c storage push error kind を読む
7. lower error data を読んだ後で storage owner を回収する
8. PushFailed を返す
```

F5j の y-only reader helper は bounded flag reads と `gui_sfnt_glyf_decode_y_delta` だけを使う。`gui_sfnt_glyf_decode_x_delta`、full point decode state、endpoint read、contour span read、path/raster/render/platform/host API は使わない。

### SFNT simple glyph outline point coordinate read

F5k は F5 storage に既に追加済みの `PointX` / `PointY` scalar slot から、1 logical point の coordinate pair だけを読み出す read-only boundary である。ここでは byte stream decode、flag decode、endpoint array、contour span、on-curve 判定、end-of-contour 判定、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

F5k は `GuiSfntSimpleGlyphPoint` を返さない。`GuiSfntSimpleGlyphPoint` は `on_curve` と `end_of_contour` を含む full point value であり、F5 storage にはその情報がまだ保持されていないためである。F5k は次の value-only 型を返す。

```text
GuiSfntSimpleGlyphOutlinePointCoordinate:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
```

read failure は typed enum と value-only context で返す。

```text
GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind:
    StorageCapacityInvalid
    ScalarSlotCountMismatch
    ScalarStorageCapacityMismatch
    PointIndexOutOfRange
    CoordinateNotReady
    ScalarSlotMissing

GuiSfntSimpleGlyphOutlinePointCoordinateReadError:
    kind GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    point_index i32
    scalar_slot_count i32
    scalar_slots_len i32
    scalar_slots_cap i32
```

slot order は F5d/F5i と同じ固定 layout を使う。

```text
ContourEndpoint [0, contour_count)
PointX          [contour_count, contour_count + point_count)
PointY          [contour_count + point_count, contour_count + point_count + point_count)
```

`gui_sfnt_simple_glyph_outline_storage_read_point_coordinate` は次の順序を守る。

```text
1. storage capacity shape を検査する
2. expected scalar slot count を計算する
3. storage.scalar_slot_count == expected を検査する
4. scalar_slots_cap == storage.scalar_slot_count を検査する
5. point_index が 0 <= point_index < point_count であることを検査する
6. y slot index が scalar_slots_len 内にあることを検査する
7. private scalar slot getter で x slot と y slot を読む
8. Some x / Some y なら coordinate value を返す
9. readiness 検査後に None が返った場合は ScalarSlotMissing を返す
```

F5k の raw scalar slot getter は private helper であり、unchecked public accessor として公開しない。`vec::get` を使う場所はこの private helper に閉じ込める。`CoordinateNotReady` は storage がまだ required y slot まで埋まっていない状態を表し、fallback 値や zero coordinate を返してはいけない。

### SFNT simple glyph outline point endpoint marker read

F5l は F5 storage に追加済みの `ContourEndpoint` scalar region から、1 logical point が属する contour と、その point が contour end であるかだけを読む read-only boundary である。ここでは flag byte、x/y coordinate、full point、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

F5l は endpoint marker value を返す。

```text
GuiSfntSimpleGlyphOutlinePointEndpointMarker:
    glyph GuiGlyphId
    point_index i32
    contour_index i32
    end_of_contour bool
```

read failure は typed enum と value-only context で返す。

```text
GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind:
    StorageCapacityInvalid
    ScalarSlotCountMismatch
    ScalarStorageCapacityMismatch
    PointIndexOutOfRange
    EndpointNotReady
    EndpointSlotMissing
    EndpointTopologyInvalid

GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError:
    kind GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    point_index i32
    scalar_slot_count i32
    scalar_slots_len i32
    scalar_slots_cap i32
```

`gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker` は次の順序を守る。

```text
1. storage capacity shape を検査する
2. expected scalar slot count を計算する
3. storage.scalar_slot_count == expected を検査する
4. scalar_slots_cap == storage.scalar_slot_count を検査する
5. point_index が 0 <= point_index < point_count であることを検査する
6. scalar_slots_len >= contour_count であることを検査する
7. private scalar slot getter で全 endpoint slot を final contour まで読む
8. 各 endpoint が 0 <= endpoint < point_count かつ strictly increasing であることを検査する
9. point_index <= endpoint になる最初の contour を marker candidate として記録する
10. final endpoint が point_count - 1 であることを検査する
11. endpoint topology 全体が valid で、candidate がある場合だけ marker を返す
```

F5l は最初に `point_index <= endpoint` となった contour を見つけても即時成功しない。全 endpoint slot を最後まで検査し、final endpoint が `point_count - 1` であることを確認してから成功する。これにより forged storage の `[1, 2]` のような endpoint array が point 0 に対して partial success を返すことを防ぐ。

### SFNT simple glyph point flag marker read

F5m は checked `GuiSfntSimpleGlyphPointStream` の flag range だけから、1 logical point の raw flag と on-curve marker を読む read-only boundary である。F5 storage scalar layout には `PointFlag` region を追加しない。既存の `ContourEndpoint`、`PointX`、`PointY`、`Edge`、`PathCommandTag` の境界をこの phase で動かすと、F5b から F5l の slot contract を崩すためである。

F5m は flag marker value を返す。

```text
GuiSfntSimpleGlyphPointFlagMarker:
    glyph GuiGlyphId
    point_index i32
    raw_flag i32
    on_curve bool
```

`gui_sfnt_glyf_read_point_flag_from_stream` は `Result GuiSfntSimpleGlyphPointFlagMarker GuiSfntParseError` を返す。これは byte-backed stream read であり、storage owner recovery boundary ではないため、既存の `GuiSfntParseError` を使う。

```text
point_index out of range  -> MissingGlyphOutline
flag stream corruption    -> MalformedGlyfRecord
```

F5m は次の順序を守る。

```text
1. stream topology から point_count と glyph を読む
2. point_index が 0 <= point_index < point_count であることを検査する
3. flag_cursor = stream.flag_data_offset、logical_index = 0 から scan を始める
4. flag byte を stream.flag_data range 内で読む
5. repeat bit 8 がある場合は repeat count byte を stream.flag_data range 内で読む
6. run_count = repeat_count + 1、repeat bit がない場合は run_count = 1 とする
7. logical_index + run_count <= point_count を検査する
8. run 全体が valid である場合だけ、point_index が run 内かを判定する
9. target が run 内なら raw_flag と flag bit 0 の on_curve marker を返す
10. target が run 外なら logical_index と flag_cursor を進めて scan を続ける
```

repeat run overrun は success より前に拒否する。たとえば point_count 2 の stream で repeat flag が `repeat_count = 4` を持つ場合、point 0 は run 内に見えても `MalformedGlyfRecord` を返す。forged flag stream を partial success にしてはいけない。

F5m は次を呼ばない。

```text
x/y coordinate decode
full point decode state
endpoint readers
coordinate storage readers
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point read

F5n は F5k の coordinate、F5l の endpoint marker、F5m の flag marker を合成し、既存の `GuiSfntSimpleGlyphPoint` を read-only に作る boundary である。ここでは edge/path storage、outline stream、rasterizer、renderer、platform API、host text API へ進まない。

F5n は storage と stream を混ぜて読むため、component read より前に shared precondition を検査する。

```text
1. storage capacity と stream topology を読む
2. storage capacity shape を検査する
3. storage glyph と stream glyph が一致することを検査する
4. storage contour_count と stream contour_count が一致することを検査する
5. storage point_count と stream point_count が一致することを検査する
6. point_index が shared point_count の範囲内であることを検査する
7. F5k coordinate read を 1 回だけ呼ぶ
8. F5l endpoint marker read を 1 回だけ呼ぶ
9. F5m flag marker read を 1 回だけ呼ぶ
10. component glyph / point_index が shared request と一致することを fail-closed に検査する
11. GuiSfntSimpleGlyphPoint を作る
```

shared precondition の失敗は component error に潰さない。たとえば `point_index == point_count` は `CoordinateReadFailed` ではなく、F5n の `PointIndexOutOfRange` として返す。

```text
GuiSfntSimpleGlyphOutlinePointReadErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    PointIndexOutOfRange
    CoordinateReadFailed
    EndpointMarkerReadFailed
    FlagReadFailed
    ComponentGlyphMismatch
    ComponentPointIndexMismatch
```

```text
GuiSfntSimpleGlyphOutlinePointReadError:
    kind GuiSfntSimpleGlyphOutlinePointReadErrorKind
    point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    coordinate_error Option GuiSfntSimpleGlyphOutlinePointCoordinateReadError
    endpoint_error Option GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError
    flag_error Option GuiSfntParseError
```

F5n の成功値は次の既存型である。

```text
GuiSfntSimpleGlyphPoint:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
    on_curve bool
    end_of_contour bool
```

F5n は次を直接呼ばない。

```text
vec::
raw scalar slot getter
x/y coordinate byte decode
endpoint scalar scan loop
flag scan loop
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point read step

F5o は F5n の full point read boundary を、allocation なしの cursor step として反復できるようにする段階である。ここでは `Vec GuiSfntSimpleGlyphPoint` を作らず、edge/path storage、outline stream、rasterizer、renderer、platform API、host text API へ進まない。

cursor は次に読む logical point index だけを持つ。

```text
GuiSfntSimpleGlyphOutlinePointReadCursor:
    next_point_index i32
```

step は point を返す場合と、正常終端を返す場合を enum status で分ける。

```text
GuiSfntSimpleGlyphOutlinePointReadStepStatus:
    Point
    End

GuiSfntSimpleGlyphOutlinePointReadStep:
    status GuiSfntSimpleGlyphOutlinePointReadStepStatus
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    next_cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    point Option GuiSfntSimpleGlyphPoint
```

`status = Point` のとき `point` は `Some` である。`status = End` のとき `point` は `None` であり、`cursor` と `next_cursor` はどちらも `point_count` を指す。終端は失敗ではない。ただし、終端成功を返す前に storage / stream の shared precondition は必ず検査する。これにより、forged storage / stream mismatch を `End` として隠さない。

```text
GuiSfntSimpleGlyphOutlinePointReadStepErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange
    PointReadFailed

GuiSfntSimpleGlyphOutlinePointReadStepError:
    kind GuiSfntSimpleGlyphOutlinePointReadStepErrorKind
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    point_error Option GuiSfntSimpleGlyphOutlinePointReadError
```

`gui_sfnt_simple_glyph_outline_storage_read_point_step` は次の順序を守る。

```text
1. storage capacity と stream topology を読む
2. storage capacity shape を検査する
3. storage glyph と stream glyph が一致することを検査する
4. storage contour_count と stream contour_count が一致することを検査する
5. storage point_count と stream point_count が一致することを検査する
6. cursor.next_point_index を読む
7. next_point_index < 0 または next_point_index > point_count なら CursorOutOfRange
8. next_point_index == point_count なら End step を返す
9. next_point_index < point_count なら F5n を 1 回だけ呼ぶ
10. F5n の失敗は PointReadFailed として保持する
11. F5n の成功値を Some に入れ、next_cursor = next_point_index + 1 の Point step を返す
```

F5o は F5n だけに依存し、F5k / F5l / F5m やその下位 loop を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_storage_read_point_coordinate
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker_loop
gui_sfnt_glyf_read_point_flag_from_stream_loop
gui_sfnt_glyf_read_point_flag_run_or_continue
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point read drain budget

F5p は F5o の point step を、明示的な step budget 内で正常終端まで進める no-allocation drain boundary である。これは full point `Vec` や path/raster/render の実装ではない。後続の point collection、edge/path tag population、outline stream、raster mask、render2d command emission が再利用できる traversal contract を先に固定する。

drain summary は、drain が停止した cursor、今回の drain call で読んだ point 数、最後に読んだ point を保持する。

```text
GuiSfntSimpleGlyphOutlinePointReadDrainSummary:
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    points_read i32
    last_point Option GuiSfntSimpleGlyphPoint
```

drain の成功値は、正常終端と budget exhaustion を enum で分ける。

```text
GuiSfntSimpleGlyphOutlinePointReadDrain:
    End GuiSfntSimpleGlyphOutlinePointReadDrainSummary
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointReadDrainSummary
```

`StepBudgetExhausted` は失敗ではない。ただし silent success でもない。呼び出し側が次の work slice を要求できる typed terminal である。

F5p は F5o の error をそのまま public error にせず、drain 固有 error に包む。これは F5o の通常失敗と、F5p が検出した F5o 成功値の invariant violation を分けるためである。

```text
GuiSfntSimpleGlyphOutlinePointReadDrainErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange
    StepReadFailed
    StepInvariantInvalid

GuiSfntSimpleGlyphOutlinePointReadDrainError:
    kind GuiSfntSimpleGlyphOutlinePointReadDrainErrorKind
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    step_error Option GuiSfntSimpleGlyphOutlinePointReadStepError
    step Option GuiSfntSimpleGlyphOutlinePointReadStep
```

`gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget` は次の順序を守る。

```text
1. storage capacity と stream topology を読む
2. storage capacity shape を検査する
3. storage glyph と stream glyph が一致することを検査する
4. storage contour_count と stream contour_count が一致することを検査する
5. storage point_count と stream point_count が一致することを検査する
6. cursor.next_point_index を読む
7. next_point_index < 0 または next_point_index > point_count なら CursorOutOfRange
8. next_point_index == point_count なら End summary を返す
9. non-terminal かつ remaining_steps <= 0 なら StepBudgetExhausted summary を返す
10. non-terminal かつ remaining_steps > 0 の場合だけ F5o point step を 1 回呼ぶ
11. F5o Err は StepReadFailed として保持する
12. F5o Ok Point かつ point Some で、next cursor が現在 cursor から 1 点分だけ前進している場合だけ points_read を 1 増やして iteration を続ける
13. F5o Ok Point かつ point None、next cursor が `current + 1` ではない Point、または F5o Ok End は StepInvariantInvalid として返す
```

つまり、terminal check は budget check より前、budget check は F5o call より前である。budget が尽きている non-terminal cursor では point read work を進めない。一方で、terminal cursor は budget 0 でも `End` として返す。

F5p は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_simple_glyph_outline_storage_read_point_coordinate
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker_loop
gui_sfnt_glyf_read_point_flag_from_stream_loop
gui_sfnt_glyf_read_point_flag_run_or_continue
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item classification

F5q は F5p で読める full point を、後続 outline stream / contour / path phase が直接読むための no-allocation item boundary である。これは point collection、path command、raster mask、render command ではない。

point item kind は、on-curve/off-curve と contour endpoint を enum として表す。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemKind:
    OnCurve
    OffCurve
    EndOnCurve
    EndOffCurve

GuiSfntSimpleGlyphOutlinePointStreamItem:
    point GuiSfntSimpleGlyphPoint
    kind GuiSfntSimpleGlyphOutlinePointStreamItemKind
```

`EndOnCurve` と `EndOffCurve` は contour 終端を typed value として運ぶための variant である。endpoint を後段が `bool` field から毎回推測する設計にしない。

classification helper は `GuiSfntSimpleGlyphPoint` だけを読む。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point:
    GuiSfntSimpleGlyphPoint
    -> GuiSfntSimpleGlyphOutlinePointStreamItemKind

gui_sfnt_simple_glyph_outline_point_stream_item:
    GuiSfntSimpleGlyphPoint
    -> GuiSfntSimpleGlyphOutlinePointStreamItem
```

`gui_sfnt_simple_glyph_outline_point_stream_item` は外部から kind を受け取らない。kind は point payload から exactly once 導く。

classification order は固定する。

```text
1. on_curve を読む
2. end_of_contour を読む
3. end_of_contour が true なら EndOnCurve / EndOffCurve を返す
4. end_of_contour が false なら OnCurve / OffCurve を返す
```

F5q は次を直接呼ばない。

```text
ByteBuf
GuiSfntSimpleGlyphPointStream
GuiSfntSimpleGlyphOutlineStorage
gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_
gui_sfnt_lookup_
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item step

F5r は F5o の `GuiSfntSimpleGlyphOutlinePointReadStep` を、F5q の `GuiSfntSimpleGlyphOutlinePointStreamItem` を持つ step に変換する pure boundary である。これは byte-backed reader ではなく、full point `Vec`、sink mutation、path command、raster mask、render command でもない。

F5r の成功 status は、item を 1 つ読めた状態と終端を分ける。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus:
    Item
    End

GuiSfntSimpleGlyphOutlinePointStreamItemStep:
    status GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    next_cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

`status = Item` の場合、`item` は `Some` であり、`next_cursor.next_point_index == cursor.next_point_index + 1` である。`status = End` の場合、`item` は `None` であり、`next_cursor.next_point_index == cursor.next_point_index` である。

F5r の error は invariant failure だけを表す。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemStepErrorKind:
    PointStepInvariantInvalid

GuiSfntSimpleGlyphOutlinePointStreamItemStepError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemStepErrorKind
    step GuiSfntSimpleGlyphOutlinePointReadStep
```

変換 helper は次である。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step:
    GuiSfntSimpleGlyphOutlinePointReadStep
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemStep GuiSfntSimpleGlyphOutlinePointStreamItemStepError
```

変換順序は固定する。

```text
1. F5o step の status、cursor、next_cursor、point を読む
2. status = Point かつ point = Some point の場合、next cursor が cursor + 1 であることを検査する
3. 検査に通った Point だけで F5q constructor を exactly once 呼び、Item step を返す
4. status = End かつ point = None の場合、next cursor が cursor と同じであることを検査する
5. 検査に通った End だけを End step として返す
6. それ以外は PointStepInvariantInvalid を返す
```

F5r は F5q の `gui_sfnt_simple_glyph_outline_point_stream_item` constructor だけを呼ぶ。`gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point` を直接呼んではならない。kind の導出は F5q constructor の契約であり、F5r が再実装しない。

F5r は次を直接呼ばない。

```text
ByteBuf
GuiSfntSimpleGlyphPointStream
GuiSfntSimpleGlyphOutlineStorage
gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_
gui_sfnt_lookup_
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item drain

F5s は F5o の point step と F5r の item step conversion を、明示的な step budget 内で進める classified item drain boundary である。これは full point `Vec`、item list、path command、raster mask、render command ではない。後続の contour/path/sink phase が同じ cursor contract を再利用できるように、classified item の streaming traversal だけを固定する。

drain summary は、drain が停止した cursor、今回の drain call で読んだ item 数、最後に読んだ classified item を保持する。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary:
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    items_read i32
    last_item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

drain の成功値は、正常終端と budget exhaustion を enum で分ける。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemDrain:
    End GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary
```

`StepBudgetExhausted` は失敗ではない。non-terminal cursor で `remaining_steps <= 0` だったため、F5o/F5r を呼ばずに停止した typed terminal である。terminal cursor は budget 0 でも `End` になる。

F5s の error は、共有 cursor 検証、F5o read failure、F5r conversion failure、F5s 自身の defensive invariant failure を分ける。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange
    PointStepReadFailed
    ItemStepConvertFailed
    ItemStepInvariantInvalid

GuiSfntSimpleGlyphOutlinePointStreamItemDrainError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    point_step_error Option GuiSfntSimpleGlyphOutlinePointReadStepError
    item_step_error Option GuiSfntSimpleGlyphOutlinePointStreamItemStepError
    point_step Option GuiSfntSimpleGlyphOutlinePointReadStep
    item_step Option GuiSfntSimpleGlyphOutlinePointStreamItemStep
```

`ItemStepConvertFailed` は F5r が F5o step shape を不正として拒否した場合である。`ItemStepInvariantInvalid` は F5r の成功値を F5s が再検査した時の defensive failure であり、future internal change や forged constructor value に対する fail-closed branch として保持する。

`gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget` は次の順序を守る。

```text
1. shared cursor validation helper で storage capacity と stream topology と cursor を検査する
2. next_point_index == point_count なら End summary を返す
3. non-terminal かつ remaining_steps <= 0 なら StepBudgetExhausted summary を返す
4. non-terminal かつ remaining_steps > 0 の場合だけ F5o point step を 1 回呼ぶ
5. F5o Err は PointStepReadFailed として保持する
6. F5o Ok step は F5r item step conversion に 1 回だけ渡す
7. F5r Err は ItemStepConvertFailed として保持する
8. F5r Ok Item かつ item Some で、input cursor と next cursor が期待値に一致する場合だけ items_read を 1 増やして iteration を続ける
9. F5r Ok Item かつ item None、cursor 不一致、next cursor 不一致、または non-terminal で F5r Ok End は ItemStepInvariantInvalid として返す
```

つまり、terminal check は budget check より前、budget check は F5o call より前である。F5s は F5p public drain を呼ばない。F5p/F5s は shared cursor validation helper を共有するが、それぞれ自分の error kind へ変換する。

F5s は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_simple_glyph_outline_storage_read_point_coordinate
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker_loop
gui_sfnt_glyf_read_point_flag_from_stream_loop
gui_sfnt_glyf_read_point_flag_run_or_continue
gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection

F5t は F5s の classified item stream を後続 phase が owner として保持できるようにする allocator-backed collection boundary である。これは F5s drain-to-collection loop ではない。F5t は empty collection allocation、single item push、single item read だけを固定し、stream traversal、path command、raster mask、render command、platform API には進まない。

F5t は F5b の `GuiSfntSimpleGlyphOutlineStorageLimit` を使わない。scalar slot storage の contour / edge / path command limit と、classified item collection の item limit は意味が違うためである。F5t は専用 limit を持つ。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit:
    max_items i32
```

allocation は次の順序を守る。

```text
1. capacity shape を検査する
2. max_items > 0 を検査する
3. capacity.point_count <= max_items を検査する
4. vec::with_capacity point_count で item storage を確保する
```

allocation error は string ではなく enum と typed payload で返す。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind:
    InvalidCapacity
    InvalidLimit
    CapacityRejected
    ItemStorageAllocFailed
```

collection owner は次を保持する。owner なので `Clone` / `Copy` は実装しない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollection:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    items Vec GuiSfntSimpleGlyphOutlinePointStreamItem
    item_count i32
```

`gui_sfnt_simple_glyph_outline_point_stream_item_collection_free` は collection owner を消費し、内部 `items` に対して `vec::free` を 1 回だけ呼ぶ。free は stream traversal、path/raster/render、platform API を呼ばない。

push は次の順序を守る。

```text
1. capacity shape を検査する
2. items.len == item_count を検査する
3. items.cap == capacity.point_count を検査する
4. item_count < capacity.point_count を検査する
5. item.point.glyph == capacity.glyph を検査する
6. item.point.point_index == item_count を検査する
7. item.kind == kind_from_point item.point を検査する
8. vec::push を 1 回だけ呼ぶ
```

`ItemKindMismatch` は public constructor で forged item を作れることへの fail-closed branch である。item payload の kind を信頼せず、F5q の `kind_from_point` を authority とする。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind:
    InvalidCapacity
    CollectionLengthMismatch
    CollectionCapacityMismatch
    CollectionFull
    ItemGlyphMismatch
    ItemIndexMismatch
    ItemKindMismatch
    ItemStoragePushFailed
```

push error は collection owner、rejected item、error kind、lower `StdErrorKind` option を保持する。validation failure では lower error は `None`、`vec::push` failure では `Some StdErrorKind` である。`vec_push_error_kind` は `vec_push_error_vec` で owner を回収する前に読む。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushError:
    collection GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item GuiSfntSimpleGlyphOutlinePointStreamItem
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind
    storage_error Option StdErrorKind
```

read は `Option` を public surface にしない。`Option::None` だけでは forged collection invariant、範囲外、missing slot が区別できないためである。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind:
    InvalidCapacity
    CollectionLengthMismatch
    CollectionCapacityMismatch
    ItemIndexOutOfRange
    ItemStorageMissing
```

`gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item` は `Result GuiSfntSimpleGlyphOutlinePointStreamItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError` を返す。

F5t は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget
lower byte / point readers
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection drain

F5u は F5s の classified item stream を F5t の collection owner へ commit する境界である。これは path command、raster mask、render command、platform API ではない。F5s は `last_item` だけを summary に持ち、読んだ item 列そのものは返さないため、F5u は F5s に caller の `remaining_steps` を直接渡さない。

F5u が F5s へ渡す step budget は 0 または 1 だけである。

```text
remaining_steps <= 0
    F5s budget 0

remaining_steps > 0
    F5s budget 1
```

budget 0 は terminal / non-terminal の分類だけを F5s に委譲するために使う。terminal cursor なら `End`、non-terminal なら `StepBudgetExhausted` になる。budget 1 は最大 1 item だけを読み、読めた item を F5t push へ渡すために使う。

F5u の success summary は collection owner を含む。owner なので `Clone` / `Copy` は実装しない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary:
    collection GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    items_read i32
    last_item Option GuiSfntSimpleGlyphOutlinePointStreamItem

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain:
    End GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary
```

`items_read` は F5u 呼び出し内で collection へ commit できた item 数である。F5s の 1 step summary count ではない。push failure が起きた場合、読み取れたが commit できなかった item は `rejected_item` と `item_drain_result` に保持し、F5u の `cursor` と `items_read` は commit 済み位置のままにする。

F5u の error は F5s failure、F5s success invariant failure、F5t push failure を分ける。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind:
    CollectionCursorMismatch
    ItemDrainFailed
    ItemDrainInvariantInvalid
    CollectionPushFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainError:
    collection GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    items_read i32
    last_item Option GuiSfntSimpleGlyphOutlinePointStreamItem
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind
    item_drain_error Option GuiSfntSimpleGlyphOutlinePointStreamItemDrainError
    item_drain_result Option GuiSfntSimpleGlyphOutlinePointStreamItemDrain
    push_error_kind Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind
    push_storage_error Option StdErrorKind
    rejected_item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

`CollectionCursorMismatch` は collection の commit 済み item 数と cursor の `next_point_index` が一致しない場合である。この precondition がないと、terminal cursor と空 collection のような不整合を成功値として返せてしまう。`ItemDrainFailed` は lower F5s error を `item_drain_error` に保持する。`ItemDrainInvariantInvalid` は F5s が 0 / 1 item 以外を返した、budget 0 なのに item を返した、または `items_read == 1` なのに `last_item == None` だった場合であり、lower F5s success value を `item_drain_result` に保持する。`CollectionPushFailed` は F5t push failure を `push_error_kind`、`push_storage_error`、`rejected_item` に保持し、push error owner は collection owner として回収する。

push failure branch では次の順序を守る。

```text
1. push_error_kind を &push_error から読む
2. push_storage_error を &push_error から読む
3. rejected_item を &push_error から読む
4. push_error を消費して collection owner を回収する
```

`gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget` は次の順序を守る。

```text
1. collection.item_count == cursor.next_point_index を検査する
2. remaining_steps から step_budget 0 / 1 を作る
3. F5s drain を step_budget で 1 回呼ぶ
4. F5s Err は ItemDrainFailed として collection owner を保持して返す
5. F5s Ok の summary.items_read が 0 / 1 以外なら ItemDrainInvariantInvalid
6. summary.items_read == 0 なら collection owner を変更せず End / StepBudgetExhausted を返す
7. summary.items_read == 1 なら last_item Some を要求する
8. last_item を F5t collection push へ 1 回渡す
9. push Err は CollectionPushFailed として owner と rejected item を保持して返す
10. push Ok は collection owner、cursor、items_read、last_item、remaining_steps を更新する
11. F5s result が End なら End、budget exhausted なら StepBudgetExhausted、まだ budget があれば次 iteration へ進む
```

F5u は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection contour span

F5v は F5u/F5t の classified item collection owner から contour span を導出する境界である。これは byte-backed F4 contour span lookup への fallback ではない。collection に格納済みの item を authority とし、partial collection、forged item、endpoint topology mismatch を typed error として返す。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    -> Result GuiSfntSimpleGlyphContourSpan GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
```

success は既存 `GuiSfntSimpleGlyphContourSpan` を返す。

```text
GuiSfntSimpleGlyphContourSpan:
    glyph GuiGlyphId
    contour_index i32
    start_point_index i32
    end_point_index i32
    point_count i32
```

F5v は collection を借用して読むだけなので、collection owner を消費しない。error は owner recovery payload ではないが、診断に必要な typed context を保持する。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind:
    InvalidCapacity
    CollectionLengthMismatch
    CollectionCapacityMismatch
    CollectionIncomplete
    ContourIndexOutOfRange
    ItemReadFailed
    ItemGlyphMismatch
    ItemIndexMismatch
    ItemKindMismatch
    MissingContourEnd
    ContourCountMismatch
    FinalContourEndMismatch
    ContourSpanInvariantInvalid

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    contour_index i32
    item_index i32
    observed_contour_count i32
    last_endpoint_index i32
    item_count i32
    items_len i32
    items_cap i32
    read_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

F5v は次の順序を守る。

```text
1. capacity shape を検査する
2. items.len == item_count を検査する
3. items.cap == capacity.point_count を検査する
4. item_count == capacity.point_count を検査する
5. contour_index range を検査する
6. index 0 から point_count - 1 まで全 item を collection_read_item で読む
7. 各 item の glyph、point index、kind を再検査する
8. EndOnCurve / EndOffCurve だけを contour endpoint として数える
9. requested contour の start/end を記録しても scan は止めない
10. scan 後に requested contour が見つかったことを検査する
11. observed_contour_count == capacity.contour_count を検査する
12. last_endpoint_index == capacity.point_count - 1 を検査する
13. start/end から point_count を導出し、span invariant を検査してから success を返す
```

`observed_contour_count == capacity.contour_count` だけでは不十分である。例えば `contour_count = 2`、`point_count = 4`、endpoint が `[1, 2]` の forged collection は observed count が 2 でも point 3 がどの contour にも属さない。したがって F5v は最終 endpoint が必ず `point_count - 1` であることを `FinalContourEndMismatch` として検査する。

F5v は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_glyf_read_contour_endpoint
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection contour point

F5w は F5v の collection-backed contour span を authority として、contour-local point index から `GuiSfntSimpleGlyphContourPoint` を 1 点だけ取り出す境界である。F4 byte-backed contour point lookup へ戻らず、collection に格納済みの classified item を読む。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    contour_point_index i32
    -> Result GuiSfntSimpleGlyphContourPoint GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
```

success は既存 `GuiSfntSimpleGlyphContourPoint` を返す。

```text
GuiSfntSimpleGlyphContourPoint:
    span GuiSfntSimpleGlyphContourSpan
    contour_point_index i32
    point GuiSfntSimpleGlyphPoint
```

error は owner recovery payload ではない。collection は借用で読み、診断に必要な span failure、local index、absolute index、collection read failure、rejected item を保持する。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind:
    ContourSpanFailed
    ContourPointIndexOutOfRange
    ItemReadFailed
    ItemGlyphMismatch
    ItemIndexMismatch
    ItemKindMismatch
    ContourPointInvariantInvalid

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind
    contour_index i32
    contour_point_index i32
    absolute_point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    span Option GuiSfntSimpleGlyphContourSpan
    span_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    read_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
    item_count i32
    items_len i32
    items_cap i32
```

F5w は次の順序を守る。

```text
1. collection から capacity / item_count / items_len / items_cap を読む
2. F5v contour span lookup を exactly once 呼ぶ
3. F5v error は ContourSpanFailed として span_error に保持する
4. F5v success span について glyph、contour_index、start/end/count、capacity range を再検査する
5. span invariant failure は ContourPointInvariantInvalid とし、item は読まない
6. contour_point_index range を collection read より前に検査する
7. local range failure は ContourPointIndexOutOfRange とし、absolute_point_index は -1 にする
8. absolute_point_index = span.start_point_index + contour_point_index を計算する
9. absolute index が span/capacity range を外れるなら ContourPointInvariantInvalid とし、item は読まない
10. collection_read_item を exactly once 呼び、absolute point の item を読む
11. item の glyph、point index、kind を再検査する
12. success は gui_sfnt_simple_glyph_contour_point span contour_point_index point を返す
```

span invariant の再検査は F5v の契約を疑うためではなく、後続境界が lower boundary bug を別の error に誤分類しないための visible invariant である。`span.point_count == span.end_point_index - span.start_point_index + 1`、`span.glyph == capacity.glyph`、`span.end_point_index < capacity.point_count` を F5w でも確認する。

F5w は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
edge / path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection contour edge

F5x は F5v の collection-backed contour span と F5w の collection-backed contour point を authority として、contour-local edge index から `GuiSfntSimpleGlyphContourEdge` を 1 本だけ取り出す境界である。F4 byte-backed contour edge lookup へ戻らず、collection に格納済みの point pair だけで topology edge を構成する。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphContourEdge GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError
```

success は既存 `GuiSfntSimpleGlyphContourEdge` を返す。

```text
GuiSfntSimpleGlyphContourEdge:
    start GuiSfntSimpleGlyphContourPoint
    end GuiSfntSimpleGlyphContourPoint
    edge_index i32
    next_contour_point_index i32
```

error は owner recovery payload ではない。collection は借用で読み、診断に必要な span failure、start/end point failure、start/end point value、collection shape を保持する。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind:
    ContourSpanFailed
    EdgeIndexOutOfRange
    StartPointFailed
    EndPointFailed
    ContourEdgeInvariantInvalid

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind
    contour_index i32
    edge_index i32
    next_contour_point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    span Option GuiSfntSimpleGlyphContourSpan
    span_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    start_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
    end_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
    start Option GuiSfntSimpleGlyphContourPoint
    end Option GuiSfntSimpleGlyphContourPoint
    item_count i32
    items_len i32
    items_cap i32
```

F5x は次の順序を守る。

```text
1. collection から capacity / item_count / items_len / items_cap を読む
2. F5v contour span lookup を exactly once 呼ぶ
3. F5v error は ContourSpanFailed として span_error に保持する
4. F5v success span について glyph、contour_index、start/end/count、capacity range を再検査する
5. span invariant failure は ContourEdgeInvariantInvalid とし、start/end point は読まない
6. edge_index range を F5w point lookup より前に検査する
7. edge range failure は EdgeIndexOutOfRange とし、next_contour_point_index は -1 にする
8. next_contour_point_index を edge_index + 1 から計算し、contour end では 0 に wrap する
9. F5w contour point lookup を start / end の順で exactly twice 呼ぶ
10. lower point error は StartPointFailed / EndPointFailed として start_error / end_error に保持する
11. start span と end span が F5v span と一致することを再検査する
12. start local index == edge_index、end local index == next_contour_point_index を再検査する
13. start absolute index == span.start_point_index + edge_index を再検査する
14. end absolute index == span.start_point_index + next_contour_point_index を再検査する
15. success は gui_sfnt_simple_glyph_contour_edge start end edge_index next_contour_point_index を返す
```

1 point contour は valid topology として扱う。`span.point_count == 1` かつ `edge_index == 0` の場合、`next_contour_point_index == 0` となり、start / end は同じ absolute point を参照する。この self-wrap は implicit close ではなく contour topology 上の edge として保持する。

F5x は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection curve segment

F5y は F5x の collection-backed contour edge を authority として、1 本の edge から `GuiSfntSimpleGlyphCurveSegment` を分類する境界である。必要な lookahead point がある場合だけ F5w を呼び、F4 byte-backed curve segment helper や storage reader へ戻らない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphCurveSegment GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

success は既存 `GuiSfntSimpleGlyphCurveSegment` を返す。`NoSegment` は valid topology state であり、parse error ではない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentErrorKind:
    ContourEdgeFailed
    LookaheadPointFailed
    CurveSegmentInvariantInvalid

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentErrorKind
    contour_index i32
    edge_index i32
    next_contour_point_index i32
    lookahead_contour_point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    edge_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError
    lookahead_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
    edge Option GuiSfntSimpleGlyphContourEdge
    lookahead Option GuiSfntSimpleGlyphContourPoint
    item_count i32
    items_len i32
    items_cap i32
```

F5y は次の順序を守る。

```text
1. collection から capacity / item_count / items_len / items_cap を読む
2. F5x contour edge lookup を exactly once 呼ぶ
3. F5x error は ContourEdgeFailed として edge_error に保持する
4. F5x success edge から start / end / span / next_contour_point_index を読む
5. edge span の glyph、contour_index、start/end/count、capacity range を再検査する
6. start / end span が edge span と一致することを再検査する
7. start local index == edge_index を再検査する
8. end local index == next_contour_point_index を再検査する
9. start absolute index == span.start_point_index + edge_index を再検査する
10. end absolute index == span.start_point_index + next_contour_point_index を再検査する
11. recomputed next index が edge metadata と一致することを再検査する
12. edge invariant failure は CurveSegmentInvariantInvalid とし、lookahead 判定へ進まない
13. start on-curve かつ end off-curve の場合だけ lookahead index を計算する
14. lookahead index は next_contour_point_index + 1 を使い、contour end では 0 に wrap する
15. needed lookahead は F5w contour point lookup を exactly once 呼んで読む
16. needed lookahead failure は LookaheadPointFailed として lookahead_error に保持する
17. lookahead span / local index / absolute index を再検査する
18. lookahead invariant failure は CurveSegmentInvariantInvalid とする
19. needed lookahead success は gui_sfnt_classify_simple_glyph_curve_segment edge Option::Some lookahead を返す
20. lookahead 不要 path は F5w を呼ばず、gui_sfnt_classify_simple_glyph_curve_segment edge Option::None を返す
```

F5y は required lookahead を読めない場合に `Option::None` を渡して `MissingLookahead` を作ってはならない。`MissingLookahead` は lower pure classifier の防御的 state であり、collection-backed boundary では needed lookahead の失敗を `LookaheadPointFailed` として返す。一方で、1 point contour と off-curve start は valid `NoSegment` success として保持する。

F5y は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
path helpers
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path command pair

F5z は F5y の collection-backed curve segment を、既存の pure path command pair projection へ渡す境界である。これは contour stream、path command list、sink trait、rasterizer、renderer ではない。1 edge について `MoveTo` 相当の command と draw command を O(1) pair value として返すだけである。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathCommandPair GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5z は新しい error enum を持たない。F5y が失敗した場合は `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` をそのまま返す。F5y が成功した場合、`gui_sfnt_simple_glyph_curve_segment_path_command_pair` で `GuiSfntSimpleGlyphPathCommandPair` へ写す。`NoSegment` は parse error ではなく、既存 F4o と同じく move / draw の両方で explicit `SkipNoSegment` command として保持される。

F5z は次の順序を守る。

```text
1. F5y collection curve segment lookup を exactly once 呼ぶ
2. F5y error は変更せず Result::Err として返す
3. F5y success segment は gui_sfnt_simple_glyph_curve_segment_path_command_pair へ exactly once 渡す
4. pair projection result を Result::Ok として返す
```

F5z は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
push
sink traversal / event consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink event pair

F5aa は F5z の collection-backed path command pair を、既存の pure path sink event pair projection へ渡す境界である。これは sink trait、event consumer、contour traversal、path command list、rasterizer、renderer ではない。1 edge について first event と second event を O(1) pair value として返すだけである。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathSinkEventPair GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5aa は新しい error enum を持たない。F5z が失敗した場合は `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` をそのまま返す。F5z が成功した場合、`gui_sfnt_simple_glyph_path_command_pair_sink_event_pair` で `GuiSfntSimpleGlyphPathSinkEventPair` へ写す。この projection は total なので、`Option::None` や silent no-op へ変換してはならない。

F5aa は次の順序を守る。

```text
1. F5z collection path command pair lookup を exactly once 呼ぶ
2. F5z error は変更せず Result::Err として返す
3. F5z success pair は gui_sfnt_simple_glyph_path_command_pair_sink_event_pair へ exactly once 渡す
4. event pair projection result を Result::Ok として返す
```

F5aa は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
push
sink traversal / event consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink event kind pair

F5ab は F5aa の collection-backed path sink event pair を、既存の pure path sink event kind pair projection へ渡す境界である。これは sink trait、event consumer、contour traversal、path command list、rasterizer、renderer ではない。1 edge について first kind と second kind を O(1) pair value として返すだけである。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathSinkEventKindPair GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5ab は新しい error enum を持たない。F5aa が失敗した場合は `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` をそのまま返す。F5aa が成功した場合、`gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair` で `GuiSfntSimpleGlyphPathSinkEventKindPair` へ写す。この projection は total なので、`Option::None` や silent no-op へ変換してはならない。

F5ab は次の順序を守る。

```text
1. F5aa collection path sink event pair lookup を exactly once 呼ぶ
2. F5aa error は変更せず Result::Err として返す
3. F5aa success event pair は gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair へ exactly once 渡す
4. kind pair projection result を Result::Ok として返す
```

F5ab は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
push
sink traversal / event consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink event kind at

F5ac は F5ab の collection-backed path sink event kind pair を、既存の typed slot kind projection へ渡す境界である。これは sink trait、event consumer、contour traversal、path command list、rasterizer、renderer ではない。1 edge の first / second kind のうち、`GuiSfntSimpleGlyphPathSinkEventSlot` で指定された 1 kind だけを返す。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> Result GuiSfntSimpleGlyphPathSinkEventKind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5ac は新しい error enum を持たない。F5ab が失敗した場合は `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` をそのまま返す。F5ab が成功した場合、`gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at` で typed slot に対応する `GuiSfntSimpleGlyphPathSinkEventKind` へ写す。この slot projection は total なので、`Option::None`、silent no-op、fallback へ変換してはならない。

F5ac は次の順序を守る。

```text
1. F5ab collection path sink event kind pair lookup を exactly once 呼ぶ
2. F5ab error は変更せず Result::Err として返す
3. F5ab success kind pair は gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at へ exactly once 渡す
4. typed slot projection result を Result::Ok として返す
```

F5ac は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
push
sink traversal / event consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink event at

F5ad は F5aa の collection-backed path sink event pair を、既存の typed slot event projection へ渡す境界である。これは sink trait、event consumer、contour traversal、path command list、rasterizer、renderer ではない。1 edge の first / second event のうち、`GuiSfntSimpleGlyphPathSinkEventSlot` で指定された 1 event だけを返す。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> Result GuiSfntSimpleGlyphPathSinkEvent GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5ad は新しい error enum を持たない。F5aa が失敗した場合は `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` をそのまま返す。F5aa が成功した場合、`gui_sfnt_simple_glyph_path_sink_event_pair_event_at` で typed slot に対応する `GuiSfntSimpleGlyphPathSinkEvent` へ写す。この slot projection は total なので、`Option::None`、silent no-op、fallback へ変換してはならない。

F5ad は次の順序を守る。

```text
1. F5aa collection path sink event pair lookup を exactly once 呼ぶ
2. F5aa error は変更せず Result::Err として返す
3. F5aa success event pair は gui_sfnt_simple_glyph_path_sink_event_pair_event_at へ exactly once 渡す
4. typed slot projection result を Result::Ok として返す
```

F5ad は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_path_sink_event_pair_kind_at
gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span
gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget
gui_sfnt_simple_glyph_outline_storage_read_point_step
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_glyf_decode_
vec::
push
sink traversal / event consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path contour step

F5ae は F5ad の collection-backed path sink event at boundary を使い、`GuiSfntSimpleGlyphPathContourCursor` の現在位置を `GuiSfntSimpleGlyphPathContourStep` に写す境界である。これは contour-wide traversal、sink mutation、event consumer、path command list allocation、renderer、rasterizer ではない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepErrorKind:
    ContourSpanFailed
    CursorGlyphMismatch
    PathSinkEventFailed
```

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    cursor GuiSfntSimpleGlyphPathContourCursor
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    span_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    event_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphPathContourCursor
    -> Result GuiSfntSimpleGlyphPathContourStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ae はまず collection contour span lookup を exactly once 呼ぶ。失敗した場合は `ContourSpanFailed` として返し、`span_error = Some error` / `event_error = None` にする。span が成功した場合、F5ae は cursor glyph と collection capacity glyph を比較する。cursor glyph と collection capacity glyph が一致しない場合は `CursorGlyphMismatch` として返し、下位 event lookup へ進まない。

cursor glyph が一致した場合だけ、F5ae は F5ad `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at` を exactly once 呼ぶ。F5ad が失敗した場合は `PathSinkEventFailed` として返し、`span_error = None` / `event_error = Some error` にする。F5ad が成功した場合、F5ae は返された event から `gui_sfnt_simple_glyph_path_sink_event_kind` で kind を導く。F5ac は呼ばない。F5ac は kind だけを欲しい caller 用の sibling boundary であり、F5ae の authority ではない。

成功 path は次の順序を守る。

```text
1. collection contour span lookup を exactly once 呼ぶ
2. cursor glyph と collection capacity glyph を比較する
3. F5ad collection path sink event at lookup を exactly once 呼ぶ
4. returned event から kind を exactly once 導く
5. private cursor next helper で next state を作る
6. GuiSfntSimpleGlyphPathContourStep を作る
```

F5ae は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point
gui_sfnt_lookup_simple_glyph_path_contour_step
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_lookup_simple_glyph_curve_segment
gui_sfnt_lookup_simple_glyph_contour_edge
gui_sfnt_lookup_simple_glyph_contour_point
gui_sfnt_lookup_simple_glyph_contour_span
gui_sfnt_glyf_simple_curve_segment_with_tables
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
gui_sfnt_glyf_simple_contour_span_with_tables
vec::
push
sink traversal / event consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink step

F5af は F5ae の collection-backed path contour step を authority として、`GuiSfntSimpleGlyphPathSinkPolicy` による policy decision を合成し、`GuiSfntSimpleGlyphPathSinkStep` を返す境界である。これは contour-wide traversal、action step traversal、sink consumer、path command list allocation、renderer、rasterizer ではない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphPathContourCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5af は F5ae `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step` を exactly once 呼ぶ。F5ae が `Result::Err` を返した場合、F5af は error を wrap せず同じ error value として返す。F5af 自身は新しい fallible authority を持たないため、専用 error enum を追加しない。

F5ae が `Result::Ok contour_step` を返した場合、F5af は pure helper `gui_sfnt_simple_glyph_path_sink_step_from_contour_step` を exactly once 呼び、policy decision と tail close handling を既存の sink step contract に委譲する。policy reject は `Result::Err` ではなく、`GuiSfntSimpleGlyphPathSinkStep.primary_action = Reject` として成功 payload に残る。

成功 path は次の順序を守る。

```text
1. F5ae collection-backed contour step lookup を exactly once 呼ぶ
2. error は wrap せず Result::Err error として返す
3. success contour_step を pure sink-step projection に渡す
4. Result::Ok sink_step を返す
```

F5af は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_lookup_simple_glyph_path_sink_step
gui_sfnt_lookup_simple_glyph_path_contour_step
gui_sfnt_lookup_simple_glyph_path_command_pair
gui_sfnt_simple_glyph_path_sink_action
gui_sfnt_simple_glyph_path_sink_action_step
vec::
push
sink traversal / action APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink action step

F5ag は F5af の collection-backed sink step を authority として、`GuiSfntSimpleGlyphPathSinkActionCursor` から `GuiSfntSimpleGlyphPathSinkActionStep` を 1 つ返す境界である。これは action stream 全体の traversal、checked advance、sink consumer、path command list allocation、renderer、rasterizer ではない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphPathSinkActionCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ag は action cursor を `contour_cursor` と `action_slot` に分解する。`contour_cursor` は F5af `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step` に渡し、`action_slot` は pure `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` に渡す。F5af が `Result::Err` を返した場合、F5ag は error を wrap せず同じ error value として返す。

F5af が `Result::Ok sink_step` を返した場合、F5ag は pure helper `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` を exactly once 呼ぶ。この pure helper が action selection と action next を決める。`Primary` は同じ contour cursor の `Tail` へ進み、`Tail` は source step の next に従う。policy reject は `Result::Err` ではなく、action step の action payload に残る。

成功 path は次の順序を守る。

```text
1. action cursor から contour cursor を読む
2. action cursor から action slot を読む
3. F5af collection-backed sink step lookup を exactly once 呼ぶ
4. error は wrap せず Result::Err error として返す
5. success sink_step を pure action-step projection に渡す
6. Result::Ok action_step を返す
```

F5ag は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_lookup_simple_glyph_path_sink_action_step
gui_sfnt_lookup_simple_glyph_path_sink_step
gui_sfnt_lookup_simple_glyph_path_contour_step
gui_sfnt_simple_glyph_path_sink_action_step_advance
gui_sfnt_simple_glyph_path_sink_action_step_item
vec::
push
sink traversal / consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink action step advance and item

F5ah は F5ag の collection-backed action step lookup を authority として、action stream を 1 action 分だけ checked advance 可能な typed item にする境界である。これは contour-wide traversal、consumer、real sink、renderer、rasterizer、platform API ではない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_advance:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    step &GuiSfntSimpleGlyphPathSinkActionStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepAdvance GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    step &GuiSfntSimpleGlyphPathSinkActionStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

advance helper は `gui_sfnt_simple_glyph_path_sink_action_step_next step` を exactly once 読み、その enum だけを `match` する。

```text
next = Continue cursor
    -> gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step collection cursor policy
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step

next = EndContour
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour
```

`EndContour` は `Option::None` や `Result::Err` ではなく successful terminal state である。`Result::Err` は `Continue cursor` の下位 F5ag lookup から来た typed collection contour step error だけを伝播する。policy reject は action payload に残し、F5ah が `Reject` / `NoAction` / `CloseContour` を見て traversal を変えない。

item helper は advance helper に exactly once 委譲する。`Result::Err` は wrap せず、`Result::Ok advance` なら現在 step を `*step` で明示 copy して `GuiSfntSimpleGlyphPathSinkActionStepItem` に束ねる。

F5ah は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_lookup_simple_glyph_path_sink_action_step
gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance
gui_sfnt_lookup_simple_glyph_path_sink_action_step_item
gui_sfnt_lookup_simple_glyph_path_sink_step
gui_sfnt_lookup_simple_glyph_path_contour_step
vec::
push
sink traversal / consumer APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink action item next and consumer item

F5ai は F5ah の collection-backed action step item を authority として、checked advance を次 action item へ進め、同じ境界で current action payload と checked next state を future sink consumer 用 packet に束ねる段階である。これは byte-backed F4ab/F4ac を collection-backed item stream に写したものであり、byte buffer、font table metadata、sink traversal、real sink mutation、renderer、rasterizer、platform API へ戻らない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_item_next:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item &GuiSfntSimpleGlyphPathSinkActionStepItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item &GuiSfntSimpleGlyphPathSinkActionStepItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

action item next helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_advance item` を exactly once 読み、その enum だけを `match` する。

```text
advance = Continue next_step
    -> gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item collection &next_step policy
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item

advance = EndContour
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
```

`EndContour` は successful terminal state であり、`Option::None`、`Result::Err`、silent no-op、hidden fallback に変換しない。`Result::Err` は `Continue next_step` の下位 F5ah lookup から来た typed collection contour step error だけを伝播する。

consumer item helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_step item` を exactly once 読み、`gui_sfnt_simple_glyph_path_sink_action_step_action &stored_step` で current action を exactly once value として copy する。next state は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_item_next collection item policy` を exactly once 呼んで得る。`Result::Err` は wrap せず、`Result::Ok next` の場合だけ `GuiSfntSimpleGlyphPathSinkActionConsumerItem action next` を返す。

F5ai は action payload を解釈しない。`EmitEvent`、`Reject`、`NoAction`、`CloseContour` は consumer item の `action` に保持され、後続の明示 consumer / apply phase が読む。F5ai は次を直接呼ばない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_lookup_simple_glyph_path_sink_action_item_next
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item
gui_sfnt_lookup_simple_glyph_path_sink_action_step_item
gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance
gui_sfnt_lookup_simple_glyph_path_sink_action_step
gui_sfnt_lookup_simple_glyph_path_sink_step
gui_sfnt_lookup_simple_glyph_path_contour_step
vec::
push
consumer apply / consume / traversal APIs
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink action consumer next and consume once

F5aj は F5ai の collection-backed action consumer item を authority として、consumer stream を 1 step 進める境界である。byte-backed F4ad/F4ah/F4ai と同じ責務分割を保つが、font bytes、table metadata、byte-backed lookup helper、lower collection helper、sink traversal、renderer、rasterizer、platform API へ戻らない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_next:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItemNext GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_apply_advance:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    step &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_consume_once:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

consumer item next helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item` を exactly once 読み、その saved next だけを `match` する。

```text
next = Continue next_item
    -> gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item collection &next_item policy
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer_item

next = EndContour
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour
```

consumer apply advance helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step` を exactly once 読む。`Continue continue_step` の場合だけ `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next &continue_step` を exactly once 読み、その saved next を authority とする。

```text
terminal = Continue continue_step
    saved_next = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next &continue_step
    saved_next = Continue next_item
        -> gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item collection &next_item policy
        -> Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue next_consumer_item
    saved_next = EndContour
        -> Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour

terminal = Rejected reason
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected reason

terminal = EndContour
    -> Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour
```

この branch は original consumer item を要求せず、action payload を再読込・再解釈しない。policy reject は typed `Rejected reason` terminal として保持し、silent no-op や fallback へ変換しない。`EndContour` は successful terminal state であり、`Option::None` や `Result::Err` にしない。

consume-once helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item` を exactly once 呼び、collection apply advance helper を exactly once 呼ぶ。`Result::Ok advance` の場合だけ `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step apply_step advance` へ exactly once 渡し、apply step と advance を捨てず typed consume step に束ねる。

F5aj は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item
gui_sfnt_lookup_simple_glyph_path_sink_action_step_item
gui_sfnt_lookup_simple_glyph_path_sink_action_step
gui_sfnt_lookup_simple_glyph_path_sink_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment
vec::
push
payload direct match / original item reinterpretation
sink traversal / real sink mutation
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink action start consumer

F5ak は collection-backed action stream の contour start boundary である。F5aj が既存 consumer item から 1 step 進める境界であるのに対し、F5ak は collection が保持する glyph authority から first item、first consumer item、first consume step、first consume summary を作る。caller supplied glyph は受け取らない。collection-backed API で外部 glyph を受け取ると forged cursor を作れるため、glyph は必ず `collection_capacity -> capacity.glyph` から読む。authority sequence は `collection_capacity -> capacity.glyph -> start_cursor -> F5ag action step -> F5ah step item` である。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consumer_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_once:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

start item helper は次の順序を守る。

```text
capacity = gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection
glyph = gui_sfnt_simple_glyph_outline_storage_capacity_glyph &capacity
start_cursor = gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
start_step = gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step collection start_cursor policy
item = gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item collection &start_step policy
```

この helper だけが F5ag action step と F5ah action step item を直接呼ぶ。F5ag は cursor glyph と collection capacity glyph の一致を F5ae で検査するため、F5ak の start cursor は collection capacity glyph から作る必要がある。

start consumer item helper は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_item collection contour_index policy` を exactly once 呼び、成功時だけ `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item collection &item policy` を exactly once 呼ぶ。

start consume-once helper は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consumer_item collection contour_index policy` を exactly once 呼び、成功時だけ `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_consume_once collection state &consumer_item policy` を exactly once 呼ぶ。

start consume summary helper は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_once collection state contour_index policy` を exactly once 呼び、成功時だけ `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を exactly once 呼ぶ。summary projection は失敗しないため、新しい error domain は追加しない。

F5ak は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_item
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once
caller supplied glyph
lower F5 helper from higher F5ak helper
summary advance / summary drain
vec::
push
sink traversal / real sink mutation
render / raster / platform / host APIs
```

### SFNT simple glyph outline point stream item collection path sink action consume summary drain

F5al は F5ak の collection-backed start consume summary と F5aj の consume-once をつなぎ、collection-backed action consumer を explicit budget 内で domain terminal まで進める boundary である。F4aq の byte-backed drain と同じ terminal contract を使うが、byte-backed glyph lookup へ戻らない。これは outline allocation、sink mutation、renderer、rasterizer、platform API を持たない。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_consume_summary_advance_once:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_consume_summary_drain_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

advance-once helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary` を 1 回だけ読み、続いて `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を 1 回だけ読む。`Continue item` の場合だけ F5aj `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_consume_once collection state &item policy` を 1 回だけ呼ぶ。成功時だけ `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を 1 回だけ呼び、新しい summary を `Continue` として返す。`Rejected` と `EndContour` は parse error ではなく `Result::Ok` の domain terminal として返す。

drain helper は terminal-before-budget の順序を守り、budget 判定より先に `summary_terminal` を 1 回だけ読む。`Rejected reason` と `EndContour` は budget を消費せず、current summary と一緒に `Result::Ok` で返す。`Continue` かつ `remaining_steps <= 0` は `StepBudgetExhausted current_summary` を返す。`Continue` かつ `remaining_steps > 0` の場合だけ F5al advance-once を 1 回だけ呼び、`Result::Err error` はそのまま伝播する。advance-once が `Continue next_summary` を返した場合は `remaining_steps - 1` で同じ drain helper へ再帰する。advance-once が保守上 `Rejected` または `EndContour` を返した場合は、advance-once に渡した current summary を drain result に入れる。

start drain helper は F5ak `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary collection state contour_index policy` を 1 回だけ呼び、成功時だけ F5al drain helper へ 1 回渡す。start drain helper は F5al advance-once、F5aj consume-once、F5ak の lower start helper、F4 byte-backed helper を直接呼ばない。

F5al は次を直接呼ばない。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action*
gui_sfnt_glyf_*_with_tables
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event*
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item
F5ak lower start helpers from start drain helper
F5aj consume-once from start drain helper
F5al advance-once from start drain helper
action payload direct match
vec::
push
sink traversal / real sink mutation
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action drain outcome

F5am は F5al の collection-backed start drain result を、同じ collection の capacity と一緒に後続 outline / path owner 境界へ渡すための value-only boundary である。F5am は owner allocation、path command push、sink mutation、rasterizer、renderer、platform API、host text API、font fallback を持たない。

F5am の public API は start drain から outcome までを同じ呼び出しで行う helper だけである。drain result と collection capacity を任意に組み合わせる public projection API は提供しない。private projection は、public start outcome helper が F5al start drain に成功した直後の drain value だけを capacity 付き outcome に写すための内部 helper である。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    summary GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    rejected GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryRejected

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainOutcome:
    EndContour GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    Rejected GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_outcome_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainOutcome GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

public start outcome helper は F5al `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_budget collection state contour_index policy remaining_steps` を 1 回だけ呼ぶ。`Result::Err error` はそのまま返す。`Result::Ok drain` の場合だけ private projection を 1 回だけ呼び、private projection は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection` を 1 回だけ読んで、`EndContour`、`Rejected`、`StepBudgetExhausted` を capacity 付き outcome へ写す。

`Rejected` は string や fallback state に変換しない。既存の `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryRejected` を capacity と一緒に保持し、後続 boundary が enum `match` で拒否理由と停止 summary を扱えるようにする。`StepBudgetExhausted` も成功ではなく追加 work slice が必要な typed terminal として扱う。

F5am は次を直接呼ばない。

```text
F5al advance-once
F5al drain helper from private projection
F5ak lower start helpers
F5aj consume-once
F4 byte-backed lookup helper
lower collection path event / contour / step helpers
byte-backed table helper
Vec / push
path command owner allocation
sink traversal / real sink mutation
render / raster / platform / host APIs
font fallback
public forged collection/drain pairing API
```

### SFNT simple glyph outline point stream item collection path sink action storage owner

F5an は F5am の capacity 付き drain outcome を authority として、`EndContour` の場合だけ F5b outline storage allocation へ進む owner-taking boundary である。F5an は collection、drain result、byte-backed table、path sink、renderer、platform API を直接受け取らない。caller が別 collection と別 drain result を組み合わせて owner allocation へ進める public API は提供しない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageAllocError:
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    alloc_error GuiSfntSimpleGlyphOutlineStorageAllocError

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageTerminal:
    Allocated GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageOwner
    Rejected GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_drain_outcome_alloc_storage_owner:
    outcome GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainOutcome
    limit &GuiSfntSimpleGlyphOutlineStorageLimit
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageAllocError
```

`EndContour drain_summary` では `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_drain_summary_capacity &drain_summary` を 1 回だけ呼び、得た capacity で `gui_sfnt_simple_glyph_outline_storage_alloc &capacity limit` を 1 回だけ呼ぶ。allocation 成功時は `Allocated StorageOwner` を返し、allocation 失敗時だけ `StorageAllocError` を `Result::Err` で返す。

`Rejected drain_rejected` と `StepBudgetExhausted drain_summary` は typed terminal であり、storage allocation failure ではない。そのため `Result::Ok StorageTerminal::Rejected drain_rejected` または `Result::Ok StorageTerminal::StepBudgetExhausted drain_summary` として caller へ返す。これらの branch では storage owner を作らず、F5b allocation も呼ばない。

`StorageOwner` と `StorageTerminal` は owner を含むため `Clone` / `Copy` を実装しない。`StorageAllocError` は owner を含まず、F5am drain summary と F5b allocation error を caller が診断や回復に使える typed payload として保持する。

F5an は次を直接呼ばない。

```text
F5al start / advance / drain helper
F5ak lower start helpers
F5aj consume-once
F4 byte-backed lookup helper
lower collection path event / contour / step helpers
byte-backed table helper
Vec / push
slot population
path command owner fill
sink traversal / real sink mutation
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action contour endpoint start

F5ao は F5an の storage terminal を authority として、`Allocated StorageOwner` の場合だけ F5d contour endpoint region cursor start へ進む owner-recovery boundary である。F5ao は endpoint slot population、byte-backed endpoint read、path sink traversal、renderer、platform API を直接呼ばない。

F5an の `StorageOwner` は public constructor を持つため、F5ao は forged owner を fail-closed に扱う。cursor start より前に summary capacity と owner 内 storage の trusted capacity を非消費で比較し、一致しない場合は original owner を保持した `ContourEndpointStartError` を返す。

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_owner_storage_capacity:
    owner &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageOwner
    -> GuiSfntSimpleGlyphOutlineStorageCapacity

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    previous_endpoint Option i32

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartErrorKind:
    StorageSummaryCapacityMismatch
    CursorStartFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartError:
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageOwner
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartErrorKind
    cursor_error Option StdErrorKind

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartTerminal:
    Started GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartOwner
    Rejected GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_terminal_start_contour_endpoint:
    terminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageTerminal
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartError
```

`storage_owner_storage_capacity` は owner を消費しない。implementation は `field::get_ref owner "storage"` で borrowed storage を得て、既存の `gui_sfnt_simple_glyph_outline_storage_capacity storage` を呼ぶ。`gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_owner_storage owner` は consuming accessor なので、capacity mismatch や cursor start failure の前には呼ばない。

`Allocated owner` branch では、summary capacity と borrowed storage capacity の glyph、contour count、point count、edge count、path command pair count、path command count が一致することを先に検査する。不一致なら `StorageSummaryCapacityMismatch` を `Result::Err` で返し、error payload が original owner を保持する。

capacity が一致した後だけ、F5d `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint` を 1 回だけ呼ぶ。cursor start が失敗した場合は `CursorStartFailed` と cursor error を保持した `Result::Err` を返し、original owner を失わない。

cursor start が成功した場合だけ、F5an の consuming storage accessor を 1 回だけ呼び、`previous_endpoint = none` を持つ `Started ContourEndpointStartOwner` を返す。`Rejected` と `StepBudgetExhausted` は typed terminal として `Result::Ok` で通過し、capacity match、storage capacity read、cursor start、storage consume を行わない。

`ContourEndpointStartOwner`、`ContourEndpointStartError`、`ContourEndpointStartTerminal` は owner を含むため `Clone` / `Copy` を実装しない。`ContourEndpointStartErrorKind` は owner を含まないため `Clone` / `Copy` を実装してよい。

F5ao は次を直接呼ばない。

```text
F5al start / advance / drain helper
F5ak lower start helpers
F5aj consume-once
F4 byte-backed lookup helper
lower collection path event / contour / step helpers
byte-backed table helper
Vec / push
endpoint push
point / curve / path command population
sink traversal / real sink mutation
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action contour endpoint push

F5ap は F5ao の contour endpoint start terminal を authority として、`Started StartOwner` の場合だけ F5e typed contour endpoint push へ進む owner-recovery boundary である。F5ap は endpoint slot を 1 件だけ受け取り、iteration、byte-backed endpoint read、path sink traversal、renderer、platform API を直接呼ばない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    previous_endpoint Option i32

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushError:
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartOwner
    endpoint GuiSfntSimpleGlyphContourEndpointSlot
    push_error_kind GuiSfntSimpleGlyphContourEndpointPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushTerminal:
    Pushed GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushOwner
    Rejected GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start_terminal_push_endpoint:
    terminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartTerminal
    endpoint GuiSfntSimpleGlyphContourEndpointSlot
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushError
```

`Started owner` branch では、owner を消費する前に summary、cursor、previous endpoint を borrow-copy する。その後で `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start_owner_storage owner` を 1 回だけ呼び、F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint previous_endpoint` を 1 回だけ呼ぶ。

F5e push が成功した場合は、F5e の returned storage、returned cursor、returned previous endpoint を使う。previous endpoint は `some next_previous_endpoint_value` に包み、`Pushed ContourEndpointPushOwner` を返す。

F5e push が失敗した場合は、lower metadata を storage 回収より前に読む。

```text
gui_sfnt_simple_glyph_contour_endpoint_push_error_kind &push_error
gui_sfnt_simple_glyph_contour_endpoint_push_error_region_error_kind &push_error
gui_sfnt_simple_glyph_contour_endpoint_push_error_push_error_kind &push_error
gui_sfnt_simple_glyph_contour_endpoint_push_error_storage push_error
```

returned storage と保存済みの summary、cursor、previous endpoint から `ContourEndpointStartOwner` を復元し、endpoint と lower metadata を持つ `ContourEndpointPushError` を `Result::Err` で返す。

`Rejected drain_rejected` と `StepBudgetExhausted drain_summary` は typed terminal であり、endpoint push failure ではない。そのため `Result::Ok ContourEndpointPushTerminal::Rejected drain_rejected` または `Result::Ok ContourEndpointPushTerminal::StepBudgetExhausted drain_summary` として caller へ返す。これらの branch では endpoint を読まず、F5e push、storage consume、owner/error construction を行わない。

`ContourEndpointPushOwner`、`ContourEndpointPushError`、`ContourEndpointPushTerminal` は owner を含むため `Clone` / `Copy` を実装しない。

F5ap は次を直接呼ばない。

```text
F5al start / advance / drain helper
F5ak lower start helpers
F5aj consume-once
F4 byte-backed lookup helper
byte-backed endpoint read / read-push helper
lower collection path event / contour / step helpers
path sink traversal / real sink mutation
point / curve / path command population
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action contour endpoint drain

F5aq は F5ap の `ContourEndpointPushOwner` を authority として、collection-backed contour span から残りの contour endpoint slot を bounded drain し、全 contour endpoint 完了後だけ PointX region cursor start へ進む owner-recovery boundary である。F5aq は PointX value push、byte-backed endpoint read、path sink traversal、renderer、platform API を直接呼ばない。

`ContourEndpointPushOwner` は public constructor を持つため、F5aq は owner を消費する前に authority を固定順で検査する。

```text
authority check order:
    summary capacity == owner storage capacity
    cursor well formed
    cursor region == ContourEndpoint
    cursor matches summary capacity ContourEndpoint region
    collection capacity == summary capacity
```

この順序より前に span lookup、PointX cursor start、storage consume、F5e push を行ってはいけない。各 authority failure は current `ContourEndpointPushOwner` を保持した typed error として返す。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    EndpointSourceFailed
    EndpointPushFailed
    PointXCursorStartFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainError:
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushOwner
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainErrorKind
    contour_index i32
    source_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    endpoint Option GuiSfntSimpleGlyphContourEndpointSlot
    push_error_kind Option GuiSfntSimpleGlyphContourEndpointPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainTerminal:
    PointXStarted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXStartOwner
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushOwner

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_owner_drain_to_point_x_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushOwner
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainError
```

authority check 後だけ cursor の `next_index`、`start`、`end` を読む。`next_index == end` の場合だけ `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX` で PointX cursor を開始する。成功した場合は storage を消費して `PointXStarted PointXStartOwner` を返す。失敗した場合は `PointXCursorStartFailed` と lower `StdErrorKind` を持つ owner-preserving error を返す。

`next_index < end` かつ `remaining_steps <= 0` の場合は、span lookup も mutation も行わず `StepBudgetExhausted ContourEndpointPushOwner` を返す。

`next_index < end` かつ `remaining_steps > 0` の場合、contour index は `next_index - start` である。endpoint source は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span collection contour_index` だけである。span failure は lower `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError` と current owner を保持した `EndpointSourceFailed` とする。span success は span の `end_point_index` から `GuiSfntSimpleGlyphContourEndpointSlot` を作る。

F5e push は internal push helper だけが呼ぶ。helper は summary、cursor、previous endpoint を borrow-copy してから storage を消費し、`gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint previous_endpoint` を 1 回だけ呼ぶ。F5e failure では lower metadata を storage 回収より前に読み、returned storage と保存済み summary、cursor、previous endpoint から `ContourEndpointPushOwner` を復元し、`EndpointPushFailed` を返す。

push success は F5e returned storage、returned cursor、returned previous endpoint だけから次の PushOwner を作り、`remaining_steps - 1` で drain を継続する。

`PointXStartOwner`、`ContourEndpointDrainError`、`ContourEndpointDrainTerminal` は owner を含むため `Clone` / `Copy` を実装しない。

F5aq は次を直接呼ばない。

```text
F4 byte-backed lookup helper
byte-backed endpoint read / read-push helper
F5al / F5ak / F5aj traversal helper
lower collection path event / contour / step helpers
path sink traversal / real sink mutation
PointX value push
point / curve / path command population
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action PointX drain

F5ar は F5aq の `PointXStartOwner` を authority として、collection-backed point stream item source から PointX region の scalar slot を bounded drain し、全 PointX slot 完了後だけ PointY region cursor start へ進む owner-recovery boundary である。F5ar は PointY value push、byte-backed coordinate reader、path sink traversal、renderer、platform API を直接呼ばない。

`PointXStartOwner` は public constructor を持つため、F5ar は owner を消費する前に authority を固定順で検査する。

```text
authority check order:
    summary capacity == owner storage capacity
    cursor well formed
    cursor region == PointX
    cursor matches summary capacity PointX region
    collection capacity == summary capacity
```

この順序より前に collection item read、PointX push、PointY cursor start、storage consume を行ってはいけない。各 authority failure は current `PointXStartOwner` を保持した typed error として返す。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    PointSourceReadFailed
    PointSourceGlyphMismatch
    PointSourceIndexMismatch
    PointSourceKindMismatch
    PointXPushFailed
    PointYCursorStartFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainError:
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXStartOwner
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainErrorKind
    point_index i32
    read_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
    point Option GuiSfntSimpleGlyphPointXSlot
    push_error_kind Option GuiSfntSimpleGlyphPointXPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainTerminal:
    PointYStarted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYStartOwner
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXStartOwner

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_x_start_owner_drain_to_point_y_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXStartOwner
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainError
```

authority check 後だけ cursor の `next_index`、`start`、`end` を読む。`next_index == end` の場合だけ `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY` で PointY cursor を開始する。成功した場合は storage を消費して `PointYStarted PointYStartOwner` を返す。失敗した場合は `PointYCursorStartFailed` と lower `StdErrorKind` を持つ owner-preserving error を返す。

`next_index < end` かつ `remaining_steps <= 0` の場合は、collection read も mutation も行わず `StepBudgetExhausted PointXStartOwner` を返す。

`next_index < end` かつ `remaining_steps > 0` の場合、logical point index は `next_index - start` である。PointX source は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item collection point_index` だけである。read failure は lower `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError` と current owner を保持した `PointSourceReadFailed` とする。

read success 後は、item payload が forged でないことを caller 側で再検査する。item point の glyph raw id は summary capacity glyph raw id と一致しなければならない。item point index は `point_index` と一致しなければならない。`gui_sfnt_simple_glyph_outline_point_stream_item_kind_matches_point &item` は true でなければならない。失敗した場合は `PointSourceGlyphMismatch`、`PointSourceIndexMismatch`、`PointSourceKindMismatch` として item を保持した typed error を返し、PointX push へ進まない。

PointX push は internal push helper だけが呼ぶ。helper は summary と cursor を borrow-copy してから storage を消費し、`gui_sfnt_simple_glyph_outline_storage_push_point_x storage cursor point` を 1 回だけ呼ぶ。PointX push failure では lower metadata を storage 回収より前に読む。

```text
read gui_sfnt_simple_glyph_point_x_push_error_kind &push_error
read gui_sfnt_simple_glyph_point_x_push_error_point &push_error
read gui_sfnt_simple_glyph_point_x_push_error_region_error_kind &push_error
read gui_sfnt_simple_glyph_point_x_push_error_push_error_kind &push_error
consume gui_sfnt_simple_glyph_point_x_push_error_storage push_error
```

returned storage と保存済み summary / cursor から `PointXStartOwner` を復元し、`PointXPushFailed` を返す。push success は F5g returned storage、returned cursor だけから次の PointXStartOwner を作り、`remaining_steps - 1` で drain を継続する。

`PointYStartOwner`、`PointXDrainError`、`PointXDrainTerminal` は owner を含むため `Clone` / `Copy` を実装しない。`PointXDrainErrorKind` は value enum なので `Clone` / `Copy` を実装してよい。

F5ar は次を直接呼ばない。

```text
F4 byte-backed lookup helper
byte-backed PointX read / read-push helper
byte-backed PointY read / read-push helper
F5al / F5ak / F5aj traversal helper
lower collection path event / contour / step helpers
path sink traversal / real sink mutation
PointY value push
point / curve / path command population
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action PointY drain

F5as は F5ar の `PointYStartOwner` を authority として、collection-backed point stream item source から PointY region の scalar slot を bounded drain し、全 PointY slot 完了後だけ Edge region cursor start へ進む owner-recovery boundary である。F5as は edge value population、path command population、byte-backed coordinate reader、path sink traversal、renderer、platform API を直接呼ばない。

`PointYStartOwner` は public constructor を持つため、F5as は owner を消費する前に authority を固定順で検査する。

```text
authority check order:
    summary capacity == owner storage capacity
    cursor well formed
    cursor region == PointY
    cursor matches summary capacity PointY region
    collection capacity == summary capacity
```

この順序より前に collection item read、PointY push、Edge cursor start、storage consume を行ってはいけない。各 authority failure は current `PointYStartOwner` を保持した typed error として返す。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    PointSourceReadFailed
    PointSourceGlyphMismatch
    PointSourceIndexMismatch
    PointSourceKindMismatch
    PointYPushFailed
    EdgeCursorStartFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainError:
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYStartOwner
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainErrorKind
    point_index i32
    read_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
    point Option GuiSfntSimpleGlyphPointYSlot
    push_error_kind Option GuiSfntSimpleGlyphPointYPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainTerminal:
    EdgeStarted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeStartOwner
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYStartOwner

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_y_start_owner_drain_to_edge_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYStartOwner
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainError
```

authority check 後だけ cursor の `next_index`、`start`、`end` を読む。`next_index == end` の場合だけ `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::Edge` で Edge cursor を開始する。成功した場合は storage を消費して `EdgeStarted EdgeStartOwner` を返す。失敗した場合は `EdgeCursorStartFailed` と lower `StdErrorKind` を持つ owner-preserving error を返す。

`next_index < end` かつ `remaining_steps <= 0` の場合は、collection read も mutation も行わず `StepBudgetExhausted PointYStartOwner` を返す。

`next_index < end` かつ `remaining_steps > 0` の場合、logical point index は `next_index - start` である。PointY source は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item collection point_index` だけである。read failure は lower `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError` と current owner を保持した `PointSourceReadFailed` とする。

read success 後は、item payload が forged でないことを caller 側で再検査する。item point の glyph raw id は summary capacity glyph raw id と一致しなければならない。item point index は `point_index` と一致しなければならない。`gui_sfnt_simple_glyph_outline_point_stream_item_kind_matches_point &item` は true でなければならない。失敗した場合は `PointSourceGlyphMismatch`、`PointSourceIndexMismatch`、`PointSourceKindMismatch` として item を保持した typed error を返し、PointY push へ進まない。

PointY push は internal push helper だけが呼ぶ。helper は summary と cursor を borrow-copy してから storage を消費し、`gui_sfnt_simple_glyph_outline_storage_push_point_y storage cursor point` を 1 回だけ呼ぶ。PointY push failure では lower metadata を storage 回収より前に読む。

```text
read gui_sfnt_simple_glyph_point_y_push_error_kind &push_error
read gui_sfnt_simple_glyph_point_y_push_error_point &push_error
read gui_sfnt_simple_glyph_point_y_push_error_region_error_kind &push_error
read gui_sfnt_simple_glyph_point_y_push_error_push_error_kind &push_error
consume gui_sfnt_simple_glyph_point_y_push_error_storage push_error
```

returned storage と保存済み summary / cursor から `PointYStartOwner` を復元し、`PointYPushFailed` を返す。push success は F5i returned storage、returned cursor だけから次の PointYStartOwner を作り、`remaining_steps - 1` で drain を継続する。

`EdgeStartOwner`、`PointYDrainError`、`PointYDrainTerminal` は owner を含むため `Clone` / `Copy` を実装しない。`PointYDrainErrorKind` は value enum なので `Clone` / `Copy` を実装してよい。

F5as は次を直接呼ばない。

```text
F4 byte-backed lookup helper
byte-backed PointX read / read-push helper
byte-backed PointY read / read-push helper
F5al / F5ak / F5aj traversal helper
lower collection path event / contour / step helpers
path sink traversal / real sink mutation
PointX value push
edge value population
path command population
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action Edge drain

F5at は F5as の `EdgeStartOwner` を authority として、owner storage の endpoint marker と collection-backed contour span / contour edge source から Edge region の scalar slot を bounded drain し、全 Edge slot 完了後だけ PathCommandTag region cursor start へ進む owner-recovery boundary である。F5at は curve segment classification、path command tag population、byte-backed lookup、path sink traversal、renderer、platform API を直接呼ばない。

Edge region の slot contract は次である。

```text
slot index:
    global_edge_index == absolute start point index

stored scalar value:
    contour_index

derived local edge index:
    global_edge_index - span.start_point_index
```

local edge index は scalar として保存しない。PathCommandTag phase は edge slot の `global_edge_index` と保存済み `contour_index` から collection-backed source を再び検査し、local edge index を導出する。これにより Edge region は contour ownership だけを保持し、curve segment / path command の分類 authority を次 phase に残す。

`EdgeStartOwner` は public constructor を持つため、F5at は owner を消費する前に authority を固定順で検査する。

```text
authority check order:
    summary capacity == owner storage capacity
    cursor well formed
    cursor region == Edge
    cursor matches summary capacity Edge region
    collection capacity == summary capacity
```

この順序より前に endpoint marker read、collection contour span / contour edge source、Edge push、PathCommandTag cursor start、storage consume を行ってはいけない。`cursor matches summary capacity Edge region` は cursor の region / start / end が capacity と一致することを意味し、`next_index` を Edge region start に固定しない。`StepBudgetExhausted EdgeStartOwner` からの partial drain 再開を拒否してはいけない。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeSlot:
    edge_index i32
    contour_index i32
    contour_edge_index i32
    next_contour_point_index i32

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    EndpointMarkerReadFailed
    EndpointMarkerGlyphMismatch
    EndpointMarkerIndexMismatch
    ContourSpanSourceFailed
    ContourSpanInvariantMismatch
    ContourEdgeSourceFailed
    EdgeSourceContourMismatch
    EdgeSourceIndexMismatch
    EdgeSourceNextIndexMismatch
    EdgePushFailed
    PathCommandTagCursorStartFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainError:
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeStartOwner
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainErrorKind
    edge_index i32
    endpoint_error Option GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError
    span_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    span Option GuiSfntSimpleGlyphContourSpan
    edge_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError
    edge Option GuiSfntSimpleGlyphContourEdge
    edge_slot Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeSlot
    scalar_value Option i32
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainTerminal:
    PathCommandTagStarted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagStartOwner
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeStartOwner

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_start_owner_drain_to_path_command_tag_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeStartOwner
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainError
```

authority check 後だけ cursor の `next_index`、`start`、`end` を読む。`next_index == end` の場合だけ `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PathCommandTag` で PathCommandTag cursor を開始する。成功した場合は storage を消費して `PathCommandTagStarted PathCommandTagStartOwner` を返す。失敗した場合は `PathCommandTagCursorStartFailed` と lower `StdErrorKind` を持つ owner-preserving error を返す。

`next_index < end` かつ `remaining_steps <= 0` の場合は、endpoint marker read、collection source、mutation を行わず `StepBudgetExhausted EdgeStartOwner` を返す。

`next_index < end` かつ `remaining_steps > 0` の場合、global edge index は `next_index - start` である。F5at は private helper で `field::get_ref owner "storage"` から storage を borrow し、`gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker storage edge_index` を呼ぶ。この helper は owner を消費しない。endpoint marker failure は lower `GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError` と current owner を保持した `EndpointMarkerReadFailed` とする。

endpoint marker success 後は marker payload が forged でないことを再検査する。marker glyph raw id は summary capacity glyph raw id と一致しなければならない。marker index は `edge_index` と一致しなければならない。失敗した場合は `EndpointMarkerGlyphMismatch`、`EndpointMarkerIndexMismatch` として typed error を返し、collection source へ進まない。

collection source は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span collection contour_index` と `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge collection contour_index contour_edge_index` だけである。curve segment source は F5at では呼ばない。

span success 後は次を検査する。

```text
span glyph == capacity glyph
span index == contour_index
0 <= span.start_point_index
span.start_point_index <= span.end_point_index
span.end_point_index < capacity.point_count
span.point_count == span.end_point_index - span.start_point_index + 1
span.start_point_index <= global_edge_index <= span.end_point_index
```

これらが成り立つ場合だけ `contour_edge_index = global_edge_index - span.start_point_index` を導出する。contour edge success 後は edge source の contour、local edge index、absolute start point index、wrap 後 next local index を再検査し、成功した場合だけ `EdgeSlot` を作る。

Edge push は internal push helper だけが呼ぶ。helper は summary と cursor を borrow-copy してから storage を消費し、`gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage cursor scalar_value` を 1 回だけ呼ぶ。Edge push failure では lower metadata を storage 回収より前に読む。

```text
read gui_sfnt_simple_glyph_outline_region_push_error_kind &push_error
read gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &push_error
read gui_sfnt_simple_glyph_outline_region_push_error_push_error_kind &push_error
consume gui_sfnt_simple_glyph_outline_region_push_error_storage push_error
```

returned storage と保存済み summary / cursor から `EdgeStartOwner` を復元し、`EdgePushFailed` を返す。push success は F5d returned storage、returned cursor だけから次の EdgeStartOwner を作り、`remaining_steps - 1` で drain を継続する。

`PathCommandTagStartOwner`、`EdgeDrainError`、`EdgeDrainTerminal` は owner を含むため `Clone` / `Copy` を実装しない。`EdgeSlot` と `EdgeDrainErrorKind` は value-only なので `Clone` / `Copy` を実装してよい。

F5at は次を直接呼ばない。

```text
F4 byte-backed lookup helper
byte-backed PointX read / read-push helper
byte-backed PointY read / read-push helper
F5al / F5ak / F5aj traversal helper
lower collection path event / contour step helpers
path sink traversal / real sink mutation
curve segment source
path command tag population
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path sink action PathCommandTag drain

F5au は F5at の `PathCommandTagStartOwner` を authority として、owner storage の Edge owner scalar と collection-backed path sink event kind source から PathCommandTag region の scalar slot を bounded drain し、全 PathCommandTag slot 完了後だけ complete owner へ進む owner-recovery boundary である。F5au は byte-backed lookup、古い path sink traversal、renderer、platform API、font fallback へ戻らない。

PathCommandTag region の slot contract は次である。

```text
logical path command index:
    cursor.next_index - cursor.start

edge index:
    logical path command index / 2

event slot:
    logical path command index % 2
    0 => First
    1 => Second

stored scalar value:
    MoveTo        1
    LineTo        2
    QuadraticTo   3
    SkipNoSegment 4
```

`SkipNoSegment` の reason は scalar へ保存しない。後続の path command value / stream boundary は同じ collection-backed event kind source から reason を再導出する。これにより scalar region は描画 command の payload storage ではなく、command value construction の前段階で使う stable tag storage に留まる。

`PathCommandTagStartOwner` は public constructor を持つため、F5au は owner を消費する前に authority を固定順で検査する。

```text
authority check order:
    summary capacity == owner storage capacity
    cursor well formed
    cursor region == PathCommandTag
    cursor matches summary capacity PathCommandTag region
    collection capacity == summary capacity
```

この順序より前に Edge owner scalar read、collection contour span / event kind source、PathCommandTag push、complete owner transition、storage consume を行ってはいけない。`cursor matches summary capacity PathCommandTag region` は cursor の region / start / end が capacity と一致することを意味し、`next_index` を PathCommandTag region start に固定しない。`StepBudgetExhausted PathCommandTagStartOwner` からの partial drain 再開を拒否してはいけない。

```text
GuiSfntSimpleGlyphPathCommandTag:
    MoveTo
    LineTo
    QuadraticTo
    SkipNoSegment

GuiSfntSimpleGlyphOutlineEdgeOwnerMarker:
    glyph GuiGlyphId
    edge_index i32
    contour_index i32

GuiSfntSimpleGlyphOutlineEdgeOwnerReadErrorKind:
    StorageCapacityInvalid
    ScalarSlotCountMismatch
    ScalarStorageCapacityMismatch
    EdgeIndexOutOfRange
    EdgeOwnerNotReady
    EdgeOwnerSlotMissing
    EdgeOwnerContourOutOfRange

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagSlot:
    path_command_index i32
    edge_index i32
    contour_index i32
    contour_edge_index i32
    event_slot GuiSfntSimpleGlyphPathSinkEventSlot
    tag GuiSfntSimpleGlyphPathCommandTag

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagCompleteOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    PathCommandIndexInvalid
    EventSlotOrdinalInvalid
    EdgeOwnerReadFailed
    EdgeOwnerGlyphMismatch
    EdgeOwnerIndexMismatch
    ContourSpanSourceFailed
    ContourSpanInvariantMismatch
    EventKindSourceFailed
    TagPushFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainTerminal:
    PathCommandTagCompleted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagCompleteOwner
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagStartOwner

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_start_owner_drain_to_complete_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagStartOwner
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainError
```

authority check 後だけ cursor の `next_index`、`start`、`end` を読む。`next_index == end` の場合だけ storage を消費して `PathCommandTagCompleted PathCommandTagCompleteOwner` を返す。`next_index < end` かつ `remaining_steps <= 0` の場合は、Edge owner scalar read、collection source、mutation を行わず `StepBudgetExhausted PathCommandTagStartOwner` を返す。

`next_index < end` かつ `remaining_steps > 0` の場合、logical path command index は `next_index - start` である。F5au は absolute cursor `next_index` を command index として使わない。logical index から `edge_index = div_s path_command_index 2` と `event_slot_ordinal = rem_s path_command_index 2` を導出し、ordinal は `0 => First`、`1 => Second` だけを許す。

F5au は private helper で `field::get_ref owner "storage"` から storage を borrow し、`gui_sfnt_simple_glyph_outline_storage_read_edge_owner storage edge_index` を呼ぶ。この helper は owner を消費しない。read helper は storage capacity、scalar slot count、scalar storage capacity、edge index range、Edge slot presence、stored contour index range を検査する。edge owner success 後も marker glyph raw id と edge index が summary capacity / requested edge と一致することを再検査する。

collection source は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span collection contour_index` と `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at collection contour_index contour_edge_index event_slot` だけである。old sink traversal や byte-backed lookup は呼ばない。

span success 後は次を検査する。

```text
span glyph == capacity glyph
span index == contour_index
0 <= span.start_point_index
span.start_point_index <= span.end_point_index
span.end_point_index < capacity.point_count
span.point_count == span.end_point_index - span.start_point_index + 1
span.start_point_index <= global edge_index <= span.end_point_index
```

これらが成り立つ場合だけ `contour_edge_index = edge_index - span.start_point_index` を導出し、event kind source へ進む。event kind success 後は `GuiSfntSimpleGlyphPathCommandTag` へ写し、`gui_sfnt_simple_glyph_path_command_tag_scalar_value` で stable scalar に変換する。

PathCommandTag push は internal push helper だけが呼ぶ。helper は summary と cursor を borrow-copy してから storage を消費し、`gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage cursor scalar_value` を 1 回だけ呼ぶ。Tag push failure では lower metadata を storage 回収より前に読む。

```text
read gui_sfnt_simple_glyph_outline_region_push_error_kind &push_error
read gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &push_error
read gui_sfnt_simple_glyph_outline_region_push_error_push_error_kind &push_error
consume gui_sfnt_simple_glyph_outline_region_push_error_storage push_error
```

returned storage と保存済み summary / cursor から `PathCommandTagStartOwner` を復元し、`TagPushFailed` を返す。push success は F5d returned storage、returned cursor だけから次の PathCommandTagStartOwner を作り、`remaining_steps - 1` で drain を継続する。

`PathCommandTagCompleteOwner`、`PathCommandTagDrainError`、`PathCommandTagDrainTerminal` は owner を含むため `Clone` / `Copy` を実装しない。`PathCommandTagSlot`、`PathCommandTagDrainErrorKind`、`GuiSfntSimpleGlyphPathCommandTag`、`GuiSfntSimpleGlyphOutlineEdgeOwnerMarker`、`GuiSfntSimpleGlyphOutlineEdgeOwnerReadError` は value-only なので `Clone` / `Copy` を実装してよい。

F5au は次を直接呼ばない。

```text
F4 byte-backed lookup helper
byte-backed PointX / PointY read helper
F5al / F5ak / F5aj traversal helper
old path sink action consumer / traversal helper
path command pair construction
path command stream construction
path sink mutation
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path command value lookup

F5av は F5au の `PathCommandTagCompleteOwner` を authority として、PathCommandTag scalar と collection-backed path sink event source を照合し、1 logical path command index に対応する `GuiSfntSimpleGlyphPathCommand` payload を read-only に返す境界である。これは path command stream construction ではなく、raster / render / platform API へも進まない。

F5av の最小 contract は次である。

```text
input:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagCompleteOwner
    path_command_index i32

output:
    Result PathCommandValue PathCommandValueError
```

`PathCommandTagCompleteOwner` は storage owner を含むが、F5av public lookup は owner を borrow するだけである。成功時も失敗時も storage は消費されない。storage mutation、`Vec` allocation、sink mutation、full stream construction は行わない。

authority check order は次である。

```text
authority check order:
    summary capacity from complete owner
    owner storage capacity without consuming complete owner
    summary capacity == owner storage capacity
    collection capacity == summary capacity
    0 <= path_command_index < capacity.path_command_count
```

authority check が終わる前に PathCommandTag scalar、Edge owner scalar、collection span、source event を読んではいけない。

logical mapping は F5au と同じである。

```text
edge_index = div_s path_command_index 2
event_slot_ordinal = rem_s path_command_index 2
0 => First
1 => Second
```

storage-level PathCommandTag read helper は次の typed error を返す。

```text
GuiSfntSimpleGlyphOutlinePathCommandTagReadErrorKind:
    StorageCapacityInvalid
    ScalarSlotCountMismatch
    ScalarStorageCapacityMismatch
    PathCommandIndexOutOfRange
    PathCommandTagNotReady
    PathCommandTagSlotMissing
    PathCommandTagScalarUnknown
```

unknown scalar は `MoveTo` などへ推測変換しない。`PathCommandTagScalarUnknown` と observed `Option i32` scalar を返す。

F5av の value / error は次である。

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValue:
    path_command_index i32
    edge_index i32
    contour_index i32
    contour_edge_index i32
    event_slot GuiSfntSimpleGlyphPathSinkEventSlot
    stored_tag GuiSfntSimpleGlyphPathCommandTag
    source_tag GuiSfntSimpleGlyphPathCommandTag
    command GuiSfntSimpleGlyphPathCommand

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValueErrorKind:
    StorageSummaryCapacityMismatch
    CollectionSummaryCapacityMismatch
    PathCommandIndexInvalid
    EventSlotOrdinalInvalid
    PathCommandTagReadFailed
    EdgeOwnerReadFailed
    EdgeOwnerGlyphMismatch
    EdgeOwnerIndexMismatch
    ContourSpanSourceFailed
    ContourSpanInvariantMismatch
    EventSourceFailed
    TagMismatch
```

`PathCommandValueError` は owner を含まない value-only error でよい。これは public lookup が owner を消費しないためである。error は storage tag read error、stored tag、Edge owner read error、Edge owner marker、span error、span、event error、source event、source tag を `Option` で保持し、fallback や string parsing なしに失敗地点を `match` できる形にする。

payload 復元は次の順序で行う。

```text
read stored PathCommandTag scalar from complete owner storage
read Edge owner scalar from complete owner storage
validate edge owner glyph == capacity glyph
validate edge owner index == edge_index
read collection contour span for edge owner contour_index
validate span glyph/index/range/count and edge containment
contour_edge_index = edge_index - span.start_point_index
read collection path sink event at contour_index contour_edge_index event_slot exactly once
derive source tag from source event
require stored tag == source tag
return command payload from source event
```

`SkipNoSegment` reason は PathCommandTag scalar からは得られないため、必ず source event payload から再導出する。stored tag と source tag が一致しない場合は `TagMismatch` を返し、別の tag や no-op へ fallback してはいけない。

F5av は次を直接呼ばない。

```text
F4 byte-backed lookup helper
metadata parser
table helper
old path sink action consumer / traversal helper
path command stream construction
storage mutation
Vec allocation / push
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path command stream cursor

F5aw は F5av の `PathCommandValue` lookup を順序付きに読むための bounded cursor / stream preparation 境界である。これは full stream object construction ではなく、`Vec` に command を蓄積しない。raster / render / platform API にも進まない。

F5aw の最小 contract は次である。

```text
cursor create input:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    start_index i32

cursor create output:
    Result PathCommandStreamCursor PathCommandStreamCursorError

step input:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    cursor PathCommandStreamCursor

step output:
    Result PathCommandStreamStep PathCommandStreamStepError

drain input:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    cursor PathCommandStreamCursor
    remaining_steps i32

drain output:
    Result PathCommandStreamDrainTerminal PathCommandStreamStepError
```

`PathCommandStreamCursor` は value-only であり、storage owner を含まない。

```text
PathCommandStreamCursor:
    next_index i32
    end_index i32
```

cursor create の authority check order は次である。

```text
summary capacity from complete owner
owner storage capacity without consuming complete owner
summary capacity == owner storage capacity
collection capacity == summary capacity
capacity shape is valid
0 <= start_index <= capacity.path_command_count
end_index = capacity.path_command_count
```

`start_index == path_command_count` は完了済み cursor として許可する。これは empty stream fallback ではなく、既に全 command を読んだ位置を表す。capacity shape 自体は既存 F5a contract に従い、`point_count <= 0` や `path_command_count != point_count * 2` は typed error にする。

step は explicit enum を返す。

```text
PathCommandStreamStep:
    Emitted PathCommandValue PathCommandStreamCursor
    Completed PathCommandStreamCursor
```

`Completed` は dummy `PathCommandValue` を持たない。step の順序は次である。

```text
validate collection / owner / cursor authority
if cursor.next_index >= cursor.end_index:
    return Completed cursor
else:
    call F5av PathCommandValue lookup exactly once with cursor.next_index
    return Emitted value advanced_cursor
```

bounded drain は explicit terminal を返す。

```text
PathCommandStreamDrainTerminal:
    Completed PathCommandStreamCursor emitted_count
    StepBudgetExhausted PathCommandStreamCursor emitted_count
```

drain は `remaining_steps <= 0` の場合、step helper も F5av lookup も呼ばず `StepBudgetExhausted cursor 0` を返す。budget がある場合は F5aw step helper だけを呼ぶ。drain から F5av lookup を直接呼んではいけない。

F5aw は次を直接呼ばない。

```text
F4 byte-backed lookup helper
metadata parser
table helper
old path sink action consumer / traversal helper
F5av lookup from drain function
storage mutation
Vec allocation / push
path object materialization
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path command stream prepare

F5ax は F5aw の `PathCommandStreamStep` を authority として、path command stream を後続 command sink / raster mask / render2d command emission へ渡す前の prepare summary に畳む境界である。これは real sink ではなく、path object construction でもない。`Vec` に command を蓄積せず、raster / render / platform API にも進まない。

F5ax の最小 contract は次である。

```text
prepare summary:
    total_count i32
    move_to_count i32
    line_to_count i32
    quadratic_to_count i32
    skip_no_segment_count i32
    last_path_command_index i32

prepare step input:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    summary PathCommandStreamPrepareSummary
    cursor PathCommandStreamCursor

prepare step output:
    Result PathCommandStreamPrepareStep PathCommandStreamPrepareStepError

prepare drain input:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    summary PathCommandStreamPrepareSummary
    cursor PathCommandStreamCursor
    remaining_steps i32

prepare drain output:
    Result PathCommandStreamPrepareDrainTerminal PathCommandStreamPrepareStepError
```

`PathCommandStreamPrepareSummary` は value-only であり、storage owner や collection reference を含まない。initial summary はすべての count を `0`、`last_path_command_index` を `-1` とする。

1 command を summary へ反映する action は explicit enum とする。

```text
PathCommandStreamPrepareAction:
    CountedMoveTo
    CountedLineTo
    CountedQuadraticTo
    CountedSkipNoSegment
```

summary update は `PathCommandValue` の command payload を 1 回だけ読み、`GuiSfntSimpleGlyphPathCommand` を `match` して、次のうち 1 つだけを increment する。

```text
MoveTo        -> move_to_count
LineTo        -> line_to_count
QuadraticTo   -> quadratic_to_count
SkipNoSegment -> skip_no_segment_count
```

同時に `total_count` を 1 増やし、`last_path_command_index` を `PathCommandValue.path_command_index` に更新する。`PathCommandValue` の field を直接読む範囲を広げず、public accessor を通す。

prepare step は explicit enum を返す。

```text
PathCommandStreamPrepareStep:
    Prepared PathCommandStreamPrepareAction PathCommandStreamPrepareSummary PathCommandStreamCursor
    Completed PathCommandStreamPrepareSummary PathCommandStreamCursor
```

`Completed` は dummy `PathCommandValue` や dummy action を持たない。step の順序は次である。

```text
call F5aw PathCommandStreamStep exactly once
if F5aw returns Err:
    return PrepareStepError with current summary and cursor
if F5aw returns Completed cursor:
    return Completed summary cursor
if F5aw returns Emitted value next_cursor:
    update summary from value
    return Prepared action updated_summary next_cursor
```

F5ax step は F5av lookup を直接呼ばない。command acquisition は必ず F5aw step helper だけを通す。

bounded prepare drain は explicit terminal を返す。

```text
PathCommandStreamPrepareDrainTerminal:
    Completed PathCommandStreamPrepareSummary PathCommandStreamCursor emitted_count
    StepBudgetExhausted PathCommandStreamPrepareSummary PathCommandStreamCursor emitted_count
```

prepare drain は `remaining_steps <= 0` の場合、prepare step helper も F5aw step helper も呼ばず `StepBudgetExhausted summary cursor 0` を返す。budget がある場合は F5ax prepare step helper だけを呼ぶ。drain から F5aw step helper や F5av lookup を直接呼んではいけない。

F5ax は次を直接呼ばない。

```text
F4 byte-backed lookup helper
metadata parser
table helper
old path sink action consumer / traversal helper
F5av lookup
F5aw step helper from prepare drain
storage mutation
Vec allocation / push
path object materialization
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path command stream sink plan

F5ay は F5ax の completed prepare drain terminal を authority として、後続の explicit command sink / raster mask writer が必要とする容量だけを value-only plan として固定する境界である。これは real sink、path object construction、rasterization、render2d command emission ではない。

F5ay の入力は `PathCommandStreamPrepareSummary` 単体ではない。`StepBudgetExhausted` の partial summary が completed summary と同じ count shape を持つことがあるため、summary 単体を受け取る API は final sink / raster capacity plan の authority にならない。F5ay は必ず F5ax の drain terminal を受け取る。

```text
path command stream sink plan input:
    PathCommandStreamPrepareDrainTerminal

success authority:
    PrepareDrainTerminal.Completed summary cursor emitted_count

rejected terminal:
    PrepareDrainTerminal.StepBudgetExhausted summary cursor emitted_count
```

`StepBudgetExhausted` は `PrepareNotCompleted` error になる。silent no-op や partial plan へ変換してはいけない。

F5ay の plan は次を保持する。

```text
PathCommandStreamSinkPlan:
    total_count i32
    emitted_count i32
    draw_count i32
    move_to_count i32
    line_to_count i32
    quadratic_to_count i32
    skip_no_segment_count i32
    path_segment_capacity i32
    raster_edge_capacity i32
    last_path_command_index i32
```

capacity derivation:

```text
draw_count = line_to_count + quadratic_to_count
path_segment_capacity = move_to_count + line_to_count + quadratic_to_count
raster_edge_capacity = line_to_count + quadratic_to_count
prepared_count = path_segment_capacity + skip_no_segment_count
```

`SkipNoSegment` は source command として count するが、actual path segment capacity と raster edge capacity には入れない。これは no segment command が後続 mask edge を生成しないためである。

検査順は次である。

```text
1. terminal が Completed であることを確認する
2. total_count / move_to_count / line_to_count / quadratic_to_count / skip_no_segment_count / emitted_count が非負であることを確認する
3. total_count > 0 であることを確認する
4. last_path_command_index >= 0 であることを確認する
5. move + line、move + line + quadratic、line + quadratic、prepared_count を overflow guard 付きで計算する
6. prepared_count == total_count を確認する
7. emitted_count == total_count を確認する
8. draw_count と raster_edge_capacity の一致を確認する
```

overflow guard は `2147483647 - left` を先に計算し、`right` が残余を超える場合は `CountOverflow` を返す。raw `i32` addition の wraparound に依存してはいけない。

F5ay error は enum と typed context で表す。

```text
PathCommandStreamSinkPlanErrorKind:
    PrepareNotCompleted
    NegativeTotalCount
    NegativeMoveToCount
    NegativeLineToCount
    NegativeQuadraticToCount
    NegativeSkipNoSegmentCount
    NegativeEmittedCount
    NoCommandsPrepared
    LastPathCommandIndexInvalid
    CountOverflow
    PreparedCountMismatch
    EmittedCountMismatch
    DrawCountMismatch
```

F5ay は次を直接呼ばない。

```text
F5ax prepare drain
F5ax prepare step
F5aw step helper
F5av lookup
F4 byte-backed lookup helper
metadata parser
table helper
old path sink action consumer / traversal helper
storage mutation
Vec allocation / push
path object materialization
render / raster / platform / host APIs
font fallback
```

### SFNT simple glyph outline point stream item collection path command stream sink owner

F5az は F5ay の completed path command stream sink plan を authority として、後続 explicit command sink writer と raster mask writer が使う scalar storage owner を確保する境界である。これは writer 本体、raster mask writer、rasterization、render2d command emission、platform present ではない。

F5az の入力は `PathCommandStreamSinkPlan` である。ただし `SinkPlan` は public value なので、F5az はそれを trusted value としてそのまま使わない。全 count、capacity、derived invariant を再検査してから allocation へ進む。

```text
path command stream sink owner input:
    PathCommandStreamSinkPlan

success:
    SinkOwner plan capacity path_sink_scalars raster_mask_scalars

failure:
    SinkOwnerAllocError kind plan capacity storage_error
```

F5az の capacity は次を保持する。

```text
PathCommandStreamSinkOwnerCapacity:
    path_sink_scalar_capacity i32
    raster_mask_scalar_capacity i32
    path_segment_capacity i32
    raster_edge_capacity i32
```

scalar capacity derivation:

```text
path_sink_scalar_capacity =
    move_to_count * 3
    + line_to_count * 3
    + quadratic_to_count * 5

raster_mask_scalar_capacity =
    line_to_count * 5
    + quadratic_to_count * 7
```

`SkipNoSegment` は source command として `total_count` / `prepared_count` に入るが、path sink scalar capacity と raster mask scalar capacity には入らない。`SkipNoSegment` だけで構成された completed plan は、`path_sink_scalar_capacity = 0` かつ `raster_mask_scalar_capacity = 0` の valid owner allocation として扱う。これを silent no-op や `NoCommandsPrepared` に変換してはいけない。

検査順は次である。

```text
1. total_count / emitted_count / draw_count / move_to_count / line_to_count / quadratic_to_count / skip_no_segment_count / path_segment_capacity / raster_edge_capacity が非負であることを確認する
2. total_count > 0 であることを確認する
3. last_path_command_index >= 0 であることを確認する
4. move + line + quadratic を checked add で計算し、path_segment_capacity と一致することを確認する
5. path_segment_capacity + skip_no_segment_count を checked add で計算し、total_count と一致することを確認する
6. emitted_count == total_count を確認する
7. line + quadratic を checked add で計算し、raster_edge_capacity と一致することを確認する
8. draw_count == raster_edge_capacity を確認する
9. path sink scalar capacity と raster mask scalar capacity を checked multiply / checked add で計算する
10. path sink scalar Vec を確保する
11. raster mask scalar Vec を確保する
```

overflow guard は addition と multiplication の両方に必要である。

```text
checked add:
    remaining = 2147483647 - left
    if right > remaining:
        CountOverflow

checked multiply:
    max_factor_count = 2147483647 / factor
    if count > max_factor_count:
        CountOverflow
```

F5az error は enum と typed context で表す。coarse `InvalidPlan` は使わない。

```text
PathCommandStreamSinkOwnerAllocErrorKind:
    NegativeTotalCount
    NegativeEmittedCount
    NegativeMoveToCount
    NegativeLineToCount
    NegativeQuadraticToCount
    NegativeSkipNoSegmentCount
    NegativePathSegmentCapacity
    NegativeRasterEdgeCapacity
    NegativeDrawCount
    LastPathCommandIndexInvalid
    NoCommandsPrepared
    PathSegmentCapacityMismatch
    RasterEdgeCapacityMismatch
    PreparedCountMismatch
    EmittedCountMismatch
    DrawCountMismatch
    CountOverflow
    PathSinkScalarStorageAllocFailed
    RasterMaskScalarStorageAllocFailed
```

```text
PathCommandStreamSinkOwnerAllocError:
    kind PathCommandStreamSinkOwnerAllocErrorKind
    plan PathCommandStreamSinkPlan
    capacity Option PathCommandStreamSinkOwnerCapacity
    storage_error Option StdErrorKind
```

validation / overflow failure では `capacity = None`、`storage_error = None` とする。Vec allocation failure では `capacity = Some derived_capacity`、`storage_error = Some lower_std_error` とする。

raster mask scalar Vec の allocation に失敗した場合、すでに得た path sink scalar Vec owner を必ず 1 回だけ `vec::free` してから error を返す。path sink scalar Vec の allocation に失敗した場合は、まだ解放すべき owner が存在しないため `vec::free` を呼ばない。

F5az は次を直接呼ばない。

```text
F5ax prepare drain / step
F5aw path command stream step
F5av path command value lookup
F4 byte-backed lookup helper
metadata parser
table helper
old path sink action consumer / traversal helper
Vec push
path object materialization
rasterization
render / platform / host APIs
font fallback
```

### Supported font containers

標準設計は次を対象にする。

- TTF
- OTF
- TTC
- OTC
- variable font を含む OpenType variation tables
- WOFF / WOFF2 は後続 phase の decode container とする

初期実装で decode できない container は `Unsupported` として返す。別 container へ推測変換しない。

### Writing mode

横書き、右横書き、縦書きを同じ text layout model で扱う。

```text
GuiWritingMode:
    HorizontalLtr
    HorizontalRtl
    VerticalRl
    VerticalLr
```

`HorizontalRtl` は Arabic/Hebrew などの右横書きだけでなく、将来の bidi layout と同じ direction contract へ接続する。縦書きでは glyph orientation、advance、ruby placement、line progression が metrics に反映される。

### Metrics

Layout engine が使う metrics は rendering engine と同じ font face、size、variation、writing mode、feature set から生成する。

```text
GuiFontMetrics:
    ascent i32
    descent i32
    line_gap i32
    em_size i32

GuiGlyphMetrics:
    advance_x i32
    advance_y i32
    ink_bounds GuiRect
    allocation_bounds GuiRect

GuiRenderedTextMetrics:
    logical_bounds GuiRect
    ink_bounds GuiRect
    allocation_bounds GuiRect
    baseline i32
```

測定だけ host API、描画だけ別 engine という分離は禁止する。Host-provided mock measurer は explicit test utility であり、formal font renderer の代替ではない。

### Ruby, vertical text, math

Furigana / ruby は通常 text と同じ glyph pipeline を通る inline object とする。Ruby text だけを browser ruby layout や DOM に逃がさない。

数式描画は将来の math layout library が `GuiMathInlineBox` 相当の inline object を返し、font renderer と 2D renderer は glyph metrics、path、mask、baseline、ink bounds を共有する。

### Paint integration

Text は fill-only、stroke-only、fill+stroke、shadow をサポートする。Paint / stroke / shadow / blend mode は 2D renderer と共有し、text 専用 color model は作らない。

```text
GuiGlyphPaint:
    fill Option GuiPaint
    stroke Option GuiStroke
    shadows GuiShadowRef
    blend GuiBlendMode
```

fill と stroke が両方 `None` の描画 command は invalid である。透明 fallback や no-op にはしない。

`GuiShadowRef` は no_alloc core と alloc-backed multi-shadow の接点である。

```text
GuiShadowRef:
    NoShadow
    SingleShadow GuiShadow
    ShadowRun GuiShadowRunId

GuiShadowRunId:
    raw i32
```

`core/gui` の F1 実装は `NoShadow` と `SingleShadow` を O(1) value として扱う。複数 shadow は `alloc/gui/render2d` が owns する `Vec GuiShadow` を `GuiShadowRunId` で参照する。したがって high-level design の `shadows Vec Shadow` と no_alloc core の `GuiShadowRef` は矛盾しない。

## Error contract

F1/F2 は既存の `GuiError` を返すが、font-specific error category を public enum として同時に定義する。

```text
GuiFontErrorKind:
    InvalidFontSize
    InvalidFaceIndex
    FaceIndexRequired
    MissingFontResource
    UnsupportedFontContainer
    MissingGlyph
    InvalidGlyphPaint
```

最小 slice では `GuiFontErrorKind` を diagnostic/display に使うための data value とし、関数の戻り値は既存 `Result T GuiError` に揃える。詳細 error payload を `GuiError` へ接続する phase までは、対応は次で固定する。

- invalid request shape: `GuiError::InvalidCommand`
- invalid geometry or metrics bounds: `GuiError::InvalidGeometry`
- unsupported container / writing mode / feature: `GuiError::Unsupported`
- missing resource or missing glyph: `GuiError::Unsupported`
- capacity exhaustion: `GuiError::ResourceExhausted`

この対応は error display ではない。表示文言は `std/gui/error_display` または後続の font error display layer に置く。

## Current implementation

現在の stdlib には fixed-cell `MockTextMeasurer` と `HostTextMeasurer` wrapper がある。これは layout test、terminal text-grid、legacy smoke 用の deterministic utility であり、本格 font renderer ではない。

Formal font renderer は `core/gui/font` と `std/gui/font_resource` を通る。F1/F2 以降の font renderer 実装は `MockTextMeasurer`、`host_text_measurer`、`host_text_measurer_fixed` を fallback として import または呼び出してはならない。Fixed-cell 測定を使う場合は test utility または terminal backend として明示する。

次の実装 slice では、まず `core/gui/font` と `core/gui/render_style` に no_alloc contract を追加し、`std/gui/font_resource` に resource request boundary を追加する。TTF parsing や outline rasterization はこの slice では行わない。

## 非 goal

- Browser の `CanvasRenderingContext2D.fillText` を正式 renderer にすること。
- DOM / SVG text layout を標準 API に露出すること。
- OS font fallback を暗黙使用すること。
- Headless で fixed-cell fallback に暗黙切替すること。
- Missing glyph を別 glyph や tofu に暗黙置換すること。
