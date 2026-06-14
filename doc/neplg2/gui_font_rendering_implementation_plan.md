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

## Phase F5: outline, shaping, ruby, vertical, math bridge

目的:

- glyph outline / mask、GSUB/GPOS、縦書き、ruby、math inline bridge を段階的に実装する。

注意:

- この phase では public contract を変えず、F1/F2 で固定した型に実装を接続する。
- 未対応 feature は typed unsupported として返す。
