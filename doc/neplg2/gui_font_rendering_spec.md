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
