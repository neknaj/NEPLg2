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

## Phase F4: sfnt metadata parser

目的:

- TTF / OTF / TTC / OTC の table directory と basic metrics を decode する。

変更:

- `alloc/gui/font/sfnt.nepl` と basic table parser を追加する。
- Invalid table directory、unsupported table、collection face index error を typed error として扱う。
- Headless/offscreen tests が explicit fixture bytes を使えるようにする。

完了条件:

- HackGen の face 0 から units per em、ascent、descent、line gap、name table の代表値を取得できる。
- Missing table や invalid face index は fallback せず error になる。

## Phase F5: outline, shaping, ruby, vertical, math bridge

目的:

- glyph outline / mask、GSUB/GPOS、縦書き、ruby、math inline bridge を段階的に実装する。

注意:

- この phase では public contract を変えず、F1/F2 で固定した型に実装を接続する。
- 未対応 feature は typed unsupported として返す。
