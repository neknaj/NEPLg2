# NEPLg2 GUI font rendering implementation plan

作成日: 2026-06-13

## 実装開始 gate

実装前に次を満たす。

1. `gui_font_rendering_spec.md`、`gui_font_rendering_detailed_design.md`、`gui_font_rendering_implementation_plan.md` が存在する。
2. `gui_font_rendering_design.md` と `gui_2d_rendering_design.md` の方針と矛盾しない。
3. Zenn 方針に照らして、platform abstraction、Option / Result、enum / match、fallback 禁止、契約と現状実装の分離が満たされている。
4. subagent が文書を確認し、`implementation may start` 相当の結論を返す。
5. blocker / required 指摘がある場合は doc を修正し、再 review する。

## Phase F1: core font and render style contract

目的:

- Font renderer と 2D renderer の共有 contract を no_alloc value として追加する。
- 本格 TTF parser の前に、layout / renderer が依存できる型境界を固定する。

変更:

- `stdlib/core/gui/font.nepl` を追加する。
- `GuiFontFaceId`、`GuiGlyphId`、`GuiFontSize`、`GuiWritingMode`、`GuiFontMetrics`、`GuiGlyphMetrics`、`GuiRenderedTextMetrics` を追加する。
- `GuiFontErrorKind`、`GuiShadowRunId`、`GuiShadowRef` を追加する。
- `gui_font_size_result` は denominator 0 以下を `GuiError::InvalidCommand` として返す。
- `stdlib/core/gui/render_style.nepl` を追加する。
- `GuiBlendMode`、`GuiShadow`、`GuiGlyphPaint` を追加する。
- `GuiGlyphPaint` は `shadows GuiShadowRef` を持ち、alloc-backed multi-shadow は `GuiShadowRef::ShadowRun` で参照する。
- `gui_glyph_paint_result` は fill と stroke が両方 `None` の場合 `GuiError::InvalidCommand` を返す。
- `core/gui/prelude.nepl` から font / render_style を公開する。
- `tests/stdlib/gui_core.n.md` に doctest を追加する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_core.n.md --no-tree -o tmp_gui_core_font.json -j 1
node nodesrc/test_stdlib_gui_layering_policy.js
git diff --check
```

Subagent review:

- 実装前に文書レビューを受ける。
- 実装後に core が alloc/std/platform を import していないこと、fallback sentinel がないこと、invalid case が Result で返ることを確認させる。

## Phase F2: std font resource boundary

目的:

- Font bytes loading を app-facing raw path 文字列ではなく typed request として std layer に置く。
- Web VFS / native filesystem / bare embedded blob の差を `std/gui` と platform provider の境界へ押し出す。

変更:

- `stdlib/std/gui/font_resource.nepl` を追加する。
- `GuiFontDecodePolicy`、`GuiFontResourceSource`、`GuiFontResourcePath`、`GuiResourceHash`、`GuiFontResourceRequest` を追加する。
- `gui_font_resource_request` は typed path、face index、expected hash、decode policy を保持する。
- F2 は request shape だけを検査する。`face_index` が `Some n` で `n < 0` の場合は `GuiError::InvalidCommand` とする。Collection font の `face_count` が必要な検査は F4 へ送る。
- `std/gui.nepl` facade から公開する。
- `tests/stdlib/gui_std.n.md` に doctest を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` を追加し、標準 API に DOM / Canvas / FontFace / CoreText / DirectWrite / fontconfig handle が入らないことを固定する。
- 同 source policy で formal font renderer / font resource contract が `MockTextMeasurer`、`HostTextMeasurer`、`host_text_measurer_fixed` に依存しないことを固定する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_std.n.md --no-tree -o tmp_gui_std_font.json -j 1
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/run_source_policy_regressions.js --warn-only
git diff --check
```

## Phase F3: bundled HackGen fixture routing

目的:

- `web/src/fonts/HackGenConsoleNF-Regular.ttf` を formal fixture として `fonts/HackGenConsoleNF-Regular.ttf` に mapping する。
- HackGen 専用 API を作らず、任意 font resource を登録できる経路を保つ。

変更:

- Web VFS manifest に canonical resource path `fonts/HackGenConsoleNF-Regular.ttf` と `fonts/HackGen-LICENSE.txt` を登録する。
- Web VFS 内部 path は `/fonts/...` とし、canonical path とは別の transport 表現として扱う。
- `web/src/gui-font/font-resource-vfs.ts` を追加し、bundled resource manifest、path normalization、VFS mount、typed mount error を持たせる。
- Web Playground startup で mount promise を開始し、`neplg2 run` の直前に完了を待つ。失敗時は typed error を terminal に表示して実行を開始しない。
- Compile-only path は runtime font bytes を要求しないため mount を待たない。
- Native resource root の探索 contract を doc と source policy に追加する。
- Bare は embedded blob provider が未設定なら unsupported を返す contract にする。
- Source policy で、HackGen 専用 API、suffix match、silent success、binary/read-only file の compile overlay 混入を禁止する。

検証:

```powershell
npm --prefix web run build:ts
node nodesrc/test_web_gui_font_rendering_contract.js
git diff --check
```

完了条件:

- `web/src/fonts/HackGenConsoleNF-Regular.ttf` が `/fonts/HackGenConsoleNF-Regular.ttf` として VFS に read-only mount される。
- `VFS.serializeForCompile()` が font binary と read-only license text を compiler overlay へ含めない。
- `FetchUnavailable`、`InvalidResourcePath`、`NetworkError`、`HttpError`、`InvalidBytes`、`InvalidText`、`VfsWriteFailed` のいずれも typed error として扱われる。
- Native / Bare / Headless の resource provider contract が doc と source policy で検査される。

## Phase F4a: sfnt directory and numeric metrics parser

目的:

- TTF / OTF / TTC / OTC の table directory と numeric basic metrics を decode する。

変更:

- `alloc/gui/font/sfnt.nepl` と basic table parser を追加する。
- Invalid table directory、invalid table offset、unsupported container、collection face index error を typed error として扱う。
- 未解析の extra table は error にせず無視する。error にするのは unsupported container、必須 numeric table の欠落、範囲外 offset、face selection の不整合だけである。
- Headless/offscreen tests が explicit fixture bytes を使えるようにする。

完了条件:

- explicit fixture bytes から container kind、face count、face index、units per em、ascent、descent、line gap、num glyphs を取得できる。
- Missing `head` / `hhea` / `maxp` や invalid face index は代替成功させず error になる。

## Phase F4b: sfnt name table policy

目的:

- font family、subfamily、full name などの代表値を name table から decode するための encoding policy を固定する。
- name parser を numeric metrics parser から分け、metadata parse の成功条件に name decode を混ぜない。

変更:

- `alloc/gui/font/sfnt.nepl` を facade にし、F4a 実装を `alloc/gui/font/sfnt/metadata.nepl` へ置く。
- `alloc/gui/font/sfnt/name.nepl` を追加する。
- `GuiSfntNameEncodingKind`、`GuiSfntNameRecord`、`GuiSfntNameSelection`、`GuiSfntNames` を追加する。
- name ID 1 / 2 / 4 を family / subfamily / full name として扱う。
- 代表 record の順位は、Windows platform 3 encoding 1 language 0x0409、Windows platform 3 のその他、Macintosh platform 1 encoding 0 language 0、Macintosh platform 1 のその他、の順にする。
- Windows platform 3 encoding 1 language 0x0409 は UTF-16BE ASCII subset として decode する。
- Macintosh platform 1 encoding 0 language 0 は Roman ASCII subset として decode する。
- higher-ranked candidate が未対応 encoding の場合は、lower-ranked candidate へ暗黙に切り替えず `UnsupportedNameEncoding` を返す。
- `name` table 欠落は `MissingTable`、format 0 以外は `UnsupportedNameTableFormat`、record / string range 不正や empty selected string は `MalformedNameRecord`、ASCII subset 外文字は `UnsupportedNameCharacter` とする。
- name ID 1 / 2 / 4 の candidate が存在しない場合、その field は `Option::None` とする。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_parse_names` を呼ばないこと、SFNT parser が platform / host font API / path display-name authority を持たないことを固定する。

完了条件:

- explicit fixture bytes から `Demo` / `Regular` / `Demo Regular` を取得できる。
- `name` table がない fixture は `MissingTable` になる。
- unsupported selected record は `UnsupportedNameEncoding`、UTF-16BE の奇数 byte length は `MalformedNameRecord`、ASCII subset 外文字は `UnsupportedNameCharacter` になる。
- `gui_sfnt_parse_metadata` の existing F4a doctest は name table の有無に依存せず通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
git diff --check
```

## Phase F4c: sfnt cmap glyph lookup

目的:

- Unicode BMP code point から `GuiGlyphId` を取得する最初の `cmap` parser を追加する。
- glyph mapping を host font API、browser text API、path / family name、暗黙置換に依存させない。

変更:

- `alloc/gui/font/sfnt/cmap.nepl` を追加する。
- `alloc/gui/font/sfnt.nepl` facade から metadata、name、cmap を再公開する。
- `GuiSfntDirectory` に optional `cmap` table record を追加し、`gui_sfnt_directory_cmap` を公開する。
- `GuiSfntParseErrorKind` に `UnsupportedCmapEncoding`、`UnsupportedCmapTableFormat`、`MalformedCmapRecord`、`MissingGlyphMapping` を追加する。
- `gui_sfnt_lookup_glyph_id` は `Result GuiGlyphId GuiSfntParseError` を返し、raw `i32` を public glyph id として返さない。
- F4c の subtable selection は platformID 3 / encodingID 1 の最初の record だけを選ぶ。対象 record がなければ `UnsupportedCmapEncoding`、選択 record が format 4 でなければ `UnsupportedCmapTableFormat` とする。
- BMP 外 code point は `UnsupportedCmapEncoding`、BMP 内で segment がない、glyphIdArray entry が 0、computed glyph id が 0 の場合は `MissingGlyphMapping` とする。
- Format 4 の declared table header、encoding record array overlap、`length`、`segCountX2`、`reservedPad`、segment array bounds、idRangeOffset target bounds を検査し、不正なら `MalformedCmapRecord` とする。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_lookup_glyph_id` を呼ばないこと、SFNT facade が `metadata` / `name` / `cmap` を公開すること、`cmap` parser が platform / host font API / 暗黙置換 / path authority を持たないことを固定する。

完了条件:

- explicit fixture bytes から ASCII `A` の glyph id 36 を `GuiGlyphId` として取得できる。
- `cmap` table がない fixture は `MissingTable` になる。
- platformID 3 / encodingID 1 がない fixture は `UnsupportedCmapEncoding` になる。
- selected record が format 4 以外の場合は `UnsupportedCmapTableFormat` になる。
- glyph 0、missing segment、壊れた format 4 array、encoding record array を指す selected subtable offset、短い declared table header は typed error になる。
- unsupported selected record と別の plausible record が同居しても別 record に切り替えない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
git diff --check
```

## Phase F4d: sfnt hmtx horizontal metrics lookup

目的:

- `GuiGlyphId` から horizontal advance width と left side bearing を取得する最初の `hmtx` parser を追加する。
- layout engine が host text measurement や fixed-cell utility に逃げず、font bytes の metrics table を authority として使えるようにする。

変更:

- `alloc/gui/font/sfnt/hmtx.nepl` を追加する。
- `alloc/gui/font/sfnt.nepl` facade から metadata、name、cmap、hmtx を再公開する。
- `GuiSfntDirectory` に optional `hmtx` table record を追加し、`gui_sfnt_directory_hmtx` を公開する。
- `GuiSfntParseErrorKind` に `MalformedHmtxRecord` と `MissingGlyphMetric` を追加する。
- `GuiSfntHorizontalMetric` を追加し、glyph、advance_width、left_side_bearing を typed value として返す。
- `gui_sfnt_lookup_horizontal_metric` は `Result GuiSfntHorizontalMetric GuiSfntParseError` を返す。
- `hhea.numberOfHMetrics` は `hhea.offset + 34` の u16 として読む。このため `hhea.length >= 36` は `hmtx` lookup 専用の要件とし、F4a metadata parser の `hhea.length >= 10` は変更しない。
- `numberOfHMetrics <= 0`、`numberOfHMetrics > maxp.numGlyphs`、`glyphRaw <= 0`、`glyphRaw >= maxp.numGlyphs`、declared `hmtx.length` 不足は typed error とする。
- `hmtx.length` は `numberOfHMetrics * 4 + (numGlyphs - numberOfHMetrics) * 2` 以上でなければならない。file 末尾に余分な byte があっても declared table length を越えて読まない。
- `glyphRaw < numberOfHMetrics` は `longHorMetric[glyphRaw]` を読む。`glyphRaw >= numberOfHMetrics` は最後の longHorMetric の advance width と leftSideBearing array を読む。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_lookup_horizontal_metric` を呼ばないこと、`hmtx` parser が platform / host font API / path authority / fixed-cell fallback / name or cmap 代替を持たないことを固定する。

完了条件:

- explicit fixture bytes から glyph 1 の longHorMetric advance width と left side bearing を取得できる。
- explicit fixture bytes から glyph 3 の last advance width と leftSideBearing array entry を取得できる。
- `hmtx` table がない fixture は `MissingTable` になる。
- `hhea` が `numberOfHMetrics` を読めない fixture、invalid `numberOfHMetrics`、glyph range 外、declared `hmtx.length` 不足は typed error になる。
- `gui_sfnt_parse_metadata` の existing F4a doctest は `hmtx` table の有無に依存せず通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
git diff --check
```

## Phase F4e: sfnt loca/glyf glyph header bounds lookup

目的:

- `GuiGlyphId` から glyph header の x/y bounds を取得する最初の `loca` / `glyf` parser を追加する。
- layout engine が rendered bounds を扱う前段として、host text measurement や fixed-cell utility に逃げず、font bytes の outline table header を authority として使えるようにする。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` を追加する。
- `alloc/gui/font/sfnt.nepl` facade から metadata、name、cmap、hmtx、glyf を再公開する。
- `GuiSfntDirectory` に optional `loca` / `glyf` table record を追加し、`gui_sfnt_directory_head`、`gui_sfnt_directory_loca`、`gui_sfnt_directory_glyf` を公開する。
- `GuiSfntParseErrorKind` に `UnsupportedLocaFormat`、`MalformedGlyfRecord`、`MissingGlyphOutline` を追加する。
- `GuiSfntGlyphBounds` を追加し、glyph、x_min、y_min、x_max、y_max を typed value として返す。
- `gui_sfnt_lookup_glyph_bounds` は `Result GuiSfntGlyphBounds GuiSfntParseError` を返す。
- `head.indexToLocFormat` は `head.offset + 50` の i16 として読む。このため `head.length >= 52` は `glyf` lookup 専用の要件とし、F4a metadata parser の `head.length >= 20` は変更しない。
- `indexToLocFormat == 0` は short loca offset として u16 value を 2 倍する。`indexToLocFormat == 1` は long loca offset として u32 value を読む。u32 value が i32 範囲外なら `MalformedGlyfRecord` とする。
- `indexToLocFormat` が 0 / 1 以外なら `UnsupportedLocaFormat` とする。
- `loca.length` は format 0 で `(numGlyphs + 1) * 2`、format 1 で `(numGlyphs + 1) * 4` 以上でなければならない。file 末尾に余分な byte があっても declared table length を越えて読まない。
- `glyphRaw <= 0`、`glyphRaw >= maxp.numGlyphs`、empty glyph range は `MissingGlyphOutline` とする。
- `start > end`、`end > glyf.length`、glyph header 10 byte 未満、inverted x/y bounds は `MalformedGlyfRecord` とする。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_lookup_glyph_bounds` を呼ばないこと、`glyf` parser が platform / host font API / path authority / fixed-cell fallback / name or cmap or hmtx 代替を持たないことを固定する。

完了条件:

- explicit fixture bytes から glyph 1 の negative x/y min を含む bounds を取得できる。
- format 1 loca fixture から glyph bounds を取得できる。
- `loca` / `glyf` table がない fixture は `MissingTable` になる。
- `head` が `indexToLocFormat` を読めない fixture、unsupported format、long loca high-bit u32 offset、declared `loca.length` 不足、decreasing offset、empty glyph、short glyph header、inverted bounds は typed error になる。
- `gui_sfnt_parse_metadata` の existing F4a doctest は `loca` / `glyf` table の有無に依存せず通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4f: sfnt simple glyph topology lookup

目的:

- full outline / rasterization の前段として、simple glyph の contour endpoint array、instruction length、point data range を typed value として取得する。
- 後続の flags / coordinate decode が host font API や fallback に逃げず、font bytes 内の checked topology から始められるようにする。

変更:

- `GuiSfntParseErrorKind` に `UnsupportedGlyphOutlineFormat` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphTopology` と `gui_sfnt_lookup_simple_glyph_topology` を追加する。
- `GuiSfntSimpleGlyphTopology` は glyph、bounds、contour_count、point_count、instruction_length、point_data_offset、point_data_length を持つ。
- `point_data_offset` は file absolute offset ではなく `glyf` table-relative offset とする。
- `numberOfContours < 0` は composite glyph / unsupported outline format として `UnsupportedGlyphOutlineFormat` を返す。
- `numberOfContours == 0` は renderable outline がないため `MissingGlyphOutline` を返す。
- endpoint array 全体、instructionLength、instructions、point data range は selected glyph range 内に閉じる。
- endpoint は strict increasing とし、`point_count = last_endpoint + 1` とする。overflow や `point_count <= 0` は `MalformedGlyfRecord` とする。
- `numberOfContours > 0` かつ `point_count > 0` で `point_data_length == 0` なら `MalformedGlyfRecord` とする。
- Source policy で simple topology API、typed error、declared range validation、metadata / name / cmap / hmtx / platform API 非依存を固定する。

