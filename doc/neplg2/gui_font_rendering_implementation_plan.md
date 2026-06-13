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

## Phase F5: outline, shaping, ruby, vertical, math bridge

目的:

- glyph outline / mask、GSUB/GPOS、縦書き、ruby、math inline bridge を段階的に実装する。

注意:

- この phase では public contract を変えず、F1/F2 で固定した型に実装を接続する。
- 未対応 feature は typed unsupported として返す。
