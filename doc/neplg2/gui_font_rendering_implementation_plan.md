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

## Phase F5: outline, shaping, ruby, vertical, math bridge

目的:

- glyph outline / mask、GSUB/GPOS、縦書き、ruby、math inline bridge を段階的に実装する。

注意:

- この phase では public contract を変えず、F1/F2 で固定した型に実装を接続する。
- 未対応 feature は typed unsupported として返す。