完了条件:

- explicit fixture bytes から glyph 1 の contour count、point count、instruction length、point data offset、point data length を取得できる。
- composite glyph、zero contour、non-increasing endpoint、short endpoint array、short instruction length、instruction overrun、missing point data は typed error になる。
- F4e の glyph bounds doctest と `glyf.nepl` module doctest は引き続き通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4g: sfnt simple glyph point stream range lookup

目的:

- simple glyph の flags repeat 展開と x/y coordinate byte range を検査する。
- coordinate value や point `Vec` をまだ作らず、後続 decoder が読む raw byte range を typed value として返す。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphPointStream` と `gui_sfnt_lookup_simple_glyph_point_stream` を追加する。
- `GuiSfntSimpleGlyphPointStream` は topology、flag_data_offset、flag_data_length、x_data_offset、x_data_length、y_data_offset、y_data_length、trailing_data_offset、trailing_data_length を持つ。
- すべての offset は `glyf` table-relative とする。
- `flag_data_offset = topology.point_data_offset` とする。
- `flag_data_length` は expanded logical flag count ではなく、repeat count byte を含む raw consumed flag stream length とする。
- repeat flag byte 自身は 1 point 分であり、repeat count byte は追加 point 数である。`repeat_count = 0` は current flag 1 個だけを意味する。
- flags scan はちょうど `point_count` 個の logical flags を満たす。point count に届かない、repeat byte 欠落、repeat run overrun は `MalformedGlyfRecord` とする。
- x/y coordinate byte length は short bit と same bit だけから計算する。short bit が立つ場合、same / positive bit は sign であり byte length には影響しない。
- `x_data_offset = flag_data_offset + flag_data_length`、`y_data_offset = x_data_offset + x_data_length`、`trailing_data_offset = y_data_offset + y_data_length`、`trailing_data_length = glyph_end - trailing_data_offset` とする。
- `trailing_data_length < 0` は `MalformedGlyfRecord`。`trailing_data_length >= 0` は success として明示値で返す。
- Source policy で raw flag length、repeat semantics、x/y length formula、trailing data policy、metadata / name / cmap / hmtx / platform API 非依存を固定する。

完了条件:

- explicit fixture bytes から no-repeat point stream の flag/x/y/trailing ranges を取得できる。
- repeat run を含む fixture で raw flag length と coordinate ranges を取得できる。
- `repeat_count = 0` を current flag 1 個として扱う fixture が成功する。
- short=1、short=0 same=1、short=0 same=0 の x/y byte length 分岐を doctest で固定する。
- repeat overrun、missing repeat byte、x coordinate overrun、y coordinate overrun は typed `MalformedGlyfRecord` になる。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4h: sfnt simple glyph single point decode

目的:

- checked point stream range から 1 logical point の coordinate、on-curve、contour end state を復元する。
- full point `Vec` / outline builder は allocation failure と owner recovery の contract を設計してから後続 phase で実装する。
- F4h は allocation なしで動作し、F4g の range validation を必ず通る。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphPoint` と `gui_sfnt_lookup_simple_glyph_point` を追加する。
- `GuiSfntSimpleGlyphPoint` は glyph、point_index、x、y、on_curve、end_of_contour を持つ。
- `point_index < 0` または `point_index >= topology.point_count` は `MissingGlyphOutline` とする。
- flag / coordinate / endpoint の byte 構造不整合は `MalformedGlyfRecord` とする。
- `gui_sfnt_lookup_simple_glyph_point` は `gui_sfnt_glyf_simple_point_stream_with_tables` を通り、F4g-derived `flag_data` / `x_data` / `y_data` range 内だけを読む。
- flag bit 0 を `on_curve` とする。
- x delta は xShort / xPositive / xSame から `+u8`、`-u8`、`0`、`i16be` に復元する。
- y delta は yShort / yPositive / ySame から `+u8`、`-u8`、`0`、`i16be` に復元する。
- coordinate は point 0 から `point_index` まで累積する。target が repeat run の途中にある場合も、target より前の repeated point の delta は消費・累積する。
- `end_of_contour` は topology から endpoint array offset を復元し、endpoint value と point_index の一致で判定する。
- F4h は `trailing_data_length` を読まず、zero padding も要求しない。
- Source policy で single point API、no Vec allocation、F4g validation reuse、cumulative coordinate semantics、out-of-range error kind、platform / fallback 非依存を固定する。

完了条件:

- no-repeat fixture で point 0 と endpoint point を decode できる。
- repeat run fixture で target が repeat run 内にある場合でも、前の repeated point の delta が累積される。
- signed long coordinate と negative short coordinate を decode できる。
- `repeat_count = 0` fixture で x/y 0、contour end を decode できる。
- `point_index = -1` と `point_index = point_count` は `MissingGlyphOutline` になる。
- coordinate overrun 系 fixture を point lookup 経由でも `MalformedGlyfRecord` として扱える。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4i: sfnt simple glyph contour span lookup

目的:

- checked simple glyph topology から、1 contour の inclusive logical point range を返す。
- full outline `Vec` / curve segment builder / mask rasterizer は allocation failure と owner recovery の contract を設計してから後続 phase で実装する。
- F4i は allocation なしで動作し、F4f の topology validation だけに依存する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphContourSpan` と `gui_sfnt_lookup_simple_glyph_contour_span` を追加する。
- `GuiSfntSimpleGlyphContourSpan` は glyph、contour_index、start_point_index、end_point_index、point_count を持つ。
- `end_point_index` は inclusive endpoint とし、`point_count = end_point_index - start_point_index + 1` とする。
- `contour_index < 0` または `contour_index >= topology.contour_count` は `MissingGlyphOutline` とする。
- endpoint array read failure や F4f topology validation で観測された endpoint 不整合は `MalformedGlyfRecord` とする。
- contour 0 の start は 0、contour n の start は contour n-1 の endpoint + 1 とする。
- `gui_sfnt_lookup_simple_glyph_contour_span` は `gui_sfnt_glyf_simple_topology_with_tables` を通る。
- F4i は `gui_sfnt_glyf_simple_point_stream_with_tables`、`gui_sfnt_lookup_simple_glyph_point_stream`、`gui_sfnt_lookup_simple_glyph_point` を呼ばない。
- Source policy で contour span API、F4f validation reuse、F4g/F4h 非依存、metadata 非依存、no Vec allocation を固定する。

完了条件:

- two-contour fixture の contour 0 が start 0、end 1、point_count 2 を返す。
- two-contour fixture の contour 1 が start 2、end 3、point_count 2 を返す。
- one-contour signed coordinate fixture の contour 0 が start 0、end 2、point_count 3 を返す。
- `contour_index = -1` と `contour_index = contour_count` は `MissingGlyphOutline` になる。
- malformed endpoint fixture を contour span lookup 経由でも `MalformedGlyfRecord` として観測できる。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4j: sfnt simple glyph contour-local point lookup

目的:

- F4i の contour span と F4h の single point decode を合成し、contour-local point index から 1 点だけを復元する。
- full point `Vec` / full contour `Vec` / curve segment builder / rasterizer は後続 phase で実装する。
- F4j は allocation なしで動作し、streaming contour sink の前段になる typed boundary を提供する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphContourPoint` と `gui_sfnt_lookup_simple_glyph_contour_point` を追加する。
- `GuiSfntSimpleGlyphContourPoint` は `span GuiSfntSimpleGlyphContourSpan`、`contour_point_index i32`、`point GuiSfntSimpleGlyphPoint` を持つ。
- `contour_point_index` は contour-local index、nested `point.point_index` は glyph absolute point index とする。
- `absolute_point_index = span.start_point_index + contour_point_index` とする。
- 処理順序は `contour span lookup -> validate contour_point_index -> compute absolute_point_index -> point decode` とし、local index validation を point decode より先に行う。
- `contour_point_index < 0` または `contour_point_index >= span.point_count` は `MissingGlyphOutline` とする。
- F4i / F4h から返る `MalformedGlyfRecord` などの typed error は伝播する。
- `gui_sfnt_glyf_simple_contour_point_with_tables` は public wrapper ではなく `gui_sfnt_glyf_simple_contour_span_with_tables` と `gui_sfnt_glyf_simple_point_with_tables` を通る。
- Source policy で contour point API、internal table helper reuse、local-before-point validation、absolute point index formula、metadata 非依存、no Vec allocation を固定する。

完了条件:

- two-contour fixture の contour 0 local 0 が absolute point 0、x 0、y 0、not contour end を返す。
- two-contour fixture の contour 1 local 1 が absolute point 3、contour end true を返す。
- one-contour signed coordinate fixture の local 1 が absolute point 1、x 2、y -6、on_curve true を返す。
- local index `-1` と `span.point_count` は `MissingGlyphOutline` になる。
- coordinate overrun fixture を contour point lookup 経由でも `MalformedGlyfRecord` として扱える。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4k: sfnt simple glyph contour edge lookup

目的:

- contour-local edge index から、contour topology 上で隣接する start / end point pair を 1 つだけ復元する。
- edge は描画線分ではなく topology pair であり、quadratic curve classification、implied on-curve point、winding、rasterization は後続 phase で実装する。
- full edge `Vec` / full contour `Vec` / curve segment builder は作らず、allocation なしの lookup boundary を提供する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphContourEdge` と `gui_sfnt_lookup_simple_glyph_contour_edge` を追加する。
- `GuiSfntSimpleGlyphContourEdge` は `start GuiSfntSimpleGlyphContourPoint`、`end GuiSfntSimpleGlyphContourPoint`、`edge_index i32`、`next_contour_point_index i32` を持つ。
- `edge_index` は contour-local edge start index、`next_contour_point_index` は wrap 後の contour-local end index とする。
- `start.contour_point_index == edge_index`、`end.contour_point_index == next_contour_point_index` を不変条件とする。
- nested `start.point.point_index` と `end.point.point_index` は glyph absolute point index のままとする。
- 処理順序は `contour span lookup -> validate edge_index -> compute next_contour_point_index -> decode start contour point -> decode end contour point` とし、edge index validation を endpoint decode より先に行う。
- `edge_index < 0` または `edge_index >= span.point_count` は `MissingGlyphOutline` とする。
- `edge_index + 1 == span.point_count` の場合、`next_contour_point_index = 0` として contour end から contour start へ wrap する。
- `span.point_count == 1` の場合、`edge_index = 0` だけを成功させ、start と end が同じ point を参照する topology self-wrap とする。
- `gui_sfnt_glyf_simple_contour_edge_with_tables` は public wrapper ではなく `gui_sfnt_glyf_simple_contour_span_with_tables` と `gui_sfnt_glyf_simple_contour_point_with_tables` を通る。
- Source policy で contour edge API、internal table helper reuse、edge-before-endpoint validation、wrap formula、metadata 非依存、no Vec allocation を固定する。

完了条件:

- two-contour fixture の contour 0 edge 0 が start local 0 / absolute 0、end local 1 / absolute 1、wrap なしを返す。
- two-contour fixture の contour 1 last edge が start local 1 / absolute 3、next local 0、end absolute 2 を返す。
- one-point contour fixture の edge 0 が next local 0、start/end absolute point equal の self-wrap を返す。
- signed coordinate fixture の edge 1 が start absolute point 1、x 2、y -6 を返す。
- edge index `-1` と `span.point_count` は `MissingGlyphOutline` になる。
- coordinate overrun fixture を contour edge lookup 経由でも `MalformedGlyfRecord` として扱える。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4l: sfnt simple glyph curve segment classification

目的:

- F4k の contour topology edge から、line / quadratic / no-segment を enum payload として 1 つだけ分類する。
- TrueType simple glyph の implied on-curve midpoint を exact に表すため、coordinate は font unit の 2 倍である `x2` / `y2` として保持する。
- full segment `Vec` / full outline `Vec` / streaming contour sink / rasterizer は作らず、allocation なしの classifier boundary を提供する。
- valid topology だが現在 edge start から drawable segment を出さない状態を `NoSegment` の成功値として返し、parse error と混同しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphCurveNoSegmentReason`
  - `GuiSfntSimpleGlyphCurveNoSegment`
  - `GuiSfntSimpleGlyphLineSegment`
  - `GuiSfntSimpleGlyphQuadraticSegment`
  - `GuiSfntSimpleGlyphCurveSegment`
  - `gui_sfnt_classify_simple_glyph_curve_segment`
  - `gui_sfnt_lookup_simple_glyph_curve_segment`
- `GuiSfntSimpleGlyphCurveSegment` は `NoSegment` / `Line` / `Quadratic` の payload 付き enum とし、inactive field を持つ shared struct にはしない。
- `Line` は edge.start / edge.end が両方 on-curve の場合だけ返す。
- `Quadratic` は edge.start が on-curve、edge.end が off-curve の場合だけ返す。edge.end は control point とする。
- quadratic end が explicit on-curve の場合、`end_x2 = lookahead.x * 2`、`end_y2 = lookahead.y * 2` とする。
- quadratic end が implied midpoint の場合、`end_x2 = control.x + lookahead.x`、`end_y2 = control.y + lookahead.y` とする。`div_s ... 2` や丸めは使わない。
- `span.point_count == 1` は `NoSegment SinglePointContour` の成功値とする。
- edge.start が off-curve の場合は `NoSegment OffCurveStart` の成功値とする。F4l は implied contour start を合成しない。
- pure classifier で off-curve end に `lookahead = None` が渡された場合は `NoSegment MissingLookahead` とし、byte lookup 側ではこの状態を出さないように必要な時だけ lookahead を読む。
- `gui_sfnt_glyf_simple_curve_segment_with_tables` は public wrapper ではなく `gui_sfnt_glyf_simple_contour_edge_with_tables` と `gui_sfnt_glyf_simple_contour_point_with_tables` を通る。
- Source policy で curve segment API、payload enum、doubled coordinate field、no integer midpoint division、conditional lookahead decode、internal helper reuse、metadata 非依存、no curve segment `Vec` allocation を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_curve.n.md` を追加し、巨大化した `tests/stdlib/gui_font_sfnt_glyf.n.md` とは別に分類規則の doctest を保持する。
- `tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md` を追加し、public `gui_sfnt_lookup_simple_glyph_curve_segment` が最小 SFNT byte fixture から odd implied midpoint の `Quadratic` へ到達する smoke を保持する。
- 現時点の compiler では `alloc/gui/font/sfnt/glyf` import の resource static check が 60 秒制限に近いため、public lookup smoke は `skip` 付きの仕様化 doctest とし、source policy で fixture、public lookup 呼び出し、`ByteBuilder` binary construction、`io_bytebuf_from_str_result` 禁止を固定する。

完了条件:

- on-curve -> on-curve edge が `Line` になり、start/end doubled coordinate を返す。
- on-curve -> off-curve -> on-curve が `Quadratic` になり、control doubled coordinate と explicit end doubled coordinate を返す。
- on-curve -> off-curve -> off-curve が `Quadratic` になり、`end_is_implied = true`、odd midpoint を `end_x2` / `end_y2` で丸めず返す。
- 1 point contour が `Result::Ok (NoSegment SinglePointContour)` 相当の typed success になる。
- off-curve start が `NoSegment OffCurveStart` の typed success になる。
- `edge_index` 範囲外や malformed bytes は引き続き `Result::Err GuiSfntParseError` になる。
- classifier helper と byte lookup helper は full outline allocation、`Vec GuiSfntSimpleGlyphCurveSegment`、rasterizer、platform API、fallback rendering path を使わない。
- public lookup smoke は UTF-8 text conversion ではなく `ByteBuilder` で binary SFNT bytes を組み立てる。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
# skip policy check: current compiler exceeds the normal 60s timeout for this byte-level public lookup smoke.
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve_lookup.json -j 1
# executable smoke check until the resource static check is made faster:
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve_lookup_long.json -j 1; Remove-Item Env:NEPL_TEST_CASE_TIMEOUT_MS
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4m: sfnt simple glyph path command projection

目的:

- F4l の `GuiSfntSimpleGlyphCurveSegment` を、後続の outline / path sink が読む明示的な move command / draw command へ写す。
- full outline `Vec` / streaming sink trait / winding / fill rule / rasterizer / render2d command はまだ作らない。
- `NoSegment` を parse error や silent no-op にせず、`SkipNoSegment` command として明示的に保持する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathMoveTo`
  - `GuiSfntSimpleGlyphPathLineTo`
  - `GuiSfntSimpleGlyphPathQuadraticTo`
  - `GuiSfntSimpleGlyphPathSkipNoSegment`
  - `GuiSfntSimpleGlyphPathCommand`
  - `gui_sfnt_simple_glyph_curve_segment_move_to_command`
  - `gui_sfnt_simple_glyph_curve_segment_draw_command`
- `GuiSfntSimpleGlyphPathCommand` は `MoveTo` / `LineTo` / `QuadraticTo` / `SkipNoSegment` の payload 付き enum とし、inactive field を持つ shared struct にはしない。
- Path command payload は full edge / line / quadratic / no-segment value を再保持せず、source contour/edge index、doubled coordinate、no-segment reason の小さな値へ射影する。
- `Line` は `move_to_command` で `MoveTo`、`draw_command` で `LineTo` を返す。
- `Quadratic` は `move_to_command` で `MoveTo`、`draw_command` で `QuadraticTo` を返す。
- `NoSegment` はどちらの関数でも `SkipNoSegment` を返す。
- command index を受け取らず、`Option` / `Result` も返さない。
- `MoveTo`、`LineTo`、`QuadraticTo` は F4l の doubled coordinate をそのまま使い、integer midpoint division や coordinate fallback を行わない。
- Source policy で path command API、payload enum、no command index / no `Option` / no `Result` contract、`SkipNoSegment`、no `Vec GuiSfntSimpleGlyphPathCommand` allocation、no metadata parse、no render2d/backend/platform import、no rasterizer を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` を追加し、typed value から line / quadratic / no-segment projection を検査する。

完了条件:

- `Line` segment が `MoveTo` と `LineTo` を明示的な関数で返す。
- `Quadratic` segment が control / end doubled coordinate と `end_is_implied` を保持する `QuadraticTo` を返す。
- `NoSegment` が `SkipNoSegment` と reason を保持する。
- path command projection は full outline allocation、`Vec GuiSfntSimpleGlyphPathCommand`、rasterizer、platform API、render2d command、fallback rendering path を使わない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4n: sfnt simple glyph path command public lookup

目的:

- SFNT byte input から contour-local edge の move / draw path command を public API として取得できるようにする。
- F4l の byte-backed curve segment lookup と F4m の path command projection を合成するだけに限定する。
- full outline `Vec` / command list / sink trait / winding / fill rule / rasterizer / render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_move_to_command`
  - `gui_sfnt_lookup_simple_glyph_draw_command`
- 両関数は `gui_sfnt_lookup_simple_glyph_curve_segment` を呼び、`Result::Err` は同じ `GuiSfntParseError` として伝播する。
- `Result::Ok segment` の場合、move helper は `gui_sfnt_simple_glyph_curve_segment_move_to_command`、draw helper は `gui_sfnt_simple_glyph_curve_segment_draw_command` を呼び、`Result::Ok GuiSfntSimpleGlyphPathCommand` を返す。
- F4n では `gui_sfnt_parse_metadata`、`*_with_tables` helper、point / contour table helper、curve classification logic を直接呼ばない。
- `NoSegment` は `Result::Ok SkipNoSegment` として保持し、`Result::Err`、`Option::None`、empty command、silent no-op、fallback rendering path にしない。
- Source policy で public signatures、F4l/F4m composition、no metadata unwrap / no table-helper bypass / no `Vec GuiSfntSimpleGlyphPathCommand` / no render2d/backend/platform import を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に `NoSegment -> move_to_command -> SkipNoSegment` の cheap typed doctest assertion を追加する。

完了条件:

- move lookup と draw lookup が `Result GuiSfntSimpleGlyphPathCommand GuiSfntParseError` を返す。
- move lookup は byte-backed curve segment lookup の成功値を F4m move projection に渡す。
- draw lookup は byte-backed curve segment lookup の成功値を F4m draw projection に渡す。
- F4n は full outline allocation、command list、rasterizer、platform API、render2d command、metadata unwrap bypass を使わない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4o: sfnt simple glyph path command pair lookup

目的:

- 同じ contour-local edge の move command と draw command を 1 つの pair value として取得できるようにする。
- F4n の move lookup と draw lookup を別々に呼ぶことで同じ SFNT edge decode が 2 回走る問題を避ける。
- contour stream、command sequence、full outline `Vec`、sink trait、winding、fill rule、rasterizer、render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathCommandPair`
  - `gui_sfnt_simple_glyph_path_command_pair`
  - `gui_sfnt_simple_glyph_path_command_pair_move_command`
  - `gui_sfnt_simple_glyph_path_command_pair_draw_command`
  - `gui_sfnt_simple_glyph_curve_segment_path_command_pair`
  - `gui_sfnt_lookup_simple_glyph_path_command_pair`
- `GuiSfntSimpleGlyphPathCommandPair` は ordered list ではなく、`move_command` と `draw_command` だけを持つ O(1) value とする。
- pure helper は F4m の `move_to_command` と `draw_command` を同じ segment に適用して pair を返す。
- byte-backed helper は `gui_sfnt_lookup_simple_glyph_curve_segment` を 1 回だけ呼び、`Result::Err` は同じ `GuiSfntParseError` として伝播する。
- `Result::Ok segment` の場合、`gui_sfnt_simple_glyph_curve_segment_path_command_pair` を呼び、`Result::Ok GuiSfntSimpleGlyphPathCommandPair` を返す。
- `NoSegment` は pair 内の move / draw の両方で `SkipNoSegment` の成功値として保持する。
- F4o では command index、count、next、current point state、`Vec GuiSfntSimpleGlyphPathCommand`、`push` を導入しない。
- F4o public helper では `gui_sfnt_parse_metadata`、`*_with_tables` helper、lower public lookup helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で pair API、curve lookup 1 回、pair helper composition、no list / no sink / no metadata unwrap / no table-helper bypass を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に line pair、implied quadratic pair、NoSegment pair の typed doctest assertion を追加する。

完了条件:

- line segment pair が `MoveTo` と `LineTo` を保持する。
- implied quadratic segment pair が `MoveTo` と `QuadraticTo` を保持し、doubled coordinate と `end_is_implied` を落とさない。
- NoSegment pair が move / draw の両方で `SkipNoSegment` と reason を保持する。
- byte-backed public lookup が curve segment lookup を 1 回だけ呼ぶ thin composition になっている。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4p: sfnt simple glyph path sink event adapter

目的:

- F4o の `GuiSfntSimpleGlyphPathCommandPair` を、後続の contour/path sink が読む single-edge event pair へ写す。
- full contour stream、command sequence、sink trait、ownership / allocation boundary、winding、fill rule、rasterizer、render2d command はまだ作らない。
- `SkipNoSegment` を empty event にせず、既存の typed path command を event として保持する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkEvent`
  - `GuiSfntSimpleGlyphPathSinkEventPair`
  - `gui_sfnt_simple_glyph_path_command_sink_event`
  - `gui_sfnt_simple_glyph_path_sink_event_command`
  - `gui_sfnt_simple_glyph_path_sink_event_pair`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_first_event`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_second_event`
  - `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair`
- `GuiSfntSimpleGlyphPathSinkEvent` は `Command GuiSfntSimpleGlyphPathCommand` の thin wrapper とし、`MoveTo` / `LineTo` / `QuadraticTo` / `SkipNoSegment` payload を再定義しない。
- `GuiSfntSimpleGlyphPathSinkEventPair` は `first_event` と `second_event` だけを持つ O(1) value とする。
- pure helper は `gui_sfnt_simple_glyph_path_command_pair_move_command` と `gui_sfnt_simple_glyph_path_command_pair_draw_command` だけを読み、first / second event を作る。
- F4p では `Option` / `Result`、command index、count、next、current point state、contour closure、off-curve contour-start synthesis、`Vec GuiSfntSimpleGlyphPathSinkEvent`、`push` を導入しない。
- F4p の pure helper では byte-backed lookup、metadata parser、`*_with_tables` helper、lower point / contour helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で pair-to-sink-event adapter、thin wrapper、event pair accessors、no duplicate payload enum、no lookup/parser/helper bypass、no allocation/stream state を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に direct path command から sink event / event pair を作る cheap typed doctest assertion を追加する。line / quadratic / NoSegment の payload behavior は既存 F4m/F4o doctest と F4p source policy で固定し、既存の重い executable case へ nested event match は足さない。

完了条件:

- direct `MoveTo` / `LineTo` command pair が first event `MoveTo`、second event `LineTo` として読める。
- direct `SkipNoSegment` command が `GuiSfntSimpleGlyphPathSinkEvent::Command` の内側で `SkipNoSegment` として読める。
- implied quadratic pair と NoSegment pair の payload preservation は F4m/F4o の executable doctest と F4p source policy で固定される。
- pure adapter が lookup / parser / table helper / renderer / platform API に依存しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4q: sfnt simple glyph path sink event kind classification

目的:

- F4p の `GuiSfntSimpleGlyphPathSinkEvent` を、後続 sink の dispatch 用 kind へ写す。
- kind は path command payload の軽量版ではなく、座標や contour/edge の authority は既存 event command payload に残す。
- real sink trait、ownership / allocation boundary、contour traversal、winding、fill、rasterizer、render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkEventKind`
  - `GuiSfntSimpleGlyphPathSinkEventKindPair`
  - `gui_sfnt_simple_glyph_path_sink_event_kind`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair`
- `GuiSfntSimpleGlyphPathSinkEventKind` は `MoveTo`、`LineTo`、`QuadraticTo`、`SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason` だけを持つ。
- `SkipNoSegment` kind の reason は diagnostics / skip counting / branch selection 用であり、source contour / edge 復元用ではない。
- kind には `contour_index`、`edge_index`、`x2`、`y2`、`control_x2`、`end_x2` などを入れない。
- `gui_sfnt_simple_glyph_path_sink_event_kind` は `gui_sfnt_simple_glyph_path_sink_event_command` で command を読み、全 variant を明示的に `match` する。catch-all arm は使わない。
- `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair` は F4p event pair accessors と `gui_sfnt_simple_glyph_path_sink_event_kind` だけを使う。
- F4q では `Option` / `Result`、`Vec GuiSfntSimpleGlyphPathSinkEventKind`、`push`、command index、count、next、current point state、contour closure、off-curve contour-start synthesis、byte-backed lookup、metadata parser、`*_with_tables` helper、lower point / contour helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で kind の dispatch 専用性、no duplicate payload、no coordinate/source index fields、no allocation/stream state、no lookup/parser/helper bypass を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の direct sink event doctest に、`MoveTo` / `LineTo` kind pair と `SkipNoSegment` reason kind を確認する cheap typed assertion を追加する。

完了条件:

- direct `MoveTo` event が `GuiSfntSimpleGlyphPathSinkEventKind::MoveTo` として分類される。
- direct `LineTo` event が `GuiSfntSimpleGlyphPathSinkEventKind::LineTo` として分類される。
- direct `SkipNoSegment` event が reason を保持した `GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment` として分類される。
- kind helper が lookup / parser / table helper / renderer / platform API に依存しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4r: sfnt simple glyph path sink event indexed selection

目的:

- F4p/F4q の two-slot pair から、後続 sink が first / second event または kind を O(1) に選択できる typed boundary を追加する。
- numeric index ではなく enum slot を使い、不正 event index を型として表現不能にする。
- contour traversal、iterator、command count、current point state、rasterizer、render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkEventSlot`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_event_at`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_kind_at`
- `GuiSfntSimpleGlyphPathSinkEventSlot` は `First` と `Second` だけを持ち、`Clone` / `Copy` を実装する。
- `event_at` は slot を明示 `match` し、`First` なら `gui_sfnt_simple_glyph_path_sink_event_pair_first_event`、`Second` なら `gui_sfnt_simple_glyph_path_sink_event_pair_second_event` だけを使う。catch-all arm は使わない。
- `kind_pair_kind_at` は slot を明示 `match` し、kind pair の first / second accessor だけを使う。catch-all arm は使わない。
- `event_pair_kind_at` は `gui_sfnt_simple_glyph_path_sink_event_pair_event_at` と `gui_sfnt_simple_glyph_path_sink_event_kind` の合成だけで実装する。kind classification logic を重複させない。
- F4r では `i32` event index、`Option` / `Result`、`Vec`、`push`、command index、count、next、current point state、contour traversal、contour closure、off-curve contour-start synthesis、byte-backed lookup、metadata parser、`*_with_tables` helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で slot enum、no numeric index、total selection、event/kind accessor composition、no allocation/stream state、no lookup/parser/helper bypass を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の direct sink event doctest に、`First` / `Second` slot で event と kind を取得できる cheap typed assertion を追加する。

完了条件:

- `First` slot が first event / first kind を返す。
- `Second` slot が second event / second kind を返す。
- event pair から single slot kind を読む helper が event selection と F4q kind helper の合成だけで動く。
- F4r は numeric index、full outline allocation、stream state、rasterizer、platform API、metadata unwrap bypass を使わない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4s: sfnt simple glyph path contour traversal step

目的:

- F4r の typed slot selection を、1 contour 内の 1 event step traversal に接続する。
- cursor / next / step を enum と struct で表し、`Option` や numeric index で終端や slot を表さない。
- public lookup は range / parse error を `Result` で返し、contour end は成功値 `GuiSfntSimpleGlyphPathContourNext::EndContour` として返す。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathContourCursor`
  - `GuiSfntSimpleGlyphPathContourNext`
  - `GuiSfntSimpleGlyphPathContourStep`
  - cursor / step constructor と accessor
  - private `gui_sfnt_simple_glyph_path_contour_next_from_cursor`
  - public `gui_sfnt_lookup_simple_glyph_path_contour_step`
- `GuiSfntSimpleGlyphPathContourCursor` / `GuiSfntSimpleGlyphPathContourNext` / `GuiSfntSimpleGlyphPathContourStep` は `Clone` / `Copy` を実装する。
- private next helper は、public lookup が `span_point_count > 0` と `0 <= edge_index < span_point_count` を検証した後だけ呼ぶ。public total helper にしない。
- next helper は slot を明示 `match` し、`First` なら same edge `Second`、`Second` なら `edge + 1` の `First` または `EndContour` を返す。catch-all arm は使わない。
- public lookup は `gui_sfnt_lookup_simple_glyph_contour_span` で contour span / point count を検証し、`gui_sfnt_lookup_simple_glyph_path_command_pair` で edge を path command pair に変換する。
- public lookup は `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair`、`gui_sfnt_simple_glyph_path_sink_event_pair_event_at`、`gui_sfnt_simple_glyph_path_sink_event_kind`、private next helper を合成する。
- F4s は `Vec`、`push`、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass を使わない。
- off-curve contour-start synthesis と contour closure insertion は F4s では行わず、既存 `SkipNoSegment OffCurveStart` を typed event として保持する。
- Source policy で cursor / next / step 型、Clone / Copy、private next helper、public lookup composition、no fallback/no allocation/no renderer/no platform を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に constructor/accessor の cheap typed assertion と、最小 SFNT fixture を使う public `gui_sfnt_lookup_simple_glyph_path_contour_step` doctest を追加する。
- public lookup doctest は `First -> Second`、`Second -> next edge First`、final `Second -> EndContour`、out-of-range edge の `GuiSfntParseErrorKind::MissingGlyphOutline` を直接検査する。
- 現行 doctest runner では public glyf lookup fixture の compile が 60 秒制限を超えるため、public lookup fixture は `skip` とし、`nodesrc/test_web_gui_font_rendering_contract.js` で doctest 名、public call、typed error branch の存在を固定する。

完了条件:

- cursor は glyph / contour / edge / slot を保持し、accessor で読める。
- step は cursor / event / kind / next を保持し、accessor で読める。
- `First` は同じ edge の `Second` に進む。
- final ではない `Second` は次 edge の `First` に進む。
- final edge の `Second` は `EndContour` を返す。
- public lookup は parse/range 不正だけ `Result::Err` にし、contour end は `Result::Ok step` の `EndContour` として返す。
- F4s は full outline allocation、renderer、platform API、font fallback、off-curve contour-start synthesis を導入しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4t: sfnt simple glyph allocation-free path sink ownership boundary

目的:

- F4s の `GuiSfntSimpleGlyphPathContourStep` を、real sink trait へ進む前の allocation-free sink decision に写す。
- off-curve contour-start synthesis と contour closure insertion を、別々の typed policy として分離する。
- policy reject を `GuiSfntParseError` に混ぜず、success payload 内の enum decision として保持する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathOffCurveStartPolicy`
  - `GuiSfntSimpleGlyphPathClosurePolicy`
  - `GuiSfntSimpleGlyphPathSinkPolicy`
  - `GuiSfntSimpleGlyphPathSinkRejectReason`
  - `GuiSfntSimpleGlyphPathSinkPrimaryAction`
  - `GuiSfntSimpleGlyphPathContourClose`
  - `GuiSfntSimpleGlyphPathSinkTailAction`
  - `GuiSfntSimpleGlyphPathSinkStep`
  - constructor / accessor
  - `gui_sfnt_simple_glyph_path_sink_primary_action_from_contour_step`
  - `gui_sfnt_simple_glyph_path_sink_tail_action_from_contour_step`
  - `gui_sfnt_simple_glyph_path_sink_step_from_contour_step`
  - public `gui_sfnt_lookup_simple_glyph_path_sink_step`
- `GuiSfntSimpleGlyphPathOffCurveStartPolicy` は `KeepTypedSkip` / `RejectUnsupported` を持つ。
- `GuiSfntSimpleGlyphPathClosurePolicy` は `KeepOpen` / `EmitCloseAfterFinalEvent` を持つ。
- `RejectUnsupported` は `SkipNoSegment OffCurveStart` だけを `Reject UnsupportedOffCurveStart` に写す。`SinglePointContour` と `MissingLookahead` は emit する。
- `GuiSfntSimpleGlyphPathSinkPrimaryAction` は `EmitEvent` / `Reject` を持ち、reject reason は dedicated enum にする。
- `GuiSfntSimpleGlyphPathSinkTailAction` は `NoTailAction` / `CloseContour` を持つ。
- tail action は次の規則にする。
  - `Reject` なら常に `NoTailAction`
  - `Continue` なら常に `NoTailAction`
  - `EndContour` かつ `KeepOpen` なら `NoTailAction`
  - `EndContour` かつ `EmitCloseAfterFinalEvent` かつ primary が emit なら `CloseContour`
- `CloseContour` は source cursor の glyph / contour index だけを持つ marker とし、renderer command にはしない。
- byte-backed public helper は `gui_sfnt_lookup_simple_glyph_path_contour_step` を呼び、成功値を pure sink-step helper に渡すだけにする。
- F4t は `Vec`、`push`、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass を使わない。
- Source policy で F4t の type set、reject/close 排他、OffCurveStart 限定、EndContour 限定 close、F4s lookup 委譲を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に cheap typed assertion を追加する。
  - keep policy は off-curve skip を emit し、final step だけ close marker を出す。
  - reject policy は off-curve start を reject にし、final step でも close marker を出さない。
  - `Continue` step は close marker を出さない。
  - `RejectUnsupported` でも `SinglePointContour` は emit される。
- F4s の skipped public lookup fixture に `gui_sfnt_lookup_simple_glyph_path_sink_step` の call を含め、source policy で byte-backed helper の存在を固定する。

完了条件:

- policy、primary action、tail action、sink step はすべて enum/struct payload として表現される。
- policy reject は `Result::Err` ではなく `GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject` になる。
- reject と close contour は同時に発生しない。
- close contour は primary が emit で、かつ `step.next = EndContour` の場合だけ発生し得る。
- off-curve policy は `OffCurveStart` だけに作用する。
- F4t は full outline allocation、renderer、platform API、font fallback、off-curve start synthesis を導入しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4u: sfnt simple glyph path sink action selection projection

目的:

- F4t の `GuiSfntSimpleGlyphPathSinkStep` から、future sink が順に処理する action を enum slot で選べるようにする。
- `Primary` / `Tail` の action 選択を、F4r/F4s の `First` / `Second` event slot から明確に分離する。
- `NoTailAction` を明示的な `NoAction` に写し、fallback や silent no-op とは別の typed state として扱う。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionSlot`
  - `GuiSfntSimpleGlyphPathSinkAction`
  - action slot の `Clone` / `Copy`
  - action の `Clone` / `Copy`
  - `gui_sfnt_simple_glyph_path_sink_action_slot_is_primary`
  - `gui_sfnt_simple_glyph_path_sink_action_slot_is_tail`
  - `gui_sfnt_simple_glyph_path_sink_primary_action_as_action`
  - `gui_sfnt_simple_glyph_path_sink_tail_action_as_action`
  - `gui_sfnt_simple_glyph_path_sink_step_action_at`
  - public `gui_sfnt_lookup_simple_glyph_path_sink_action`
- `GuiSfntSimpleGlyphPathSinkActionSlot` は `Primary` / `Tail` だけを持つ。
- `GuiSfntSimpleGlyphPathSinkAction` は `EmitEvent` / `Reject` / `CloseContour` / `NoAction` を持つ。
- primary action projection は `EmitEvent` / `Reject` だけを返し、`NoAction` を返さない。
- tail action projection は `NoTailAction -> NoAction`、`CloseContour -> CloseContour` だけを行う。
- `gui_sfnt_simple_glyph_path_sink_step_action_at` は slot の網羅的 `match` で `Primary` または `Tail` を選ぶ。
- byte-backed public helper は `gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び、成功値に pure action projection を適用する。
- F4u は `Vec`、`push`、numeric action index、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass、`*_with_tables` bypass を使わない。
- Source policy で F4u の type set、slot 軸の分離、primary が `NoAction` を返さないこと、tail の `NoAction` 限定、F4t lookup への 1 回委譲を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の cheap typed assertion を拡張する。
  - `Primary` slot は primary action、`Tail` slot は tail action を選ぶ。
  - `EmitEvent` / `Reject` / `CloseContour` / `NoAction` が明示的に区別される。
  - `NoAction` は tail の `NoTailAction` だけから得られる。

完了条件:

- sink action selection は enum / match で表現され、数値 index や fallback branch を持たない。
- `GuiSfntSimpleGlyphPathSinkActionSlot` は `GuiSfntSimpleGlyphPathSinkEventSlot` と混同されない。
- primary action projection は `NoAction` を返さない。
- policy reject は `Result::Err` ではなく `GuiSfntSimpleGlyphPathSinkAction::Reject` として保持される。
- byte-backed helper は F4t lookup にだけ委譲し、下位 glyph/contour/curve helper を直接呼ばない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4v: sfnt simple glyph path sink action traversal step

目的:

- F4u の single action projection を、contour 内で順に読める typed traversal step へ拡張する。
- future sink が `Primary -> Tail -> F4s source next` の順に action を読むための cursor / next / step を追加する。
- real sink、callback、`Vec` command stream、full outline allocation、renderer、rasterizer、platform API はまだ導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionCursor`
  - `GuiSfntSimpleGlyphPathSinkActionNext`
  - `GuiSfntSimpleGlyphPathSinkActionStep`
  - constructor / accessor
  - `Clone` / `Copy`
  - `gui_sfnt_simple_glyph_path_sink_action_next_from_step`
  - `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step`
  - public `gui_sfnt_lookup_simple_glyph_path_sink_action_step`
- `GuiSfntSimpleGlyphPathSinkActionCursor` は checked `GuiSfntSimpleGlyphPathContourCursor` と `GuiSfntSimpleGlyphPathSinkActionSlot` を持つ。
- 新しい numeric action index、command index、loop index、count field、ad-hoc traversal counter は追加しない。既存 contour cursor 内の `contour_index` / `edge_index` は F4s の authority として保持する。
- `GuiSfntSimpleGlyphPathSinkActionNext` は `Continue` / `EndContour` を持つ。contour 終端を `Option::None` や error で表さない。
- next の規則は次とする。
  - `Primary` は action payload に関係なく同じ contour cursor の `Tail` へ進む。
  - `Tail` は action payload に関係なく `sink_step.source_step.next` に従う。
  - `source_step.next = Continue next_cursor` なら `next_cursor Primary` へ進む。
  - `source_step.next = EndContour` なら `EndContour` を返す。
- `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` は F4u の `gui_sfnt_simple_glyph_path_sink_step_action_at` を使い、primary / tail action の中身を再分類しない。
- byte-backed public helper は `gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び、成功値を pure action-step helper に渡すだけにする。
- F4v は `Vec`、`push`、numeric action index、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass、`*_with_tables` bypass を使わない。
- Source policy で F4v の type set、payload-independent traversal、F4u action projection reuse、F4t lookup への 1 回委譲、下位 glyph/contour/curve helper へ直接入らないことを固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の cheap typed assertion を拡張する。
  - Primary は emit / reject に関係なく same contour cursor Tail へ進む。
  - Tail は `Continue next_cursor` の場合に next cursor Primary へ進む。
  - Tail は `EndContour` の場合に `EndContour` へ進む。
  - Tail の `NoAction` は traversal stop ではなく、F4s source next に従う。

完了条件:

- traversal state は enum / struct payload として表現され、numeric action index を持たない。
- action payload と next state は分離される。
- next は action payload を見ず、action slot と F4s source step next だけから決まる。
- byte-backed helper は F4t lookup にだけ委譲し、下位 helper を直接呼ばない。
- F4v は full outline allocation、renderer、platform API、font fallback、off-curve start synthesis を導入しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4w: sfnt simple glyph path sink action start cursor

目的:

- F4v の action traversal に、contour-local action stream の開始 cursor を追加する。
- 開始 cursor を `edge 0` / `First` / `Primary` として型で固定する。
- pure constructor と byte-backed validated entry point を分け、unchecked value construction と byte validation を混同しない。
- action payload lookup、sink policy、full outline allocation、renderer、rasterizer、platform API は導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_simple_glyph_path_sink_action_start_cursor`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`
- pure helper は `gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index 0 GuiSfntSimpleGlyphPathSinkEventSlot::First` を作り、`GuiSfntSimpleGlyphPathSinkActionSlot::Primary` と合成する。
- pure helper は unchecked value constructor であり、byte 妥当性、contour 存在、span 範囲、point count を検証しない。
- byte-backed helper は `gui_sfnt_lookup_simple_glyph_contour_span` を 1 回だけ呼び、成功した場合にだけ pure helper へ委譲する。
- byte-backed helper は F4v action-step lookup、F4t sink-step lookup、F4s contour-step lookup、point / curve / path-command helper、sink policy、renderer、rasterizer、platform font API を呼ばない。
- Source policy で F4w の doc contract、pure helper の `edge 0` / `First` / `Primary`、byte-backed helper の contour span lookup への 1 回委譲、追加 NEPL body に括弧がないことを固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の cheap typed assertion を拡張する。
  - `gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph 3` が contour `3`、edge `0`、event slot `First`、action slot `Primary` を返すことを確認する。

完了条件:

- 開始 cursor は enum slot と既存 contour cursor で表現され、numeric action index や command index を持たない。
- pure constructor は byte validation を行わないことが doc と実装で明示される。
- byte-backed helper は contour span validation にだけ委譲し、action payload や policy を読まない。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4x: sfnt simple glyph path sink action start step

目的:

- F4w の start cursor と F4v の action step lookup を接続し、contour の first action step を読む public helper を追加する。
- F4x 自体は新しい validation authority にならず、既存 action step lookup の Result 境界を再利用する。
- contour span 検証の二重実行を避けるため、byte-backed start cursor helper は呼ばない。
- real sink、full outline allocation、command list、renderer、rasterizer、platform API は導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` を追加する。
- helper は `gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index` を 1 回呼ぶ。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy` を 1 回呼ぶ。
- helper は `Result::Err error` / `Result::Ok action_step` を明示的に `match` し、新しい判断や error 変換を行わない。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`、`gui_sfnt_lookup_simple_glyph_contour_span`、`gui_sfnt_lookup_simple_glyph_path_sink_step`、F4s/F4t より下位の lookup を直接呼ばない。
- Source policy で F4x の doc contract、pure start cursor 1 回、action step lookup 1 回、禁止 helper、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture に呼び出しを追加する。
  - `Result::Ok action_step` から cursor を読み、contour `0`、edge `0`、event slot `First`、action slot `Primary` を確認する。
  - `Result::Err` は false とし、typed Result branch を明示する。

完了条件:

- start step helper は `start cursor construction + existing checked action step lookup` だけに閉じる。
- parse/range error は `Result::Err` として伝播し、policy reject は `Result::Ok` action payload に残る。
- byte-backed start cursor helper と contour span lookup を直接呼ばず、検証の二重化を避ける。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4y: sfnt simple glyph path sink action step advance

目的:

- F4v の `GuiSfntSimpleGlyphPathSinkActionStep.next` を 1 段だけ進める byte-backed helper を追加する。
- `Continue cursor` は checked action step lookup で次 step に解決し、`EndContour` は成功値として返す。
- contour 終端を `Option::None` や `Result::Err` で表さない。
- loop traversal、real sink、full outline allocation、command list、renderer、rasterizer、platform API は導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionStepAdvance`
  - `Clone` / `Copy`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance`
- `GuiSfntSimpleGlyphPathSinkActionStepAdvance` は `Continue GuiSfntSimpleGlyphPathSinkActionStep` / `EndContour` を持つ。
- helper は `gui_sfnt_simple_glyph_path_sink_action_step_next step` を読み、`match` する。
- `Continue cursor` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy` を 1 回呼ぶ。
- `Result::Err error` はそのまま伝播し、`Result::Ok next_step` は `GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step` に包む。
- `EndContour` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` として返す。
- helper は action payload を見ない。`GuiSfntSimpleGlyphPathSinkAction::Reject`、`NoAction`、`CloseContour` などで traversal を変えない。
- helper は start cursor/start step helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API を直接呼ばない。
- Source policy で F4y enum、Clone/Copy、helper body、下位 lookup 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` を拡張する。
  - cheap assertion で `GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` が成功 terminal enum として `match` できることを確認する。
  - skipped byte-backed fixture で `start_step -> advance` が `Continue next_step` を返し、next step cursor が same contour/edge/event の `Tail` であることを確認する。

完了条件:

- action step advance は `Continue next_step` / `EndContour` の typed enum で表現される。
- `Result` は byte parse/range/table error の伝播にだけ使われ、contour 終端や policy reject を error にしない。
- traversal は `step.next` だけから決まり、action payload を読まない。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4z: sfnt simple glyph path sink action step item

目的:

- F4v の `GuiSfntSimpleGlyphPathSinkActionStep` と F4y の checked advance を、後続 sink consumer が読む 1 action 分の typed item として束ねる。
- 現在 action step と次状態の lookup 結果を同時に渡せるようにしつつ、contour-wide traversal や real sink mutation には進まない。
- `EndContour` は `GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` として item 内に残し、`Option::None` や `Result::Err` に変換しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionStepItem`
  - `Clone` / `Copy`
  - `gui_sfnt_simple_glyph_path_sink_action_step_item`
  - `gui_sfnt_simple_glyph_path_sink_action_step_item_step`
  - `gui_sfnt_simple_glyph_path_sink_action_step_item_advance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item`
- `GuiSfntSimpleGlyphPathSinkActionStepItem` は `step GuiSfntSimpleGlyphPathSinkActionStep` と `advance GuiSfntSimpleGlyphPathSinkActionStepAdvance` を持つ。
- byte-backed helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy` を 1 回だけ呼ぶ。
- `Result::Err error` はそのまま伝播する。
- `Result::Ok advance` では `let stored_step %GuiSfntSimpleGlyphPathSinkActionStep *step` により現在 step を明示コピーし、`GuiSfntSimpleGlyphPathSinkActionStepItem` を返す。
- helper は action payload を見ない。`Reject`、`NoAction`、`CloseContour` などで traversal を変えない。
- helper は start cursor/start step helper、F4v action step lookup、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API を直接呼ばない。
- Source policy で F4z struct、Clone/Copy、constructor/accessor、helper body、F4y helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` を拡張する。
  - cheap assertion で synthetic action step と `EndContour` advance から item を作り、accessor で step と terminal advance を確認する。
  - skipped byte-backed fixture で `start_step -> action_step_item` が `Continue next_step` を持ち、next step cursor が same contour/edge/event の `Tail` であることを確認する。

完了条件:

- action step item は現在 step と checked advance を value として保持する。
- item helper は F4y helper だけに委譲し、lower lookup や start composition を行わない。
- `Result` は byte parse/range/table error の伝播にだけ使われ、contour 終端や policy reject を error にしない。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4aa: sfnt simple glyph path sink action start item

目的:

- F4x の first action step helper と F4z の action step item helper を接続し、contour の first action item を読む public helper を追加する。
- F4aa 自体は新しい validation authority、new item type、contour-wide traversal、real sink mutation にはならない。
- `Result::Err` は parse/range/table error の伝播にだけ使い、policy reject や contour terminal state は F4x/F4z の typed value として残す。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_item:
    &ByteBuf
    Option i32
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。
- start step が `Result::Err error` ならそのまま `Result::Err error` を返す。
- start step が `Result::Ok start_step` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy` を 1 回だけ呼ぶ。
- action step item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok item` はそのまま返す。
- helper は action payload を見ない。`Reject`、`NoAction`、`CloseContour`、`EndContour` などで traversal を変えない。
- helper は start cursor helper、F4v action step lookup、F4y advance helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API を直接呼ばない。
- Source policy で F4aa docs、helper body、F4x helper 1 回、F4z helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture を拡張する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy` を呼ぶ。
  - item 内の stored step cursor が contour `0`、edge `0`、event slot `First`、action slot `Primary` であることを確認する。
  - advance が `Continue next_step` で、next step cursor が same contour/edge/event の `Tail` であることを確認する。

完了条件:

- start item helper は F4x と F4z を value として合成し、同じ `GuiSfntSimpleGlyphPathSinkActionStepItem` を返す。
- helper body は F4x helper と F4z helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new type duplication、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ab: sfnt simple glyph path sink action item next

目的:

- F4z/F4aa の `GuiSfntSimpleGlyphPathSinkActionStepItem` から、次の action item または contour terminal state を 1 段だけ取得する public helper を追加する。
- F4ab は contour-wide traversal、iterator、real sink mutation、command list、full outline allocation、renderer、rasterizer にはならない。
- `EndContour` は successful terminal state として enum payload に残し、`Result::Err`、`Option::None`、hidden no-op へ変換しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionItemNext`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionStepItem
    EndContour
```

- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_item_next:
    &ByteBuf
    Option i32
    &GuiSfntSimpleGlyphPathSinkActionStepItem
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntParseError
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_advance item` を 1 回だけ読む。
- `advance = Continue next_step` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &next_step policy` を 1 回だけ呼ぶ。
- step item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok next_item` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item` として返す。
- `advance = EndContour` の場合は `Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` を返す。
- helper は `item.step`、`GuiSfntSimpleGlyphPathSinkActionStep.next`、action payload、primary/tail action、sink policy payload を読まない。
- helper は start cursor/start step/start item helper、F4v action step lookup、F4y advance helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ab docs、enum、Clone/Copy、helper body、item advance accessor 1 回、F4z helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - synthetic item の `EndContour` advance を `GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` として返すことを確認する。
  - byte-backed fixture で `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item` から得た item を `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next` に渡し、`Continue next_item` を得ることを確認する。
  - next item の stored step cursor が same contour/edge/event の `Tail` action slot であることを確認する。

完了条件:

- item next helper は F4z item の checked advance と F4z step-item lookup だけを value として合成する。
- helper body は item advance accessor と F4z helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ac: sfnt simple glyph path sink action consumer item

目的:

- F4z/F4aa の `GuiSfntSimpleGlyphPathSinkActionStepItem` から、future sink consumer が 1 action 分として読む typed packet を追加する。
- 現在 action と F4ab の checked next state を束ね、後続 sink が hidden current state に依存しない入力境界を作る。
- F4ac は real sink、iterator、contour-wide consumer、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerItem`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item_action`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item`
- struct は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItem:
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item:
    &ByteBuf
    Option i32
    &GuiSfntSimpleGlyphPathSinkActionStepItem
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_step item` を 1 回だけ読み、`gui_sfnt_simple_glyph_path_sink_action_step_action &stored_step` で action を 1 回だけ読む。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy` を 1 回だけ呼ぶ。
- `Result::Err error` はそのまま伝播し、`Result::Ok next` なら `GuiSfntSimpleGlyphPathSinkActionConsumerItem action next` を `Result::Ok` で返す。
- helper は `EmitEvent` / `Reject` / `NoAction` / `CloseContour` payload、primary/tail action、sink policy payload を match しない。
- helper は F4z action step item lookup、F4y advance helper、F4v action step lookup、F4x/F4aa start helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ac docs、struct、Clone/Copy、constructor/accessors、helper body、step accessor 1 回、action accessor 1 回、F4ab item-next helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - byte-backed fixture で `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item` から得た item を consumer item helper に渡す。
  - consumer item の `action` が current start action を保持していることを確認する。
  - consumer item の `next` が `Continue next_item` であり、next item の cursor が same contour/edge/event の `Tail` action slot であることを確認する。

完了条件:

- consumer item helper は F4z item の current action copy と F4ab next state だけを value として合成する。
- helper body は step accessor、action accessor、F4ab item-next helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ad: sfnt simple glyph path sink action consumer item next

目的:

- F4ac の `GuiSfntSimpleGlyphPathSinkActionConsumerItem` から、次の consumer item または contour terminal state を 1 段だけ取得する public helper を追加する。
- future sink loop が hidden current state に依存せず、typed packet continuation を扱える境界を作る。
- F4ad は contour-wide traversal、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    EndContour
```

- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next:
    &ByteBuf
    Option i32
    &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItemNext GuiSfntParseError
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item` を 1 回だけ読む。
- `next = Continue next_item` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy` を 1 回だけ呼ぶ。
- consumer item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok next_consumer_item` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer_item` として返す。
- `next = EndContour` の場合は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour` を返す。
- helper は current action、`EmitEvent` / `Reject` / `NoAction` / `CloseContour` payload、primary/tail action、sink policy payload を読まない。
- helper は F4ab item next lookup、F4z action step item lookup、F4y advance helper、F4v action step lookup、F4x/F4aa start helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ad docs、enum、Clone/Copy、helper body、consumer item next accessor 1 回、F4ac consumer item helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - synthetic consumer item の `EndContour` next を `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour` として返すことを確認する。
  - byte-backed fixture で start consumer item から `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼び、`Continue next_consumer_item` を得ることを確認する。

完了条件:

- consumer item next helper は F4ac consumer item の checked next と F4ac consumer item lookup だけを value として合成する。
- helper body は consumer next accessor と F4ac helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ae: sfnt simple glyph path sink action apply state

目的:

- F4ac/F4ad の consumer item が保持する `GuiSfntSimpleGlyphPathSinkAction` を 1 action だけ消費し、明示的な domain status と count state に変換する。
- `Reject`、`CloseContour`、`NoAction` を hidden fallback や silent no-op にせず、enum status として future sink に渡せる境界を作る。
- F4ae は contour-wide traversal、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionApplyStatus`
  - `GuiSfntSimpleGlyphPathSinkActionApplyState`
  - `GuiSfntSimpleGlyphPathSinkActionApplyStep`
  - constructor / accessor helper
  - `gui_sfnt_simple_glyph_path_sink_action_apply_state_new`
  - `gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionApplyStatus:
    EmittedEvent GuiSfntSimpleGlyphPathSinkEvent
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    ClosedContour GuiSfntSimpleGlyphPathContourClose
    NoAction
```

- state は次の 4 count を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionApplyState:
    emitted_event_count i32
    reject_count i32
    close_contour_count i32
    no_action_count i32
```

- helper は `GuiSfntSimpleGlyphPathSinkAction` を `match` し、各 variant で対応する count だけを `add count 1` する。
- `Reject` は `Result::Err` へ変換しない。typed reject status として `Rejected reason` を返す。
- `NoAction` は silent no-op ではない。`NoAction` status と `no_action_count + 1` を返す。
- count state は diagnostic / contract 検査用であり、cursor、next state、traversal authority として使わない。
- helper は F4ad consumer next、F4ac consumer item lookup、F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ae docs、enum / struct、Clone/Copy、constructor / accessor、apply helper body、4 variant の count 更新、禁止 helper、`Result` / `Option` / allocation / renderer 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - `EmitEvent`、`Reject`、`CloseContour`、`NoAction` を順に apply し、それぞれの status と count が明示的に更新されることを確認する。
  - `NoAction` が test 上でも no-op ではなく `no_action_count` を進めることを確認する。

完了条件:

- action apply helper は 1 action を 1 status に変換し、1 counter だけを更新する。
- `Rejected` と `NoAction` は成功系の domain status として保持される。
- traversal authority は F4ac/F4ad に残り、F4ae は cursor / next state を決めない。
- hidden fallback、silent no-op、new traversal loop、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4af: sfnt simple glyph path sink action consumer apply step

目的:

- F4ac の `GuiSfntSimpleGlyphPathSinkActionConsumerItem` から current action を F4ae apply state に適用し、apply result と保存済み checked continuation を同じ value として運ぶ。
- future loop / real sink が「今回の消費結果」と「次に進むための保存済み next」を同時に読める境界を作る。
- F4af は byte-backed next lookup、contour-wide traversal、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep`
  - constructor / accessor helper
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply`
- struct は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionApplyStep
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_action item` を 1 回だけ読む。
- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item` を 1 回だけ読む。
- helper は `gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action` を 1 回だけ呼ぶ。
- helper は `apply_step` と `next` を `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` に束ねる。
- `next` は F4ac packet に保存されていた `GuiSfntSimpleGlyphPathSinkActionItemNext` であり、F4af が新しく決める traversal state ではない。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` も呼ばない。次 consumer item への byte-backed 解決は F4ad に残す。
- helper は action payload を直接 `match` しない。payload 解釈は F4ae helper だけに委譲する。
- helper は `Result`、`Option`、F4ad/F4ac byte-backed lookup、F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4af docs、struct、Clone/Copy、constructor / accessor、consumer item action accessor 1 回、consumer item next accessor 1 回、F4ae apply helper 1 回、F4ad next helper 禁止、payload match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - synthetic consumer item を `apply` し、status / state count と保存済み `next` が同時に読めることを確認する。
  - `next` が `GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` のまま保存され、`GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` へ変換されないことを確認する。

完了条件:

- consumer item apply helper は current action を F4ae helper へ委譲し、保存済み checked continuation をそのまま同梱する。
- helper は F4ad の next resolution を呼ばず、traversal authority を持たない。
- hidden fallback、silent no-op、payload direct match、new traversal loop、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ag: sfnt simple glyph path sink action consumer apply terminal

目的:

- F4af の `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` を future consumer loop が扱う typed terminal 判定に変換する。
- `Rejected`、保存済み `EndContour`、保存済み `Continue` を enum で明示し、hidden fallback や silent skip を作らない。
- F4ag は contour-wide loop、byte-backed next lookup、real sink mutation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_reject_reason`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
```

- `Rejected reason` は malformed SFNT parse error ではなく domain terminal なので、`Result::Err` にはしない。
- 保存済み `EndContour` は successful terminal なので、これも `Result::Err` にはしない。
- `NoAction` は silent no-op ではないが、それだけで terminal にしない。`NoAction + Continue` は `Continue`、`NoAction + EndContour` は `EndContour` とする。
- helper は F4af の `apply_step` と `next` だけを読む。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` も呼ばない。
- helper は action payload を直接 `match` しない。reject reason の取り出しは `GuiSfntSimpleGlyphPathSinkActionApplyStatus` の分類だけに限定する。
- helper は F4ad/F4ac byte-backed lookup、F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ag docs、enum、Clone/Copy、reject reason helper、terminal helper、F4ad next helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - `Rejected` status が保存済み `EndContour` より優先されることを確認する。
  - 保存済み `EndContour` が successful terminal になることを確認する。
  - `NoAction + Continue` が terminal ではなく `Continue` になることを確認する。

完了条件:

- consumer apply step は `Continue` / `Rejected` / `EndContour` の typed terminal 判定に分類される。
- `Rejected` と `EndContour` を `Result::Err` に逃がさない。
- F4ag は next consumer item lookup や traversal loop を実装しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ah: sfnt simple glyph path sink action consumer apply advance

目的:

- F4ag の terminal 判定を使い、apply 後の consumer stream を 1 step だけ進める byte-backed boundary を追加する。
- `Continue` は次 consumer item、`Rejected` は domain terminal、`EndContour` は successful terminal として enum で明示する。
- F4ah は contour-wide loop、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step` を 1 回だけ呼ぶ。
- `Rejected reason` は `Result::Ok Rejected reason` にする。`Result::Err` にはしない。
- `EndContour` は `Result::Ok EndContour` にする。`Result::Err` にはしない。
- `Continue continue_step` では、`gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next &continue_step` を読み、保存済み `GuiSfntSimpleGlyphPathSinkActionItemNext` を authority とする。
- 保存済み next が `Continue next_item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy` を 1 回だけ呼び、成功時は `Continue next_consumer_item` を返す。
- 保存済み next が `EndContour` なら successful terminal として `EndContour` を返す。
- helper は original `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を要求しない。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼ばない。これは F4ad direct wrapper ではなく、F4ag terminal と保存済み `ActionItemNext` から F4ac lookup へ接続する 1 step boundary である。
- helper は action payload を直接 `match` せず、F4ae apply helper も呼ばない。
- helper は F4ad/F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ah docs、enum、Clone/Copy、terminal helper 1 回、stored next accessor 1 回、F4ac lookup 1 回、F4ad next helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に F4ah 用の contract doctest を追加する。
  - `Rejected` terminal が `Ok Rejected` になることを確認する。
  - `EndContour` terminal が `Ok EndContour` になることを確認する。
  - `Continue` branch の byte-backed lookup path は source policy で固定し、必要なら後続 byte-backed fixture で拡張する。
  - F4ah helper は F4ac byte-backed lookup を参照するため、現行 compiler の 60 秒 doctest 制限では外部 `.n.md` fixture の compile が timeout する。したがって runnable ではなく `skip` 付き contract doctest とし、source policy で terminal helper / stored next / F4ac lookup の exact call pattern を固定する。

完了条件:

- F4ah は F4ag terminal 判定から `Continue` / `Rejected` / `EndContour` の apply advance を返す。
- `Rejected` と `EndContour` を `Result::Err` に逃がさない。
- F4ah は F4ad next helper や contour-wide loop を実装しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側は F4ah contract fixture を `skip` として数える。実行可能な validation は既存 F4ag terminal doctest と `stdlib/alloc/gui/font/sfnt/glyf.nepl` doctest に置き、F4ah の byte-backed composition は `nodesrc/test_web_gui_font_rendering_contract.js` で静的検査する。

## Phase F4ai: sfnt simple glyph path sink action consumer item consume once

目的:

- 1 consumer item を F4af で apply し、その apply step を F4ah で 1 step advance する境界を追加する。
- F4af の apply state / status を捨てず、advance と同じ typed value に保持する。
- F4ai は contour-wide loop、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once`
- struct は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

- consume-once helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item` を 1 回だけ呼ぶ。
- consume-once helper は得られた `apply_step` を `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index &apply_step policy` へ 1 回だけ渡す。
- advance が `Result::Err error` なら parse/range failure としてそのまま伝播する。
- advance が `Result::Ok advance` なら、`apply_step` と `advance` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` に束ねて `Result::Ok` で返す。
- helper は F4ag を直接呼ばない。terminal classification は F4ah の責務である。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼ばない。F4ad direct wrapper に戻すと apply result preservation が曖昧になる。
- helper は action payload を直接 `match` せず、F4ae apply helper も直接呼ばない。
- helper は F4ad/F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ai docs、struct、Clone/Copy、constructor / accessor、F4af helper 1 回、F4ah helper 1 回、constructor 1 回、F4ag direct call 禁止、F4ad next helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に F4ai 用の contract doctest を追加する。
  - synthetic `Rejected` case で apply status / state count と `Rejected` advance の両方が保持されることを確認する。
  - synthetic `EndContour` case で apply status / state count と `EndContour` advance の両方が保持されることを確認する。
  - F4ai helper は F4ah byte-backed lookup を参照するため、現行 compiler の 60 秒 doctest 制限で外部 `.n.md` fixture の compile が timeout する場合は `skip` 付き contract doctest とし、source policy で exact call pattern を固定する。

完了条件:

- consume-once result は apply step と advance を両方保持する。
- F4ai は F4af と F4ah の薄い合成に留まり、F4ag/F4ad/lower traversal へ直接依存しない。
- F4ai は loop、real sink、renderer、rasterizer、platform backend、font fallback を実装しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側の F4ai fixture が byte-backed helper materialization で timeout する場合は、F4ah と同じく `skip` として数える。実装 body の exact call pattern は `nodesrc/test_web_gui_font_rendering_contract.js` で固定する。

## Phase F4aj: sfnt simple glyph path sink action start consumer item

目的:

- contour start から future consumer loop の初期 `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を読む public helper を追加する。
- F4aa start item と F4ac consumer item を薄く合成し、新しい value type や traversal authority を作らない。
- F4aj は consume、apply、post-apply advance、consumer item next、contour-wide loop、real sink mutation、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item:
    &ByteBuf
    Option i32
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。
- start item が `Result::Err error` ならそのまま `Result::Err error` を返す。
- start item が `Result::Ok item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &item policy` を 1 回だけ呼ぶ。
- consumer item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok consumer_item` はそのまま返す。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` と `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once` を呼ばない。
- helper は F4af apply、F4ah apply advance、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- F4ac は consumer item を作る契約上、checked `GuiSfntSimpleGlyphPathSinkActionItemNext` を内部で読む。この F4ac 内部処理は許容し、F4aj 自体の consumer item next / consume / apply / advance とは区別する。
- Source policy で F4aj docs、helper body、F4aa helper 1 回、F4ac helper 1 回、F4ad/F4ai/F4af/F4ah/direct lower helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture を拡張する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item &bytes none glyph 0 &sink_policy` を呼ぶ。
  - 成功時の action が first event の `EmitEvent` であることを確認する。
  - checked next が `Continue next_item` で、next item の stored cursor が same contour/edge/event の `Tail` action slot であることを確認する。

完了条件:

- start consumer item helper は F4aa と F4ac を value として合成し、同じ `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を返す。
- helper body は F4aa helper と F4ac helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側は既存 byte-backed public lookup fixture を `skip` として数える。実装 body の exact call pattern は `nodesrc/test_web_gui_font_rendering_contract.js` で固定する。

## Phase F4ak: sfnt simple glyph path sink action start consume once

目的:

- contour start から first consumer item を作り、その 1 item だけを consume する public helper を追加する。
- F4aj start consumer item と F4ai consume once を薄く合成し、F4ai の apply step / advance preservation contract をそのまま保つ。
- F4ak は contour-wide loop、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once:
    &ByteBuf
    Option i32
    GuiSfntSimpleGlyphPathSinkActionApplyState
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。
- start consumer item が `Result::Err error` ならそのまま `Result::Err error` を返す。
- start consumer item が `Result::Ok consumer_item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &consumer_item policy` を 1 回だけ呼ぶ。
- consume-once helper の `Result::Err error` はそのまま伝播し、`Result::Ok consume_step` はそのまま返す。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` だけを返してはならない。F4ai と同じ `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` を返し、apply state / status と post-consume advance を保持する。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、F4aa start item、F4ac consumer item、F4ad consumer item next、F4af apply、F4ah apply advance、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4ak docs、helper body、F4aj helper 1 回、F4ai helper 1 回、F4aa/F4ac/F4ad/F4af/F4ah/direct lower helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture を拡張する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once &bytes none state glyph 0 &sink_policy` を呼ぶ。
  - 成功時の consume step から apply step を読み、first event が `EmittedEvent` として status に残ることを確認する。
  - apply state の emitted event count が 1 になることを確認する。
  - advance が `Continue next_consumer` であり、next consumer の action が same edge tail の `NoAction` として保持されることを確認する。

完了条件:

- start consume-once helper は F4aj と F4ai を value として合成し、同じ `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` を返す。
- helper body は F4aj helper と F4ai helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側は既存 byte-backed public lookup fixture を `skip` として数える。実装 body の exact call pattern は `nodesrc/test_web_gui_font_rendering_contract.js` で固定する。

## Phase F4al: sfnt simple glyph path sink action consumer consume step apply summary

目的:

- `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` から consume 後 apply state と consumed action status を読む pure public helper を追加する。
- future consumer loop が F4ai/F4af の nested storage layout へ直接依存しないようにする。
- F4al は loop、iterator、real sink mutation、byte-backed lookup、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status`
- helper signature は次にする。

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state:
    &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionApplyState

gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status:
    &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionApplyStatus
```

- state helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step` を 1 回だけ呼ぶ。
- state helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step &consumer_apply_step` を 1 回だけ呼ぶ。
- state helper は `gui_sfnt_simple_glyph_path_sink_action_apply_step_state &inner_apply_step` を 1 回だけ呼ぶ。
- status helper は同じ first two calls の後、`gui_sfnt_simple_glyph_path_sink_action_apply_step_status &inner_apply_step` を 1 回だけ呼ぶ。
- helper は `advance` を読まない。traversal / terminal state は既存 `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance` の責務として分離する。
- helper は `Result`、`Option`、byte-backed lookup、consumer item next、consume-once、start helper、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4al docs、helper body、exact call count、advance 禁止、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で新しい state / status helpers を使い、nested layout へ直接入らないことを確認する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - start consume-once result から新しい state / status helpers を使い、first action status / count と post-consume advance を別々に読む。

完了条件:

- consume step apply summary helper は consume step の apply side だけを読む。
- future loop が更新後 state / consumed status を nested F4af/F4ae layout へ直接依存せずに読める。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4am: sfnt simple glyph path sink action consumer consume summary value

目的:

- `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` から、future consumer loop が直接扱う state / status / advance の flat summary value を作る。
- F4al の apply summary helper と既存 `advance` accessor を 1 value に束ね、future loop が nested F4ai/F4af/F4al storage layout へ依存しないようにする。
- F4am は contour-wide loop、iterator、real sink mutation、byte-backed lookup、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step`
- summary type は次の 3 fields を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

- `summary_from_step` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step` を 1 回だけ呼ぶ。
- `summary_from_step` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step` を 1 回だけ呼ぶ。
- `summary_from_step` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance step` を 1 回だけ呼ぶ。
- F4al の apply-state/status helper は引き続き `advance` を読まない。F4am だけが full consume summary contract として既存 advance accessor を読む。
- helper は `Result`、`Option`、byte-backed lookup、consumer item next lookup、consume-once、start helper、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4am docs、summary type、Clone / Copy、constructor/accessors、from-step exact call count、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で summary を作り、summary accessors から state / status / advance を読む。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - start consume-once result から summary を作り、first action status / count / post-consume advance を summary accessors から読む。

完了条件:

- consume summary は state / status / advance を 1 value として持つ。
- from-step helper は F4al state helper、F4al status helper、existing advance accessor をそれぞれ 1 回だけ読む。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4an: sfnt simple glyph path sink action consumer consume summary terminal

目的:

- `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` に保持された post-consume advance を、future loop が読む traversal control state へ写す。
- F4am は state / status / advance を束ねるだけで advance を解釈しない。F4an は stored advance の 3 分岐を 1 回だけ読み、loop 側が lower `ApplyAdvance` storage detail に直接依存しないようにする。
- `Terminal` は名前として使うが `Continue` も含む。これは terminal-only value ではなく traversal control projection である。
- F4an は contour-wide loop、iterator、real sink mutation、byte-backed lookup、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal`
- summary terminal type は次の 3 variants を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

- `summary_terminal` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance summary` を 1 回だけ呼ぶ。
- `summary_terminal` は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue item` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue item` に写す。
- `summary_terminal` は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected reason` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected reason` に写す。
- `summary_terminal` は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour` に写す。
- helper は `Result`、`Option`、byte-backed lookup、consumer item next lookup、consume-once、start helper、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4an docs、summary terminal enum、Clone / Copy、helper exact advance accessor call count、3 分岐の同型写像、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で summary terminal helper を使い、Rejected / EndContour を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - start consume-once result から summary terminal helper を使い、Continue 分岐と次 consumer item の action を検査する。

完了条件:

- summary terminal は Continue / Rejected / EndContour を 1 value として持つ。
- summary terminal helper は stored advance accessor を 1 回だけ読み、lower advance enum を同型写像する。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ap: sfnt simple glyph path sink action start consume summary

目的:

- F4ak start consume-once と F4am consume summary projection を薄く合成し、future consumer loop の initial summary boundary を作る。
- F4ao の consume summary advance-once が受け取る `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` を、contour start から直接得られるようにする。
- 新しい enum は増やさず、既存 `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` を返す。
- F4ap は contour-wide loop、iterator、real sink mutation、summary advance、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary:
    &ByteBuf
    Option i32
    GuiSfntSimpleGlyphPathSinkActionApplyState
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy` を 1 回だけ呼ぶ。
- `Result::Err error` はそのまま `Result::Err error` として返す。
- `Result::Ok consume_step` の場合だけ、`gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を 1 回だけ呼ぶ。
- success branch は `Result::Ok summary` を返す。
- helper が直接使ってよい byte-backed lookup は start consume-once helper だけである。
- helper は start item、start consumer item、consumer item consume-once、summary advance-once、consumer item next lookup、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、full loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4ap docs、helper signature、exact call count、error propagation、success conversion、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture を更新する。
  - start summary helper を直接呼び、F4ak 経由で作った summary と同じ first action status / count / terminal を確認する。

完了条件:

- start consume summary helper は F4ak と F4am を value として合成し、initial summary を返す。
- full loop、hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ao: sfnt simple glyph path sink action consumer consume summary advance once

目的:

- F4am/F4an の summary boundary を使い、future consumer loop の 1 step advance 境界を作る。
- `Continue` の場合だけ次 consumer item を 1 つ消費し、次 summary を返す。
- `Rejected` と `EndContour` は parse error ではなく、`Result::Ok` の domain terminal として返す。
- F4ao は contour-wide loop、iterator、real sink mutation、byte-backed start traversal、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once`
- summary advance type は次の 3 variants を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary` を 1 回だけ呼ぶ。
- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を 1 回だけ呼ぶ。
- `Continue item` branch だけが `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &item policy` を 1 回だけ呼ぶ。
- consume-once が `Result::Err error` を返した場合は、その parse error をそのまま返す。
- consume-once が `Result::Ok consume_step` を返した場合は、`gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を 1 回だけ呼び、`Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Continue next_summary` を返す。
- `Rejected reason` branch は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Rejected reason` を返す。
- `EndContour` branch は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::EndContour` を返す。
- helper が直接使ってよい byte-backed lookup は Continue branch の consume-once helper だけである。
- helper は start helper、start consume-once、consumer item next lookup、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、full loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4ao docs、summary advance enum、Clone / Copy、helper exact call count、domain terminal `Result::Ok`、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で summary advance-once helper を使い、Rejected / EndContour が `Result::Ok` domain terminal として返ることを検査する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - first action summary から summary advance-once helper を使い、Continue が次 summary を返し、その次 summary が NoAction / EndContour になることを検査する。

完了条件:

- summary advance-once は full loop ではなく、1 summary から次 summary または domain terminal へ 1 step だけ進める。
- parse error と domain terminal を混同しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4aq: sfnt simple glyph path sink action consume summary drain budget

目的:

- F4ap initial summary と F4ao advance-once を使い、contour action consumer を explicit budget 内で domain terminal まで進める。
- `StepBudgetExhausted` を typed terminal として返し、unbounded traversal、silent success、hidden fallback を避ける。
- outline allocation / sink mutation / render command emission の前に、byte-backed traversal の停止点を enum として固定する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary_drain_budget`
- drain result type は次の 3 variants を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain:
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    StepBudgetExhausted GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
```

- drain helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を 1 回だけ呼ぶ。
- `Rejected reason` branch は budget を消費せず、`Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain::Rejected reason current_summary` を返す。
- `EndContour` branch は budget を消費せず、`Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain::EndContour current_summary` を返す。
- `Continue` かつ `remaining_steps <= 0` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain::StepBudgetExhausted current_summary` を返す。
- `Continue` かつ `remaining_steps > 0` の場合だけ、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once bytes face_index summary policy` を 1 回だけ呼ぶ。
- advance-once が `Result::Err error` を返した場合は、その parse error をそのまま返す。
- advance-once が `Continue next_summary` を返した場合は、`remaining_steps - 1` で drain helper を 1 回だけ再帰呼び出しする。
- advance-once が保守上 `Rejected` / `EndContour` を返した場合は、F4ao に渡した current summary を drain result に入れる。
- start drain helper は F4ap start consume summary を 1 回だけ呼び、成功時だけ drain helper へ 1 回渡す。
- start drain helper は F4ao を直接呼ばない。
- helper は action payload direct match、`Vec`、`push`、full outline allocation、renderer、rasterizer、platform API、host text API、font fallback、lower lookup、metadata parser、`*_with_tables` を直接使わない。
- Source policy で F4aq docs、drain enum、Clone / Copy、helper exact call count、`remaining_steps == 0` / `< 0` evidence、current summary terminal payload、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture を更新する。
  - first summary から drain budget 0 と -1 が `StepBudgetExhausted` になることを検査する。
  - start drain budget 2 が `EndContour` summary を返し、emitted event count と no-action count を保持することを検査する。

完了条件:

- drain helper は bounded traversal boundary であり、unbounded traversal や command allocation にはならない。
- parse error と domain terminal と budget exhaustion を混同しない。
- hidden fallback、silent no-op、new untyped traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5a: sfnt simple glyph outline storage capacity and owner recovery contract

目的:

- F4aq の bounded traversal の後に、simple glyph outline storage が必要とする capacity を allocation-free な value として計算する。
- capacity exceeded、invalid topology、command count overflow を enum branch として分離し、owner-taking allocation API の前に失敗時の owner recovery contract を固定する。
- outline allocation、sink mutation、renderer、rasterizer、platform API、host text API、font substitute へ進まない。

変更:

- 先に source policy を追加し、F5a docs、value type、helper の責務、禁止 API、括弧なし body を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlineStorageCapacity`
  - `GuiSfntSimpleGlyphOutlineStorageLimit`
  - `GuiSfntSimpleGlyphOutlineCapacityRejectReason`
  - `GuiSfntSimpleGlyphOutlineCapacityRejected`
  - `GuiSfntSimpleGlyphOutlineCapacityCheck`
  - `gui_sfnt_simple_glyph_outline_storage_capacity_from_topology`
  - `gui_sfnt_simple_glyph_outline_storage_capacity_check_limit`
- capacity fields は glyph、contour_count、point_count、edge_count、path_command_pair_count、path_command_count とする。
- `edge_count = point_count`、`path_command_pair_count = point_count`、`path_command_count = point_count * 2` とする。
- `contour_count <= 0`、`point_count <= 0`、`contour_count > point_count` は `InvalidTopology topology` とする。
- `point_count > 1073741823` は `CommandCountOverflow topology` とする。
- limit の各値は 1 以上を許可容量として扱う。0 以下は unlimited ではなく capacity exceeded とする。
- limit check は contour、point、edge、path command の順に最初の exceeded reason を返す。
- capacity exceeded は `GuiSfntSimpleGlyphOutlineCapacityRejected` として reason、capacity、limit を保持する。
- `GuiSfntSimpleGlyphOutlineCapacityRejectReason` は limit exceeded 専用であり、`InvalidTopology` と `CommandCountOverflow` は capacity が信頼できないため `GuiSfntSimpleGlyphOutlineCapacityCheck` の独立 variant とする。
- F5a helper は `Vec`、`push`、outline point list、contour list、path command list、renderer、rasterizer、platform API、host text API、font substitute、byte-backed lookup、metadata parser、`*_with_tables`、F4aq drain helper、lower contour helper、point decoder を使わない。
- doctest は synthetic topology と synthetic limit だけで分岐を検査する。byte-backed font fixture、renderer、raster、platform、host font API は使わない。

完了条件:

- valid topology から capacity が生成され、edge / path command count が仕様通りになる。
- forged invalid topology、command count overflow、各 limit exceeded が enum branch として検査される。
- F5a source policy が docs と implementation の責務逸脱を検出する。
- F4aq の `StepBudgetExhausted` が capacity success として扱われていないことを docs / policy で固定する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_capacity.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_capacity.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5b+: outline, shaping, ruby, vertical, math bridge

目的:

- glyph outline / mask、GSUB/GPOS、縦書き、ruby、math inline bridge を段階的に実装する。

注意:

- F5a の capacity / owner recovery contract を保ったまま、owner-taking storage API、outline point stream、raster mask、render2d command へ順に接続する。
- 未対応 feature は typed unsupported として返す。
- F5b 以降の outline doctest は timeout と責務混在を避けるため、phase ごとの専用ファイルに分ける。
  - F5b storage owner: `tests/stdlib/gui_font_sfnt_glyf_outline_storage.n.md`
  - F5c scalar push: `tests/stdlib/gui_font_sfnt_glyf_outline_scalar_push.n.md`
  - F5d region cursor: `tests/stdlib/gui_font_sfnt_glyf_outline_region_cursor.n.md`
  - F5e/F5f contour endpoint: `tests/stdlib/gui_font_sfnt_glyf_outline_contour_endpoint.n.md`
  - F5g PointX population: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x.n.md`
  - F5h PointX reader bridge success: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_success.n.md`
  - F5h PointX reader bridge read failure: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_read_failure.n.md`
  - F5h PointX reader bridge push failure: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_push_failure.n.md`
  - F5i/F5j PointY: `tests/stdlib/gui_font_sfnt_glyf_outline_point_y.n.md`
  - F5k coordinate read: `tests/stdlib/gui_font_sfnt_glyf_outline_point_coordinate.n.md`
  - F5l endpoint marker read: `tests/stdlib/gui_font_sfnt_glyf_outline_point_endpoint.n.md`

## Phase F5b: sfnt simple glyph outline scalar storage owner

目的:

- F5a の trusted capacity から、後続 outline builder が使う empty scalar slot storage owner を作る。
- forged capacity を capacity exceeded と混同せず、`InvalidCapacity` を limit rejection より前に返す。
- 複数 Vec owner の部分確保失敗を避けるため、F5b では 1 本の `Vec i32` scalar slot storage だけを確保する。
- point decode、contour decode、path command push、renderer、rasterizer、platform API、host text API、font substitute へ進まない。

変更:

- 先に source policy を追加し、F5b docs、storage owner、error enum、shape validation、scalar overflow guard、allocation/free 回数、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に `alloc/collections/vec` を qualified import する。
- 次の型を追加する。
  - `GuiSfntSimpleGlyphOutlineStorage`
  - `GuiSfntSimpleGlyphOutlineStorageAllocErrorKind`
  - `GuiSfntSimpleGlyphOutlineStorageAllocError`
  - `GuiSfntSimpleGlyphOutlineScalarSlotCountCheck`
- `GuiSfntSimpleGlyphOutlineStorage` は `capacity`、`scalar_slots Vec i32`、`scalar_slot_count` を持つ owner であり、`Clone` / `Copy` を実装しない。
- `scalar_slot_count` は `contour_count + point_count + point_count + edge_count + path_command_count` とする。
- `gui_sfnt_simple_glyph_outline_storage_capacity_shape_is_valid` は capacity shape を検査する。`point_count <= 1073741823` は `point_count * 2` 比較より前に確認する。
- `gui_sfnt_simple_glyph_outline_storage_scalar_slot_count_check` は staged residual guard で i32 overflow を検出する。
- `gui_sfnt_simple_glyph_outline_storage_alloc` は次の順序を守る。
  - `shape_is_valid` が false なら `InvalidCapacity` と `capacity_check = none` を返す。
  - shape が valid の場合だけ `gui_sfnt_simple_glyph_outline_storage_capacity_check_limit` を呼ぶ。
  - `Rejected` は `CapacityRejected` と `capacity_check = some checked` を返す。
  - `Fits` の場合だけ scalar slot count を検査する。
  - scalar overflow は `ScalarSlotCountOverflow` と `capacity_check = some checked` を返す。
  - `vec::with_capacity` は 1 回だけ呼ぶ。
  - allocation failure は `ScalarSlotStorageAllocFailed` と `capacity_check = some checked` を返す。
- `gui_sfnt_simple_glyph_outline_storage_free` は storage owner を消費し、`vec::free` を 1 回だけ呼ぶ。
- doctest は synthetic capacity / limit だけで success、invalid forged capacity、limit rejection、scalar slot overflow を検査する。byte-backed font fixture、renderer、raster、platform、host font API は使わない。

完了条件:

- small topology から storage が確保され、`len == 0`、`cap == scalar_slot_count`、`scalar_slot_count` が formula 通りである。
- forged invalid capacity は `CapacityRejected` ではなく `InvalidCapacity` になる。
- limit exceeded は shape valid の場合だけ `CapacityRejected` になる。
- scalar slot count overflow は allocation を試みず enum branch になる。
- source policy が docs、型、allocation ordering、`Vec` 呼び出し回数、storage owner の非 Copy / 非 Clone、禁止 API を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_storage.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_storage.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5c: sfnt simple glyph outline scalar slot push owner recovery

目的:

- F5b の `GuiSfntSimpleGlyphOutlineStorage` owner を消費し、scalar slot value を 1 件追加した owner を返す。
- `Vec` push failure を `StdErrorKind` だけへ潰さず、storage owner と rejected scalar value を error payload に返す。
- slot value の意味づけ、point decode、contour endpoint population、path command tag population、renderer、rasterizer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5c docs、push error type、helper signatures、owner recovery、禁止 API、`vec::push` 呼び出し回数を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlineStoragePushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_error`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_kind`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_scalar_value`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_storage`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_with`
  - `gui_sfnt_simple_glyph_outline_storage_push_scalar_slot`
- push helper は storage owner から capacity、scalar_slot_count、scalar_slots を取り出し、`vec::push scalar_slots value` を 1 回だけ呼ぶ。
- `Result::Ok next_slots` は `GuiSfntSimpleGlyphOutlineStorage capacity next_slots scalar_slot_count` を返す。
- `Result::Err e` は `vec::vec_push_error_kind &e` を先に読み、その後 `vec::vec_push_error_vec e` で returned slots を取り出し、returned storage と rejected scalar value と error kind を `GuiSfntSimpleGlyphOutlineStoragePushError` に入れて返す。
- F5c push helper は `vec::with_capacity`、`vec::free`、`vec::filled`、`vec::replace`、`vec::pop` を直接呼ばない。
- doctest は dedicated scalar push test file に success push と synthetic error recovery を追加する。real OOM は誘発しない。

完了条件:

- storage に scalar value を 2 件 push し、`len == 2`、`cap` と `scalar_slot_count` が F5b のまま保たれる。
- synthetic push error から storage owner、scalar value、error kind を取り出し、recovered storage を 1 回だけ free できる。
- source policy が F5c docs、型、helper、push の owner recovery、禁止 API、`vec::push` 1 回を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_scalar_push.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_scalar_push.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5d: sfnt simple glyph outline scalar region cursor

目的:

- F5b/F5c の single `Vec i32` storage owner に、contour endpoint、x、y、edge、path command tag の typed region cursor を追加する。
- unchecked boundary 計算を public API にせず、capacity shape と scalar slot count overflow を検査してから region start/end を計算する。
- fixed-capacity outline storage の invariant を守り、region push で Vec growth に依存しない。
- point decode、path command generation、renderer、rasterizer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5d docs、cursor type、region push result/error type、unchecked helper 非公開、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlineScalarRegion`
  - `GuiSfntSimpleGlyphOutlineScalarRegionCursor`
  - `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity`
  - non-public `gui_sfnt_simple_glyph_outline_scalar_region_cursor_from_valid_capacity`
  - `gui_sfnt_simple_glyph_outline_scalar_region_cursor_is_well_formed`
  - non-public cursor/capacity matching helper
  - `GuiSfntSimpleGlyphOutlineRegionPush`
  - `GuiSfntSimpleGlyphOutlineRegionPushErrorKind`
  - `GuiSfntSimpleGlyphOutlineRegionPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`
- `try_from_capacity` は `shape_is_valid` と `scalar_slot_count_check` が成功した後でだけ raw boundary helper を呼ぶ。
- `push_region_scalar` は capacity、`scalar_slot_count`、`scalar_slots_len`、`scalar_slots_cap` を先に読み、次の順序で検査する。
  - capacity shape
  - scalar slot count `Fits`
  - `scalar_slot_count == expected`
  - `scalar_slots_cap == scalar_slot_count`
  - cursor well-formed
  - cursor region/start/end match
  - `scalar_slots_len == cursor.next_index`
  - `cursor.next_index < cursor.end`
  - F5c `gui_sfnt_simple_glyph_outline_storage_push_scalar_slot` を 1 回だけ呼ぶ
- `scalar_slots_len == cursor.next_index` は `RegionFull` より前に検査する。
- `GuiSfntSimpleGlyphOutlineRegionPush` と `GuiSfntSimpleGlyphOutlineRegionPushError` は storage owner を持つため `Clone` / `Copy` を実装しない。
- doctest は cursor boundary、region push success、region full、storage cursor mismatch を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の capacity から、region cursor が `0..2`、`2..6`、`6..10`、`10..14`、`14..22` を返す。
- contour endpoint region に 2 件 push でき、storage len と cursor next index が 2 になる。
- full region への追加は storage owner と rejected scalar value を保持した `RegionFull` になる。
- empty storage に full cursor を渡す forged case は `StorageCursorMismatch` になる。
- source policy が unchecked public helper、validation order、fixed Vec cap invariant、F5c push 呼び出し回数、禁止 API、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_region_cursor.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_region_cursor.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5e: sfnt simple glyph contour endpoint population

目的:

- F5d の contour endpoint region cursor を使い、typed contour endpoint slot を owner-preserving に storage へ追加する。
- byte-backed endpoint array reading、point flag decode、x/y coordinate decode、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。
- capacity、cursor、endpoint sequence の validation order を source policy と doctest で固定する。

変更:

- 先に source policy を追加し、F5e docs、endpoint slot type、success/error owner payload、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphContourEndpointSlot`
  - `GuiSfntSimpleGlyphContourEndpointPush`
  - `GuiSfntSimpleGlyphContourEndpointPushErrorKind`
  - `GuiSfntSimpleGlyphContourEndpointPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint`
- public helper は storage capacity を検査してから `contour_count` / `point_count` を読む。
- cursor well-formed validation は `cursor.next_index` を読む前に行う。
- endpoint contour index range は final/non-final classification より前に検査する。
- previous endpoint range は `end_point_index > previous` より前に検査する。
- commit helper は F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、F5d error を `RegionPushFailed` に owner-preserving に包む。
- doctest は success、non-final endpoint at last point、final endpoint mismatch、forged PointX cursor region mismatch を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の synthetic endpoint 1, 3 を追加でき、storage len と cursor next index が 2、previous endpoint が 3 になる。
- non-final contour が final point を endpoint にした場合は `EndpointOutOfRange` になる。
- final contour endpoint が `point_count - 1` でない場合は `FinalEndpointMismatch` になる。
- PointX cursor を渡した場合は `CursorRegionMismatch` になり、storage cursor mismatch など下位 error に落ちない。
- source policy が capacity/cursor/endpoint validation order、F5d region push 呼び出し回数、direct `vec::` 禁止、byte/render/raster/platform/host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_contour_endpoint.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_contour_endpoint.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5f: sfnt simple glyph contour endpoint byte reader bridge

目的:

- 既存の checked `gui_sfnt_glyf_read_contour_endpoint` と F5e の `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint` を接続する。
- byte-backed endpoint array reading と owner-preserving storage mutation の error domain を分ける。
- x/y coordinate decode、flag decode、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5f docs、read-before-mutate ordering、read failure と push failure の分離、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphContourEndpointReadPush`
  - `GuiSfntSimpleGlyphContourEndpointReadPushErrorKind`
  - `GuiSfntSimpleGlyphContourEndpointReadPushError`
  - `gui_sfnt_glyf_read_push_contour_endpoint`
- public helper は `gui_sfnt_glyf_read_contour_endpoint` を 1 回だけ呼び、read failure では F5e push を呼ばない。
- read success では `GuiSfntSimpleGlyphContourEndpointSlot` を作り、F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint` を 1 回だけ呼ぶ。
- F5e push failure では endpoint、F5e error kind、F5d region error kind、F5c storage push error kind を owner 消費前に読む。
- doctest は byte-backed success、read failure owner recovery、push failure endpoint preservation を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- synthetic endpoint bytes から 2 contours / 4 points の endpoint 1, 3 を読み、storage len と cursor next index が 2、previous endpoint が 3 になる。
- endpoint byte range が table 外なら `ReadFailed` になり、parse error が `Some`、endpoint が `None`、storage len が 0 のまま回収できる。
- valid bytes だが F5e validation が失敗する場合は `PushFailed` になり、parse error が `None`、endpoint が `Some`、lower F5e error kind が `Some` になる。
- source policy が read-before-mutate、F5e push 呼び出し回数、lower error metadata の owner 消費前読み取り、direct `vec::` 禁止、point decode/render/raster/platform/host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_contour_endpoint.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_contour_endpoint.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5g: sfnt simple glyph point x coordinate population

目的:

- F5d の `PointX` region cursor を使い、typed x coordinate slot を owner-preserving に storage へ追加する。
- scalar storage index と glyph logical point index を混同しない validation order を固定する。
- byte-backed x decode、point flag decode、y coordinate、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5g docs、PointX slot type、success/error owner payload、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointXSlot`
  - `GuiSfntSimpleGlyphPointXPush`
  - `GuiSfntSimpleGlyphPointXPushErrorKind`
  - `GuiSfntSimpleGlyphPointXPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_point_x`
- public helper は capacity shape と scalar slot count `Fits` を検査してから `point_count` を読む。
- cursor well-formed validation と cursor/capacity boundary match は `logical_point_index = cursor.next_index - cursor.start` より前に行う。
- `PointX` region であることを確認し、`point.point_index == logical_point_index`、`0 <= point.point_index < point_count` を検査する。
- commit helper は F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、F5d error を `RegionPushFailed` に owner-preserving に包む。
- doctest は endpoint region を先に埋めてから PointX success、point index mismatch、wrong region を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の storage に contour endpoint 1, 3 を追加した後、PointX point 0 と point 1 を追加でき、storage len が 4、cursor next index が 4 になる。
- PointX cursor が logical point 0 を指す状態で slot point_index 1 を渡すと `PointIndexMismatch` になり、storage len が 2 のまま回収できる。
- PointY cursor を渡した場合は `CursorRegionMismatch` になり、storage len が 2 のまま回収できる。
- source policy が capacity/cursor/point validation order、F5d region push 呼び出し回数、direct `vec::` 禁止、`gui_sfnt_glyf_` / point decode / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5h: sfnt simple glyph point x byte reader bridge

目的:

- checked `GuiSfntSimpleGlyphPointStream` から 1 logical point の x coordinate だけを読み、F5g の `PointX` storage helper へ owner-preserving に接続する。
- byte-backed x read failure と F5g push failure の error domain を enum で分離する。
- y coordinate、endpoint array、contour span、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5h docs、read-before-mutate ordering、read failure と push failure の分離、owner 型の非 Clone / 非 Copy、x-only allowlist、full point / endpoint / render / platform 禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointXReadPush`
  - `GuiSfntSimpleGlyphPointXReadPushErrorKind`
  - `GuiSfntSimpleGlyphPointXReadPushError`
  - `gui_sfnt_glyf_read_push_point_x`
- `GuiSfntSimpleGlyphPointXReadPush` と `GuiSfntSimpleGlyphPointXReadPushError` は storage owner を持つため `Clone` / `Copy` を実装しない。
- success payload には cursor accessor と storage owner accessor を追加する。
- x-only internal helper は bounded flag reads と `gui_sfnt_glyf_decode_x_delta` だけを使う。
- `gui_sfnt_glyf_decode_y_delta`、full point decode state、endpoint read、contour span helper、public lookup wrapper、direct `Vec`、render/raster/platform/host API は使わない。
- forged bad y range は F5h では検査しない。PointY / full point phase の責務として document する。
- read failure では F5g push を呼ばず、point は `None`、parse error は `Some`、storage len は変更しない。
- read success では `GuiSfntSimpleGlyphPointXSlot` を作り、F5g `gui_sfnt_simple_glyph_outline_storage_push_point_x` を 1 回だけ呼ぶ。
- F5g push failure では rejected point、F5g error kind、F5d region error kind、F5c storage push error kind を owner 消費前に読む。
- doctest は endpoint region を先に埋めてから PointX read/push success、read failure owner recovery、push failure endpoint preservation を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の storage に contour endpoint 1, 3 を追加した後、byte-backed x reader から PointX point 0 と point 1 を追加でき、storage len が 4、cursor next index が 4 になる。
- x byte range が壊れた stream では `ReadFailed` になり、point は `None`、parse error は `Some`、storage len が 2 のまま回収できる。
- valid x read だが cursor が logical point 0 を指す状態で point_index 1 を push すると `PushFailed` になり、point は `Some`、lower F5g error kind が `Some PointIndexMismatch` になる。
- source policy が x-only allowlist、read-before-mutate、F5g push 呼び出し回数、lower error metadata の owner 消費前読み取り、direct `vec::` 禁止、full point / endpoint / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_success.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x_reader_success_f5h.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_read_failure.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x_reader_read_failure_f5h.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_push_failure.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x_reader_push_failure_f5h.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5h.json -j 1
git diff --check
```

## Phase F5i: sfnt simple glyph point y coordinate population

目的:

- F5d の `PointY` region cursor を使い、typed y coordinate slot を owner-preserving に storage へ追加する。
- `PointY` region は endpoint と全 `PointX` slot の後ろにあるため、scalar storage index と glyph logical point index の混同を防ぐ。
- byte-backed y decode、point flag decode、x coordinate、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5i docs、PointY slot type、success/error owner payload、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointYSlot`
  - `GuiSfntSimpleGlyphPointYPush`
  - `GuiSfntSimpleGlyphPointYPushErrorKind`
  - `GuiSfntSimpleGlyphPointYPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_point_y`
- public helper は capacity shape と scalar slot count `Fits` を検査してから `point_count` を読む。
- cursor well-formed validation と cursor/capacity boundary match は `logical_point_index = cursor.next_index - cursor.start` より前に行う。
- `PointY` region であることを確認し、`point.point_index == logical_point_index`、`0 <= point.point_index < point_count` を検査する。
- commit helper は F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、F5d error を `RegionPushFailed` に owner-preserving に包む。
- doctest は endpoint 2 slots と PointX 4 slots を先に埋めてから PointY success、point index mismatch、wrong region を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の storage に endpoint 2 slots と PointX 4 slots を追加した後、PointY point 0 と point 1 を追加でき、storage len が 8、cursor next index が 8 になる。
- PointY cursor が logical point 0 を指す状態で slot point_index 1 を渡すと `PointIndexMismatch` になり、storage len が 6 のまま回収できる。
- PointX cursor など wrong region を渡した場合は `CursorRegionMismatch` になり、storage len が 6 のまま回収できる。
- source policy が capacity/cursor/point validation order、F5d region push 呼び出し回数、direct `vec::` 禁止、`gui_sfnt_glyf_` / point decode / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_y.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_y_f5i.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5j.json -j 1
git diff --check
```

## Phase F5j: sfnt simple glyph point y byte reader bridge

目的:

- checked `GuiSfntSimpleGlyphPointStream` から 1 logical point の y coordinate だけを読み、F5i の `PointY` storage helper へ owner-preserving に接続する。
- byte-backed y read failure と F5i push failure の error domain を enum で分離する。
- x coordinate、endpoint array、contour span、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5j docs、read-before-mutate ordering、read failure と push failure の分離、owner 型の非 Clone / 非 Copy、y-only allowlist、full point / endpoint / render / platform 禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointYReadPush`
  - `GuiSfntSimpleGlyphPointYReadPushErrorKind`
  - `GuiSfntSimpleGlyphPointYReadPushError`
  - `gui_sfnt_glyf_read_push_point_y`
- `GuiSfntSimpleGlyphPointYReadPush` と `GuiSfntSimpleGlyphPointYReadPushError` は storage owner を持つため `Clone` / `Copy` を実装しない。
- success payload には cursor accessor と storage owner accessor を追加する。
- y-only internal helper は bounded flag reads と `gui_sfnt_glyf_decode_y_delta` だけを使う。
- `gui_sfnt_glyf_decode_x_delta`、full point decode state、endpoint read、contour span helper、public lookup wrapper、direct `Vec`、render/raster/platform/host API は使わない。
- forged bad x range は F5j では検査しない。PointX / full point phase の責務として document する。
- read failure では F5i push を呼ばず、point は `None`、parse error は `Some`、storage len は変更しない。
- read success では `GuiSfntSimpleGlyphPointYSlot` を作り、F5i `gui_sfnt_simple_glyph_outline_storage_push_point_y` を 1 回だけ呼ぶ。
- F5i push failure では rejected point、F5i error kind、F5d region error kind、F5c storage push error kind を owner 消費前に読む。
- doctest は endpoint 2 slots と PointX 4 slots を先に埋めてから PointY read/push success、read failure owner recovery、push failure point preservation を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- bad x range だが valid y range を持つ forged stream から PointY point 0 と point 1 を追加でき、storage len が 8、cursor next index が 8 になる。
- y byte range が壊れた stream では `ReadFailed` になり、point は `None`、parse error は `Some`、storage len が 6 のまま回収できる。
- valid y read だが cursor が logical point 0 を指す状態で point_index 1 を push すると `PushFailed` になり、point は `Some`、lower F5i error kind が `Some PointIndexMismatch` になる。
- source policy が y-only allowlist、read-before-mutate、F5i push 呼び出し回数、lower error metadata の owner 消費前読み取り、direct `vec::` 禁止、full point / endpoint / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_y.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_y_f5j.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5j.json -j 1
git diff --check
```

## Phase F5k: sfnt simple glyph outline point coordinate read

目的:

- F5b-F5j で population 済みの `PointX` / `PointY` scalar slot から、1 logical point の coordinate pair を read-only に取得する。
- `GuiSfntSimpleGlyphPoint` の `on_curve` / `end_of_contour` はまだ F5 storage に存在しないため、この phase では full point value を返さない。
- storage readiness、slot boundary、typed error を固定し、fallback coordinate、byte decode 再実行、renderer/rasterizer/platform 依存へ進まない。

変更:

- 先に source policy を追加し、F5k docs、private scalar getter、coordinate value、typed read error、validation order、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointCoordinate`
  - `GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointCoordinateReadError`
  - private `gui_sfnt_simple_glyph_outline_storage_scalar_slot_get`
  - `gui_sfnt_simple_glyph_outline_storage_read_point_coordinate`
- raw scalar slot getter は private にし、`vec::get` をここへ閉じ込める。unchecked public slot accessor は作らない。
- public read helper は storage owner を borrow し、storage を mutate しない。
- validation は次の順序で行う。
  - capacity shape
  - scalar slot count `Fits`
  - `storage.scalar_slot_count == expected`
  - `scalar_slots_cap == storage.scalar_slot_count`
  - `0 <= point_index < point_count`
  - `scalar_slots_len > y_slot_index`
  - private getter で x slot と y slot を読む
- `scalar_slots_len <= y_slot_index` は `CoordinateNotReady` として扱う。`scalar_slots_len > y_slot_index` で readiness が確認された後に private getter が `None` を返した場合は `ScalarSlotMissing` とする。
- `GuiSfntSimpleGlyphOutlinePointCoordinate` と read error は value-only なので `Clone` / `Copy` を実装してよい。
- doctest は既存 owner-preserving push API で endpoint、PointX、PointY を順に埋め、success、out-of-range、missing PointY readiness を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- 2 contours / 4 points の storage に endpoint 2 slots、PointX 4 slots、PointY 4 slots を追加した後、point 0 と point 1 の coordinate pair を読める。
- `point_index == point_count` は `PointIndexOutOfRange` になる。
- endpoint と PointX までしか埋まっていない storage では `CoordinateNotReady` になり、zero coordinate や byte decode fallback を返さない。
- source policy が F5k docs、value/error 型、private `vec::get` helper、public helper validation order、direct `vec::` 禁止、byte/full point/endpoint/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_coordinate.n.md --no-tree -o tmp_gui_font_outline_point_coordinate_f5k.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5k.json -j 1
git diff --check
```

## Phase F5l: sfnt simple glyph outline point endpoint marker read

目的:

- F5e/F5f で population 済みの `ContourEndpoint` scalar region から、1 logical point が属する contour と end-of-contour marker を read-only に取得する。
- endpoint topology 全体を検査してから成功し、partial success や hidden fallback を作らない。
- flag byte、x/y coordinate、full point value、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5l docs、endpoint marker value、typed read error、全 endpoint scan、final endpoint check、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointEndpointMarker`
  - `GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError`
  - private scan helper
  - `gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker`
- public read helper は storage owner を borrow し、storage を mutate しない。
- validation は次の順序で行う。
  - capacity shape
  - scalar slot count `Fits`
  - `storage.scalar_slot_count == expected`
  - `scalar_slots_cap == storage.scalar_slot_count`
  - `0 <= point_index < point_count`
  - `scalar_slots_len >= contour_count`
  - private getter で endpoint slot を contour 0 から final contour まで順に読む
- scan helper は `found` state を持ち、最初に `point_index <= endpoint` になった contour / end flag を記録する。ただしそこで成功を返さず、final contour まで endpoint range、strict increase、final endpoint `point_count - 1` を検査する。
- read helper は direct `Vec` API を呼ばず、F5k の private scalar slot getter を再利用する。
- `GuiSfntSimpleGlyphOutlinePointEndpointMarker` と read error は value-only なので `Clone` / `Copy` を実装してよい。
- doctest は既存 owner-preserving endpoint push API で success、out-of-range、not-ready を検査し、direct region push で forged `[1, 2]` endpoint topology を作って `EndpointTopologyInvalid` を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- endpoint `[1, 3]` の 2 contours / 4 points storage から、point 0/1/2/3 の contour index と end-of-contour marker を読める。
- `point_index == point_count` は `PointIndexOutOfRange` になる。
- endpoint region が空の storage では `EndpointNotReady` になる。
- forged endpoint `[1, 2]` では point 0 でも success にならず、`EndpointTopologyInvalid` になる。
- source policy が F5l docs、value/error 型、全 endpoint scan、final endpoint `point_count - 1` before success、direct `vec::` 禁止、byte/full point/coordinate/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_endpoint.n.md --no-tree -o tmp_gui_font_outline_point_endpoint_f5l.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5l.json -j 1
git diff --check
```
