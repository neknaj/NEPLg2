#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function withoutComments(text) {
    return text
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//"))
        .join("\n");
}

function functionSlice(source, name) {
    const start = source.indexOf(`fn ${name} `);
    if (start < 0) {
        return "";
    }
    const nextFn = source.indexOf("\nfn ", start + 1);
    const nextPubFn = source.indexOf("\npub fn ", start + 1);
    const candidates = [nextFn, nextPubFn].filter((index) => index >= 0);
    const next = candidates.length === 0 ? -1 : Math.min(...candidates);
    return next < 0 ? source.slice(start) : source.slice(start, next);
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertMatch(text, pattern, message) {
    assert(pattern.test(text), `${message}: expected ${pattern}`);
}

function assertNoMatch(text, pattern, message) {
    assert(!pattern.test(text), `${message}: forbidden ${pattern}`);
}

const spec = read("doc/neplg2/gui_font_rendering_spec.md");
const detailedDesign = read("doc/neplg2/gui_font_rendering_detailed_design.md");
const implementationPlan = read("doc/neplg2/gui_font_rendering_implementation_plan.md");
const redesignPlan = read("doc/neplg2/gui_redesign_implementation_plan.md");

const coreFont = read("stdlib/core/gui/font.nepl");
const coreFontImpl = withoutComments(coreFont);
const renderStyle = read("stdlib/core/gui/render_style.nepl");
const renderStyleImpl = withoutComments(renderStyle);
const fontResource = read("stdlib/std/gui/font_resource.nepl");
const fontResourceImpl = withoutComments(fontResource);
const allocGuiFacade = read("stdlib/alloc/gui.nepl");
const allocFontFacade = read("stdlib/alloc/gui/font.nepl");
const allocFontSfntFacade = read("stdlib/alloc/gui/font/sfnt.nepl");
const allocFontSfntMetadata = read("stdlib/alloc/gui/font/sfnt/metadata.nepl");
const allocFontSfntName = read("stdlib/alloc/gui/font/sfnt/name.nepl");
const allocFontSfntCmap = read("stdlib/alloc/gui/font/sfnt/cmap.nepl");
const allocFontSfntHmtx = read("stdlib/alloc/gui/font/sfnt/hmtx.nepl");
const allocFontSfntGlyf = read("stdlib/alloc/gui/font/sfnt/glyf.nepl");
const allocFontSfnt = [allocFontSfntFacade, allocFontSfntMetadata, allocFontSfntName, allocFontSfntCmap, allocFontSfntHmtx, allocFontSfntGlyf].join("\n");
const allocFontSfntImpl = withoutComments(allocFontSfnt);
const allocFontSfntMetadataImpl = withoutComments(allocFontSfntMetadata);
const allocFontSfntNameImpl = withoutComments(allocFontSfntName);
const allocFontSfntCmapImpl = withoutComments(allocFontSfntCmap);
const allocFontSfntHmtxImpl = withoutComments(allocFontSfntHmtx);
const allocFontSfntGlyfImpl = withoutComments(allocFontSfntGlyf);
const coreGuiFacade = read("stdlib/core/gui.nepl");
const coreGuiPrelude = read("stdlib/core/gui/prelude.nepl");
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiCoreTests = read("tests/stdlib/gui_core.n.md");
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const guiFontSfntTests = [
    read("tests/stdlib/gui_font_sfnt.n.md"),
    read("tests/stdlib/gui_font_sfnt_glyf.n.md"),
].join("\n");
const webFontResourceVfs = read("web/src/gui-font/font-resource-vfs.ts");
const webMain = read("web/src/main.ts");
const webPanelManager = read("web/src/workspace/panel-manager.ts");
const webTerminal = read("web/src/terminal/terminal.ts");
const webShell = read("web/src/terminal/shell.ts");
const webVfs = read("web/src/runtime/vfs.ts");
const webIndex = read("web/index.html");
const webFontResourceBehaviorTest = read("nodesrc/test_web_gui_font_resource_vfs_behavior.js");

assertMatch(
    spec,
    /GuiGlyphPaint:[\s\S]*shadows\s+GuiShadowRef[\s\S]*GuiShadowRef:[\s\S]*NoShadow[\s\S]*SingleShadow\s+GuiShadow[\s\S]*ShadowRun\s+GuiShadowRunId/,
    "font spec must use GuiShadowRef for no_alloc and alloc-backed multi-shadow contract",
);
assertMatch(
    detailedDesign,
    /F2 の `GuiFontResourceRequest` constructor は request shape だけを検査する[\s\S]*face_count[\s\S]*F4/,
    "font detailed design must defer collection face_count validation until parser or registry phase",
);
assertMatch(
    implementationPlan,
    /GuiFontResourcePath[\s\S]*GuiResourceHash[\s\S]*F2 は request shape だけを検査する/,
    "font implementation plan must use typed path/hash and keep F2 validation narrow",
);
assertMatch(
    spec,
    /Formal font renderer[\s\S]*MockTextMeasurer[\s\S]*fallback[\s\S]*してはならない/,
    "font spec must quarantine fixed-cell measurers away from formal font renderer fallback",
);
assertMatch(
    spec,
    /canonical 表記[\s\S]*fonts\/HackGenConsoleNF-Regular\.ttf[\s\S]*\/fonts\/HackGenConsoleNF-Regular\.ttf[\s\S]*suffix match[\s\S]*authority として使ってはならない/,
    "font spec must separate canonical resource path from Web VFS path and forbid suffix authority",
);
assertMatch(
    redesignPlan,
    /Phase 7:[\s\S]*font and 2D renderer contract slice[\s\S]*GuiShadowRef[\s\S]*resource hash と path は専用 value/,
    "GUI redesign plan must route font and render style work through typed Phase 7 contracts",
);
assertMatch(
    detailedDesign,
    /Web Playground は startup で mount promise を開始し[\s\S]*`neplg2 run`[\s\S]*失敗した場合は `GuiFontResourceMountError`[\s\S]*Compile-only path は runtime font bytes を必要としないため mount 完了を待たない/,
    "font detailed design must define Web run-time mount waiting without blocking compile-only path",
);
assertMatch(
    detailedDesign,
    /Native では packaged resource directory または configured resource root[\s\S]*suffix scan[\s\S]*Bare では embedded blob table[\s\S]*未設定の環境では filesystem probing を行わず unsupported/,
    "font detailed design must define Native resource root and Bare embedded blob behavior",
);
assertMatch(
    implementationPlan,
    /Phase F3:[\s\S]*canonical resource path `fonts\/HackGenConsoleNF-Regular\.ttf`[\s\S]*`web\/src\/gui-font\/font-resource-vfs\.ts`[\s\S]*HackGen 専用 API[\s\S]*binary\/read-only file の compile overlay 混入を禁止/,
    "font implementation plan must include F3 bundled resource routing and source policy scope",
);
assertMatch(
    implementationPlan,
    /Phase F4a:[\s\S]*numeric basic metrics[\s\S]*Invalid table directory、invalid table offset、unsupported container、collection face index error[\s\S]*未解析の extra table は error にせず無視する[\s\S]*Phase F4b:[\s\S]*name table/,
    "font implementation plan must split F4a numeric metrics from F4b name-table decoding policy",
);
assertMatch(
    spec,
    /SFNT representative names[\s\S]*nameID 1[\s\S]*nameID 2[\s\S]*nameID 4[\s\S]*platformID 3, encodingID 1, languageID 0x0409[\s\S]*UnsupportedNameEncoding[\s\S]*UnsupportedNameCharacter/,
    "font spec must define exact SFNT representative name priority and typed name errors",
);
assertMatch(
    detailedDesign,
    /SFNT name table[\s\S]*gui_sfnt_parse_metadata[\s\S]*gui_sfnt_parse_names[\s\S]*rank 400[\s\S]*rank 300[\s\S]*rank 200[\s\S]*rank 100/,
    "font detailed design must separate metadata and name parsers with deterministic name record ranking",
);
assertMatch(
    implementationPlan,
    /Phase F4b:[\s\S]*GuiSfntNameEncodingKind[\s\S]*GuiSfntNameRecord[\s\S]*GuiSfntNameSelection[\s\S]*GuiSfntNames[\s\S]*UnsupportedNameTableFormat[\s\S]*MalformedNameRecord[\s\S]*UnsupportedNameEncoding[\s\S]*UnsupportedNameCharacter/,
    "font implementation plan must define F4b name parser data types and error kinds",
);
assertMatch(
    spec,
    /SFNT cmap glyph mapping[\s\S]*gui_sfnt_lookup_glyph_id:[\s\S]*Result GuiGlyphId GuiSfntParseError[\s\S]*platformID 3 \/ encodingID 1[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MissingGlyphMapping/,
    "font spec must define F4c cmap lookup as typed GuiGlyphId with exact error policy",
);
assertMatch(
    spec,
    /SFNT horizontal metrics[\s\S]*GuiSfntHorizontalMetric:[\s\S]*glyph GuiGlyphId[\s\S]*advance_width i32[\s\S]*left_side_bearing i32[\s\S]*gui_sfnt_lookup_horizontal_metric:[\s\S]*Result GuiSfntHorizontalMetric GuiSfntParseError[\s\S]*MissingGlyphMetric[\s\S]*hhea\.length >= 36[\s\S]*MalformedHmtxRecord[\s\S]*numberOfHMetrics \* 4 \+ \(numGlyphs - numberOfHMetrics\) \* 2/,
    "font spec must define F4d hmtx lookup as typed horizontal metrics with exact bounds policy",
);
assertMatch(
    spec,
    /SFNT glyph header bounds[\s\S]*GuiSfntGlyphBounds:[\s\S]*glyph GuiGlyphId[\s\S]*x_min i32[\s\S]*y_min i32[\s\S]*x_max i32[\s\S]*y_max i32[\s\S]*gui_sfnt_lookup_glyph_bounds:[\s\S]*Result GuiSfntGlyphBounds GuiSfntParseError[\s\S]*head\.length >= 52[\s\S]*UnsupportedLocaFormat[\s\S]*MissingGlyphOutline[\s\S]*MalformedGlyfRecord/,
    "font spec must define F4e loca/glyf lookup as typed glyph bounds with exact bounds policy",
);
assertMatch(
    spec,
    /SFNT simple glyph topology[\s\S]*GuiSfntSimpleGlyphTopology:[\s\S]*contour_count i32[\s\S]*point_count i32[\s\S]*instruction_length i32[\s\S]*point_data_offset i32[\s\S]*point_data_length i32[\s\S]*gui_sfnt_lookup_simple_glyph_topology:[\s\S]*Result GuiSfntSimpleGlyphTopology GuiSfntParseError[\s\S]*point_data_offset[\s\S]*glyf` table-relative[\s\S]*UnsupportedGlyphOutlineFormat[\s\S]*MissingGlyphOutline[\s\S]*MalformedGlyfRecord/,
    "font spec must define F4f simple glyph topology with table-relative point data ranges and typed errors",
);
assertMatch(
    spec,
    /SFNT simple glyph point stream[\s\S]*GuiSfntSimpleGlyphPointStream:[\s\S]*flag_data_offset i32[\s\S]*flag_data_length i32[\s\S]*x_data_offset i32[\s\S]*x_data_length i32[\s\S]*y_data_offset i32[\s\S]*y_data_length i32[\s\S]*trailing_data_offset i32[\s\S]*trailing_data_length i32[\s\S]*gui_sfnt_lookup_simple_glyph_point_stream:[\s\S]*Result GuiSfntSimpleGlyphPointStream GuiSfntParseError[\s\S]*flag_data_length[\s\S]*raw flag stream[\s\S]*repeat_count = 0[\s\S]*x_data_offset = flag_data_offset \+ flag_data_length[\s\S]*trailing_data_length < 0[\s\S]*MalformedGlyfRecord/,
    "font spec must define F4g simple glyph point stream ranges, raw repeat semantics, and trailing data policy",
);
assertMatch(
    spec,
    /SFNT simple glyph single point decode[\s\S]*GuiSfntSimpleGlyphPoint:[\s\S]*glyph GuiGlyphId[\s\S]*point_index i32[\s\S]*x i32[\s\S]*y i32[\s\S]*on_curve bool[\s\S]*end_of_contour bool[\s\S]*gui_sfnt_lookup_simple_glyph_point:[\s\S]*Result GuiSfntSimpleGlyphPoint GuiSfntParseError[\s\S]*point_index < 0[\s\S]*MissingGlyphOutline[\s\S]*flag repeat byte[\s\S]*MalformedGlyfRecord[\s\S]*F4g[\s\S]*x_delta = i16be[\s\S]*end_of_contour/,
    "font spec must define F4h single point decode contract, typed errors, F4g dependency, and endpoint state",
);
assertMatch(
    spec,
    /SFNT simple glyph contour span lookup[\s\S]*GuiSfntSimpleGlyphContourSpan:[\s\S]*glyph GuiGlyphId[\s\S]*contour_index i32[\s\S]*start_point_index i32[\s\S]*end_point_index i32[\s\S]*point_count i32[\s\S]*gui_sfnt_lookup_simple_glyph_contour_span:[\s\S]*Result GuiSfntSimpleGlyphContourSpan GuiSfntParseError[\s\S]*end_point_index[\s\S]*inclusive endpoint[\s\S]*point_count[\s\S]*end_point_index - start_point_index \+ 1[\s\S]*F4f[\s\S]*F4g[\s\S]*F4h[\s\S]*contour_index < 0[\s\S]*MissingGlyphOutline/,
    "font spec must define F4i contour span contract, inclusive endpoint semantics, F4f-only dependency, and typed errors",
);
assertMatch(
    detailedDesign,
    /SFNT cmap table[\s\S]*GuiSfntCmapSubtableRecord[\s\S]*WindowsUnicodeBmpFormat4[\s\S]*idRangeOffset[\s\S]*MissingGlyphMapping/,
    "font detailed design must define F4c cmap format-4 lookup and bounds validation",
);
assertMatch(
    detailedDesign,
    /SFNT horizontal metrics table[\s\S]*GuiSfntHorizontalMetric[\s\S]*numberOfHMetrics[\s\S]*hhea\.length >= 36[\s\S]*valid public metric lookup range is `1 <= glyphRaw < numGlyphs`[\s\S]*hmtx\.length[\s\S]*leftSideBearing/,
    "font detailed design must define F4d hmtx count, glyph range, and table-relative lookup",
);
assertMatch(
    detailedDesign,
    /SFNT glyph header bounds[\s\S]*GuiSfntGlyphBounds[\s\S]*indexToLocFormat[\s\S]*head\.length >= 52[\s\S]*UnsupportedLocaFormat[\s\S]*loca\.length[\s\S]*MissingGlyphOutline[\s\S]*glyf glyph header/,
    "font detailed design must define F4e loca format, declared lengths, and glyf header lookup",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph topology[\s\S]*GuiSfntSimpleGlyphTopology[\s\S]*point_data_offset[\s\S]*relative to the `glyf` table[\s\S]*endPtsOfContours[\s\S]*instructionLength[\s\S]*UnsupportedGlyphOutlineFormat[\s\S]*point_data_length/,
    "font detailed design must define F4f simple glyph topology layout and range policy",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph point stream[\s\S]*GuiSfntSimpleGlyphPointStream[\s\S]*flag_data_length[\s\S]*raw consumed flag stream length[\s\S]*Flag scan state[\s\S]*Repeat semantics[\s\S]*A flag byte always contributes one logical point[\s\S]*repeat_count = 0[\s\S]*Coordinate byte length[\s\S]*xShort == 1[\s\S]*x bytes = 1[\s\S]*xShort == 0 and xSame == 1[\s\S]*x bytes = 0[\s\S]*xShort == 0 and xSame == 0[\s\S]*x bytes = 2[\s\S]*trailing_data_length < 0[\s\S]*MalformedGlyfRecord/,
    "font detailed design must define F4g flag scan, repeat handling, coordinate byte formula, and trailing data contract",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph single point decode[\s\S]*GuiSfntSimpleGlyphPoint:[\s\S]*parse metadata[\s\S]*gui_sfnt_glyf_simple_point_stream_with_tables[\s\S]*validate point_index[\s\S]*point_index < 0[\s\S]*MissingGlyphOutline[\s\S]*MalformedGlyfRecord[\s\S]*repeat byte[\s\S]*logical_index = 0[\s\S]*current_x = 0[\s\S]*xShort and xPositive[\s\S]*endpoint_array_offset = point_data_offset - instruction_length - 2 - endpoint_array_length[\s\S]*must not consume trailing bytes/i,
    "font detailed design must define F4h point decode flow, cursor semantics, delta formulas, and endpoint offset",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph contour span lookup[\s\S]*GuiSfntSimpleGlyphContourSpan:[\s\S]*start_point_index i32[\s\S]*end_point_index i32[\s\S]*point_count i32[\s\S]*gui_sfnt_lookup_simple_glyph_contour_span:[\s\S]*Result GuiSfntSimpleGlyphContourSpan GuiSfntParseError[\s\S]*gui_sfnt_glyf_simple_topology_with_tables[\s\S]*must not call `gui_sfnt_glyf_simple_point_stream_with_tables`[\s\S]*endpoint_array_offset = point_data_offset - instruction_length - 2 - endpoint_array_length[\s\S]*point_count = end_point_index - start_point_index \+ 1[\s\S]*MissingGlyphOutline[\s\S]*MalformedGlyfRecord/,
    "font detailed design must define F4i contour span flow, F4f-only dependency, endpoint offset, inclusive range, and typed errors",
);
assertMatch(
    implementationPlan,
    /Phase F4c:[\s\S]*alloc\/gui\/font\/sfnt\/cmap\.nepl[\s\S]*Result GuiGlyphId GuiSfntParseError[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MalformedCmapRecord[\s\S]*MissingGlyphMapping|Phase F4c:[\s\S]*alloc\/gui\/font\/sfnt\/cmap\.nepl[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MalformedCmapRecord[\s\S]*MissingGlyphMapping[\s\S]*Result GuiGlyphId GuiSfntParseError/,
    "font implementation plan must define F4c cmap parser data types and error kinds",
);
assertMatch(
    implementationPlan,
    /Phase F4d:[\s\S]*alloc\/gui\/font\/sfnt\/hmtx\.nepl[\s\S]*MalformedHmtxRecord[\s\S]*MissingGlyphMetric[\s\S]*GuiSfntHorizontalMetric[\s\S]*Result GuiSfntHorizontalMetric GuiSfntParseError[\s\S]*hhea\.offset \+ 34[\s\S]*declared `hmtx\.length`/,
    "font implementation plan must define F4d hmtx parser data types and error kinds",
);
assertMatch(
    implementationPlan,
    /Phase F4e:[\s\S]*alloc\/gui\/font\/sfnt\/glyf\.nepl[\s\S]*UnsupportedLocaFormat[\s\S]*MalformedGlyfRecord[\s\S]*MissingGlyphOutline[\s\S]*GuiSfntGlyphBounds[\s\S]*Result GuiSfntGlyphBounds GuiSfntParseError[\s\S]*head\.offset \+ 50[\s\S]*declared `loca\.length`/,
    "font implementation plan must define F4e glyf parser data types and error kinds",
);
assertMatch(
    implementationPlan,
    /Phase F4f:[\s\S]*UnsupportedGlyphOutlineFormat[\s\S]*GuiSfntSimpleGlyphTopology[\s\S]*gui_sfnt_lookup_simple_glyph_topology[\s\S]*point_data_offset[\s\S]*`glyf` table-relative[\s\S]*endpoint[\s\S]*instructionLength/,
    "font implementation plan must define F4f simple topology parser data types and exact range policy",
);
assertMatch(
    implementationPlan,
    /Phase F4g:[\s\S]*GuiSfntSimpleGlyphPointStream[\s\S]*gui_sfnt_lookup_simple_glyph_point_stream[\s\S]*flag_data_offset = topology\.point_data_offset[\s\S]*flag_data_length[\s\S]*raw consumed flag stream length[\s\S]*repeat_count = 0[\s\S]*x_data_offset = flag_data_offset \+ flag_data_length[\s\S]*trailing_data_length < 0[\s\S]*MalformedGlyfRecord[\s\S]*repeat overrun[\s\S]*missing repeat byte[\s\S]*x coordinate overrun[\s\S]*y coordinate overrun/,
    "font implementation plan must define F4g point stream parser data types, offsets, typed errors, and doctest coverage",
);
assertMatch(
    implementationPlan,
    /Phase F4h:[\s\S]*GuiSfntSimpleGlyphPoint[\s\S]*gui_sfnt_lookup_simple_glyph_point[\s\S]*point_index < 0[\s\S]*MissingGlyphOutline[\s\S]*gui_sfnt_glyf_simple_point_stream_with_tables[\s\S]*F4g-derived[\s\S]*coordinate は point 0 から `point_index` まで累積[\s\S]*trailing_data_length[\s\S]*Source policy[\s\S]*no Vec allocation[\s\S]*coordinate overrun 系 fixture/,
    "font implementation plan must define F4h point decode implementation, source policy gates, and doctest coverage",
);
assertMatch(
    implementationPlan,
    /Phase F4i:[\s\S]*GuiSfntSimpleGlyphContourSpan[\s\S]*gui_sfnt_lookup_simple_glyph_contour_span[\s\S]*end_point_index[\s\S]*inclusive endpoint[\s\S]*point_count = end_point_index - start_point_index \+ 1[\s\S]*contour_index < 0[\s\S]*MissingGlyphOutline[\s\S]*MalformedGlyfRecord[\s\S]*gui_sfnt_glyf_simple_topology_with_tables[\s\S]*gui_sfnt_glyf_simple_point_stream_with_tables[\s\S]*Source policy[\s\S]*F4g\/F4h 非依存[\s\S]*two-contour fixture/,
    "font implementation plan must define F4i contour span implementation, source policy gates, and doctest coverage",
);

assertMatch(
    coreFontImpl,
    /pub\s+enum\s+GuiWritingMode:[\s\S]*HorizontalLtr[\s\S]*HorizontalRtl[\s\S]*VerticalRl[\s\S]*VerticalLr/,
    "core/gui/font must expose all required writing modes as enum variants",
);
assertMatch(
    coreFontImpl,
    /pub\s+enum\s+GuiFontErrorKind:[\s\S]*InvalidFontSize[\s\S]*InvalidFaceIndex[\s\S]*FaceIndexRequired[\s\S]*MissingFontResource[\s\S]*UnsupportedFontContainer[\s\S]*MissingGlyph[\s\S]*InvalidGlyphPaint/,
    "core/gui/font must expose font error categories as enum values",
);
assertMatch(
    coreFontImpl,
    /pub\s+fn\s+gui_font_size_result\s+%fn\s+i32\s+fn\s+i32\s+Result\s+GuiFontSize\s+GuiError[\s\S]*gt\s+px_den\s+0[\s\S]*GuiError::InvalidCommand/,
    "core/gui/font must reject invalid font size denominator with Result",
);
assertNoMatch(
    coreFontImpl,
    /^\s*#import\s+"(?:alloc|std|platforms)\//m,
    "core/gui/font must stay no_alloc and not import alloc/std/platforms",
);

assertMatch(
    renderStyleImpl,
    /pub\s+enum\s+GuiShadowRef:[\s\S]*NoShadow[\s\S]*SingleShadow\s+%GuiShadow[\s\S]*ShadowRun\s+%GuiShadowRunId/,
    "core/gui/render_style must use GuiShadowRef instead of Vec in core",
);
assertMatch(
    renderStyleImpl,
    /pub\s+fn\s+gui_glyph_paint_result[\s\S]*and\s+is_none\s+fill\s+is_none\s+stroke[\s\S]*GuiError::InvalidCommand/,
    "core/gui/render_style must reject glyph paint with neither fill nor stroke",
);
assertNoMatch(
    renderStyleImpl,
    /^\s*#import\s+"(?:alloc|std|platforms)\//m,
    "core/gui/render_style must stay no_alloc and not import alloc/std/platforms",
);

assertMatch(
    fontResourceImpl,
    /pub\s+struct\s+GuiFontResourceRequest:[\s\S]*path\s+%GuiFontResourcePath[\s\S]*face_index\s+%Option\s+i32[\s\S]*expected_hash\s+%Option\s+GuiResourceHash[\s\S]*decode_policy\s+%GuiFontDecodePolicy/,
    "std/gui/font_resource must expose typed font resource request fields",
);
assertMatch(
    fontResourceImpl,
    /pub\s+fn\s+gui_font_resource_path_result\s+%fn\s+str\s+Result\s+GuiFontResourcePath\s+GuiError[\s\S]*gt\s+len\s+path\s+0[\s\S]*GuiError::InvalidCommand/,
    "std/gui/font_resource must reject empty resource paths",
);
assertMatch(
    fontResourceImpl,
    /gui_font_face_index_is_valid[\s\S]*Option::Some\s+index:[\s\S]*ge\s+index\s+0/,
    "std/gui/font_resource must reject negative face indexes at request-shape boundary",
);
assertNoMatch(
    fontResourceImpl,
    /\b(?:DOM|Canvas|FontFace|CoreText|DirectWrite|fontconfig|HWND|UIKit|AndroidView|HtmlCanvas)\b/,
    "std/gui/font_resource must not expose concrete font or platform handles",
);
assertNoMatch(
    fontResourceImpl,
    /\b(?:MockTextMeasurer|HostTextMeasurer|host_text_measurer|host_text_measurer_fixed|measure_text)\b/,
    "std/gui/font_resource must not depend on fixed-cell text measurement fallback",
);

assertMatch(allocGuiFacade, /#import\s+"alloc\/gui\/font"\s+as\s+\*/, "alloc/gui facade must export font parser facade");
assertMatch(allocFontFacade, /#import\s+"alloc\/gui\/font\/sfnt"\s+as\s+\*/, "alloc/gui/font facade must export sfnt parser");
assertMatch(allocFontSfntFacade, /#import\s+"\.\/sfnt\/metadata"\s+as\s+@merge/, "alloc/gui/font/sfnt facade must re-export metadata parser");
assertMatch(allocFontSfntFacade, /#import\s+"\.\/sfnt\/name"\s+as\s+@merge/, "alloc/gui/font/sfnt facade must re-export name parser");
assertMatch(allocFontSfntFacade, /#import\s+"\.\/sfnt\/cmap"\s+as\s+@merge/, "alloc/gui/font/sfnt facade must re-export cmap parser");
assertMatch(allocFontSfntFacade, /#import\s+"\.\/sfnt\/hmtx"\s+as\s+@merge/, "alloc/gui/font/sfnt facade must re-export hmtx parser");
assertMatch(allocFontSfntFacade, /#import\s+"\.\/sfnt\/glyf"\s+as\s+@merge/, "alloc/gui/font/sfnt facade must re-export glyf parser");
assertMatch(
    allocFontSfntImpl,
    /pub\s+enum\s+GuiSfntContainerKind:[\s\S]*TrueTypeSfnt[\s\S]*OpenTypeSfnt[\s\S]*TrueTypeCollection[\s\S]*OpenTypeCollection/,
    "alloc/gui/font/sfnt must expose typed container kinds",
);
assertMatch(
    allocFontSfntImpl,
    /pub\s+enum\s+GuiSfntParseErrorKind:[\s\S]*UnexpectedEof[\s\S]*UnsupportedContainer[\s\S]*InvalidTableDirectory[\s\S]*InvalidTableOffset[\s\S]*MissingTable[\s\S]*InvalidFaceIndex[\s\S]*FaceIndexRequired[\s\S]*UnsupportedNameTableFormat[\s\S]*MalformedNameRecord[\s\S]*UnsupportedNameEncoding[\s\S]*UnsupportedNameCharacter[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MalformedCmapRecord[\s\S]*MissingGlyphMapping[\s\S]*MalformedHmtxRecord[\s\S]*MissingGlyphMetric[\s\S]*UnsupportedLocaFormat[\s\S]*MalformedGlyfRecord[\s\S]*MissingGlyphOutline[\s\S]*UnsupportedGlyphOutlineFormat/,
    "alloc/gui/font/sfnt must expose typed parser errors",
);
assertMatch(
    allocFontSfntImpl,
    /pub\s+fn\s+gui_sfnt_parse_metadata\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+Result\s+GuiSfntMetadata\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt parser must take borrowed ByteBuf and return typed Result",
);
assertMatch(
    allocFontSfntImpl,
    /pub\s+struct\s+GuiSfntMetadata:[\s\S]*container_kind\s+%GuiSfntContainerKind[\s\S]*face_index\s+%i32[\s\S]*face_count\s+%i32[\s\S]*directory\s+%GuiSfntDirectory[\s\S]*metrics\s+%GuiSfntMetrics/,
    "alloc/gui/font/sfnt metadata must carry container kind, face selection, directory, and metrics",
);
assertMatch(
    allocFontSfntImpl,
    /pub\s+struct\s+GuiSfntDirectory:[\s\S]*head\s+%Option\s+GuiSfntTableRecord[\s\S]*hhea\s+%Option\s+GuiSfntTableRecord[\s\S]*maxp\s+%Option\s+GuiSfntTableRecord[\s\S]*name\s+%Option\s+GuiSfntTableRecord[\s\S]*cmap\s+%Option\s+GuiSfntTableRecord[\s\S]*hmtx\s+%Option\s+GuiSfntTableRecord[\s\S]*loca\s+%Option\s+GuiSfntTableRecord[\s\S]*glyf\s+%Option\s+GuiSfntTableRecord/,
    "alloc/gui/font/sfnt directory must track optional name, cmap, hmtx, loca, and glyf tables without requiring decoding",
);
assertMatch(
    allocFontSfntImpl,
    /pub\s+struct\s+GuiSfntMetrics:[\s\S]*units_per_em\s+%i32[\s\S]*ascent\s+%i32[\s\S]*descent\s+%i32[\s\S]*line_gap\s+%i32[\s\S]*num_glyphs\s+%i32/,
    "alloc/gui/font/sfnt metrics must expose numeric layout values",
);
assertMatch(
    allocFontSfntImpl,
    /io_bytebuf_byte_at[\s\S]*GuiSfntParseErrorKind::UnexpectedEof/,
    "alloc/gui/font/sfnt must read explicit bytes and return typed eof errors",
);
assertMatch(
    allocFontSfntImpl,
    /GuiSfntParseErrorKind::FaceIndexRequired[\s\S]*GuiSfntParseErrorKind::InvalidFaceIndex/,
    "alloc/gui/font/sfnt must reject invalid collection face selection as typed errors",
);
assertMatch(
    allocFontSfntImpl,
    /fn\s+gui_sfnt_table_record_bounds_are_valid[\s\S]*le\s+length\s+sub\s+total\s+offset/,
    "alloc/gui/font/sfnt must check table offsets with bounded ranges",
);
assertMatch(
    allocFontSfntImpl,
    /fn\s+gui_sfnt_read_u32_i32_be[\s\S]*high_byte_limit\s+%i32\s+add\s+64\s+64[\s\S]*ge\s+b0\s+high_byte_limit[\s\S]*Result::Err\s+gui_sfnt_parse_error\s+kind\s+offset/,
    "alloc/gui/font/sfnt u32-i32 reader must return the caller supplied typed error kind",
);
assertMatch(
    allocFontSfntImpl,
    /gui_sfnt_read_u32_i32_be\s+bytes\s+add\s+record_offset\s+8\s+GuiSfntParseErrorKind::InvalidTableOffset[\s\S]*gui_sfnt_read_u32_i32_be\s+bytes\s+add\s+record_offset\s+12\s+GuiSfntParseErrorKind::InvalidTableOffset/,
    "alloc/gui/font/sfnt table record parser must reject u32 table offsets or lengths outside i32 range",
);
assertMatch(
    allocFontSfntImpl,
    /table_offset_bytes\s+%i32\s+sub\s+total\s+12[\s\S]*max_face_count\s+%i32\s+div_s\s+table_offset_bytes\s+4[\s\S]*gt\s+face_count\s+max_face_count[\s\S]*GuiSfntParseErrorKind::InvalidTableDirectory/,
    "alloc/gui/font/sfnt must bound collection face count before multiplying by face-offset entry size",
);
assertMatch(
    allocFontSfntNameImpl,
    /pub\s+enum\s+GuiSfntNameEncodingKind:[\s\S]*WindowsUnicodeBmpAscii[\s\S]*MacintoshRomanAscii/,
    "alloc/gui/font/sfnt/name must expose supported representative name encodings as enum values",
);
assertMatch(
    allocFontSfntNameImpl,
    /pub\s+struct\s+GuiSfntNameRecord:[\s\S]*platform_id\s+%i32[\s\S]*encoding_id\s+%i32[\s\S]*language_id\s+%i32[\s\S]*name_id\s+%i32[\s\S]*length\s+%i32[\s\S]*offset\s+%i32/,
    "alloc/gui/font/sfnt/name must expose name table records as typed data",
);
assertMatch(
    allocFontSfntNameImpl,
    /pub\s+struct\s+GuiSfntNames:[\s\S]*family\s+%Option\s+str[\s\S]*subfamily\s+%Option\s+str[\s\S]*full_name\s+%Option\s+str/,
    "alloc/gui/font/sfnt/name must expose representative names as Option fields",
);
assertMatch(
    allocFontSfntNameImpl,
    /pub\s+fn\s+gui_sfnt_parse_names\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+Result\s+GuiSfntNames\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/name parser must take borrowed ByteBuf and return typed Result",
);
assertMatch(
    allocFontSfntNameImpl,
    /platform_id\s+3[\s\S]*encoding_id\s+1[\s\S]*language_id\s+1033[\s\S]*WindowsUnicodeBmpAscii[\s\S]*platform_id\s+1[\s\S]*encoding_id\s+0[\s\S]*language_id\s+0[\s\S]*MacintoshRomanAscii/,
    "alloc/gui/font/sfnt/name must implement documented initial encoding policy",
);
assertMatch(
    allocFontSfntNameImpl,
    /records_end\s+%i32\s+add\s+6\s+records_size[\s\S]*lt\s+string_offset\s+records_end[\s\S]*GuiSfntParseErrorKind::MalformedNameRecord/,
    "alloc/gui/font/sfnt/name must reject name string storage that overlaps the record array",
);
assertMatch(
    allocFontSfntCmapImpl,
    /pub\s+enum\s+GuiSfntCmapEncodingKind:[\s\S]*WindowsUnicodeBmpFormat4/,
    "alloc/gui/font/sfnt/cmap must expose the supported cmap encoding as typed data",
);
assertMatch(
    allocFontSfntCmapImpl,
    /pub\s+struct\s+GuiSfntCmapSubtableRecord:[\s\S]*platform_id\s+%i32[\s\S]*encoding_id\s+%i32[\s\S]*offset\s+%i32/,
    "alloc/gui/font/sfnt/cmap must expose cmap subtable records as typed data",
);
assertMatch(
    allocFontSfntCmapImpl,
    /pub\s+fn\s+gui_sfnt_lookup_glyph_id\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+i32\s+Result\s+GuiGlyphId\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/cmap lookup must take borrowed ByteBuf and return typed GuiGlyphId",
);
assertMatch(
    allocFontSfntCmapImpl,
    /gui_sfnt_cmap_subtable_record_selected[\s\S]*platform_id\s+3[\s\S]*encoding_id\s+1/,
    "alloc/gui/font/sfnt/cmap must select only Windows Unicode BMP records in F4c",
);
assertMatch(
    allocFontSfntCmapImpl,
    /not\s+eq\s+subtable_format\s+4[\s\S]*GuiSfntParseErrorKind::UnsupportedCmapTableFormat/,
    "alloc/gui/font/sfnt/cmap must reject non-format-4 selected records without switching records",
);
assertMatch(
    allocFontSfntCmapImpl,
    /le\s+raw\s+0[\s\S]*GuiSfntParseErrorKind::MissingGlyphMapping/,
    "alloc/gui/font/sfnt/cmap must reject glyph id 0 instead of returning it as success",
);
assertMatch(
    allocFontSfntCmapImpl,
    /id_range_offset_offset[\s\S]*range_offset[\s\S]*mul\s+sub\s+code_point\s+start_code\s+2[\s\S]*gui_sfnt_cmap_file_range_inside_subtable/,
    "alloc/gui/font/sfnt/cmap must compute idRangeOffset target bounds from the idRangeOffset word",
);
assertMatch(
    allocFontSfntCmapImpl,
    /table_length\s+%i32\s+gui_sfnt_table_record_length[\s\S]*lt\s+table_length\s+4[\s\S]*GuiSfntParseErrorKind::MalformedCmapRecord/,
    "alloc/gui/font/sfnt/cmap must reject declared cmap tables shorter than their header",
);
assertMatch(
    allocFontSfntCmapImpl,
    /records_end[\s\S]*lt\s+record_offset\s+records_end[\s\S]*GuiSfntParseErrorKind::MalformedCmapRecord/,
    "alloc/gui/font/sfnt/cmap must reject selected subtable offsets that overlap encoding records",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /pub\s+struct\s+GuiSfntHorizontalMetric:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*advance_width\s+%i32[\s\S]*left_side_bearing\s+%i32/,
    "alloc/gui/font/sfnt/hmtx must expose horizontal metrics as typed data",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /pub\s+fn\s+gui_sfnt_lookup_horizontal_metric\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+Result\s+GuiSfntHorizontalMetric\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/hmtx lookup must take borrowed ByteBuf and checked GuiGlyphId",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /gui_sfnt_hmtx_read_number_of_metrics[\s\S]*lt\s+gui_sfnt_table_record_length\s+&hhea\s+36[\s\S]*add\s+gui_sfnt_table_record_offset\s+&hhea\s+34/,
    "alloc/gui/font/sfnt/hmtx must read hhea.numberOfHMetrics only after hhea length 36",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /gui_sfnt_hmtx_validate_counts[\s\S]*le\s+number_of_hmetrics\s+0[\s\S]*gt\s+number_of_hmetrics\s+num_glyphs[\s\S]*GuiSfntParseErrorKind::MalformedHmtxRecord/,
    "alloc/gui/font/sfnt/hmtx must reject invalid numberOfHMetrics",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /gui_sfnt_hmtx_checked_glyph_raw[\s\S]*le\s+raw\s+0[\s\S]*ge\s+raw\s+num_glyphs[\s\S]*GuiSfntParseErrorKind::MissingGlyphMetric/,
    "alloc/gui/font/sfnt/hmtx must reject glyph 0 and glyphs outside maxp.numGlyphs",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /long_metric_bytes\s+%i32\s+mul\s+number_of_hmetrics\s+4[\s\S]*bearing_bytes\s+%i32\s+mul\s+bearing_count\s+2[\s\S]*le\s+required_length\s+gui_sfnt_table_record_length\s+&hmtx/,
    "alloc/gui/font/sfnt/hmtx must validate declared hmtx length before reading",
);
assertMatch(
    allocFontSfntHmtxImpl,
    /lt\s+glyph_raw\s+number_of_hmetrics[\s\S]*gui_sfnt_hmtx_read_long_metric[\s\S]*gui_sfnt_hmtx_read_bearing_array_metric/,
    "alloc/gui/font/sfnt/hmtx must cover longHorMetric and leftSideBearing-array paths",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntGlyphBounds:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*x_min\s+%i32[\s\S]*y_min\s+%i32[\s\S]*x_max\s+%i32[\s\S]*y_max\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose glyph header bounds as typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_glyph_bounds\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+Result\s+GuiSfntGlyphBounds\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf lookup must take borrowed ByteBuf and checked GuiGlyphId",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphTopology:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*bounds\s+%GuiSfntGlyphBounds[\s\S]*contour_count\s+%i32[\s\S]*point_count\s+%i32[\s\S]*instruction_length\s+%i32[\s\S]*point_data_offset\s+%i32[\s\S]*point_data_length\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose simple glyph topology as typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_topology\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+Result\s+GuiSfntSimpleGlyphTopology\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf simple topology lookup must take borrowed ByteBuf and checked GuiGlyphId",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPointStream:[\s\S]*topology\s+%GuiSfntSimpleGlyphTopology[\s\S]*flag_data_offset\s+%i32[\s\S]*flag_data_length\s+%i32[\s\S]*x_data_offset\s+%i32[\s\S]*x_data_length\s+%i32[\s\S]*y_data_offset\s+%i32[\s\S]*y_data_length\s+%i32[\s\S]*trailing_data_offset\s+%i32[\s\S]*trailing_data_length\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose simple glyph point stream ranges as typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_point_stream\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+Result\s+GuiSfntSimpleGlyphPointStream\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf point stream lookup must take borrowed ByteBuf and checked GuiGlyphId",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPoint:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*point_index\s+%i32[\s\S]*x\s+%i32[\s\S]*y\s+%i32[\s\S]*on_curve\s+%bool[\s\S]*end_of_contour\s+%bool/,
    "alloc/gui/font/sfnt/glyf must expose single decoded simple glyph points as typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_point\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphPoint\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf point lookup must take borrowed ByteBuf, checked GuiGlyphId, and logical point index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphContourSpan:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*contour_index\s+%i32[\s\S]*start_point_index\s+%i32[\s\S]*end_point_index\s+%i32[\s\S]*point_count\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose simple glyph contour spans as typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_contour_span\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphContourSpan\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf contour span lookup must take borrowed ByteBuf, checked GuiGlyphId, and logical contour index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_read_index_to_loc_format[\s\S]*lt\s+gui_sfnt_table_record_length\s+&head\s+52[\s\S]*add\s+gui_sfnt_table_record_offset\s+&head\s+50/,
    "alloc/gui/font/sfnt/glyf must read head.indexToLocFormat only after head length 52",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /not\s+or\s+eq\s+format\s+0\s+eq\s+format\s+1[\s\S]*GuiSfntParseErrorKind::UnsupportedLocaFormat/,
    "alloc/gui/font/sfnt/glyf must reject unsupported loca formats as typed unsupported",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /fn\s+gui_sfnt_glyf_read_u32_i32_be[\s\S]*high_byte_limit\s+%i32\s+add\s+64\s+64[\s\S]*ge\s+b0\s+high_byte_limit[\s\S]*Result::Err\s+gui_sfnt_parse_error\s+kind\s+offset/,
    "alloc/gui/font/sfnt/glyf must reject long loca offsets outside i32 range",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /entry_count\s+%i32\s+add\s+num_glyphs\s+1[\s\S]*eq\s+format\s+0[\s\S]*mul\s+entry_count\s+2[\s\S]*eq\s+format\s+1[\s\S]*mul\s+entry_count\s+4/,
    "alloc/gui/font/sfnt/glyf must validate declared loca length for short and long formats",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_checked_glyph_raw[\s\S]*le\s+raw\s+0[\s\S]*ge\s+raw\s+num_glyphs[\s\S]*GuiSfntParseErrorKind::MissingGlyphOutline/,
    "alloc/gui/font/sfnt/glyf must reject glyph 0 and glyphs outside maxp.numGlyphs",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gt\s+start\s+end[\s\S]*gt\s+end\s+gui_sfnt_table_record_length\s+&glyf[\s\S]*eq\s+start\s+end[\s\S]*MissingGlyphOutline[\s\S]*lt\s+sub\s+end\s+start\s+10/,
    "alloc/gui/font/sfnt/glyf must validate glyf declared bounds and empty/short glyph ranges",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_bounds_from_header[\s\S]*add\s+file_offset\s+2[\s\S]*add\s+file_offset\s+4[\s\S]*add\s+file_offset\s+6[\s\S]*add\s+file_offset\s+8[\s\S]*or\s+gt\s+x_min\s+x_max\s+gt\s+y_min\s+y_max/,
    "alloc/gui/font/sfnt/glyf must read x/y bounds from the glyf header and reject inverted bounds",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_read_last_endpoint[\s\S]*not\s+gui_sfnt_glyf_glyph_relative_range_is_valid\s+start\s+end\s+endpoint_offset\s+2[\s\S]*le\s+endpoint\s+previous_endpoint/,
    "alloc/gui/font/sfnt/glyf must validate simple glyph endpoints inside glyph range and strict increasing",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /lt\s+contour_count\s+0[\s\S]*GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat[\s\S]*eq\s+contour_count\s+0[\s\S]*GuiSfntParseErrorKind::MissingGlyphOutline/,
    "alloc/gui/font/sfnt/glyf must split composite, zero-contour, and malformed simple glyphs into typed errors",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /instruction_length_offset\s+%i32\s+add\s+endpoint_array_offset\s+endpoint_array_length[\s\S]*gui_sfnt_glyf_read_u16_be[\s\S]*instruction_start\s+%i32\s+add\s+instruction_length_offset\s+2[\s\S]*point_data_offset\s+%i32\s+add\s+instruction_start\s+instruction_length[\s\S]*point_data_length\s+%i32\s+sub\s+end\s+point_data_offset[\s\S]*le\s+point_data_length\s+0/,
    "alloc/gui/font/sfnt/glyf must validate instruction range and non-empty point data range",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_scan_flag_stream[\s\S]*eq\s+logical_count\s+point_count[\s\S]*sub\s+cursor\s+flag_start[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+8[\s\S]*repeat_count_offset\s+%i32\s+add\s+cursor\s+1[\s\S]*run_count\s+%i32\s+add\s+repeat_count\s+1[\s\S]*gt\s+next_logical_count\s+point_count[\s\S]*GuiSfntParseErrorKind::MalformedGlyfRecord/,
    "alloc/gui/font/sfnt/glyf must scan raw flag bytes with repeat count semantics and reject repeat overrun",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_flag_x_byte_length[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+2[\s\S]*then:[\s\S]*1[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+16[\s\S]*then:[\s\S]*0[\s\S]*else:[\s\S]*2[\s\S]*gui_sfnt_glyf_flag_y_byte_length[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+4[\s\S]*then:[\s\S]*1[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+32[\s\S]*then:[\s\S]*0[\s\S]*else:[\s\S]*2/,
    "alloc/gui/font/sfnt/glyf must derive x/y coordinate byte lengths from short and same bits",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /flag_data_offset\s+%i32\s+gui_sfnt_simple_glyph_topology_point_data_offset\s+&topology[\s\S]*flag_data_length\s+%i32\s+gui_sfnt_simple_glyph_flag_scan_raw_length\s+&scan[\s\S]*x_data_offset\s+%i32\s+add\s+flag_data_offset\s+flag_data_length[\s\S]*y_data_offset\s+%i32\s+add\s+x_data_offset\s+x_data_length[\s\S]*trailing_data_offset\s+%i32\s+add\s+y_data_offset\s+y_data_length[\s\S]*trailing_data_length\s+%i32\s+sub\s+point_data_end\s+trailing_data_offset[\s\S]*lt\s+trailing_data_length\s+0/,
    "alloc/gui/font/sfnt/glyf must derive point stream offsets and reject coordinate byte overrun",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_simple_point_with_tables[\s\S]*gui_sfnt_glyf_simple_point_stream_with_tables[\s\S]*gui_sfnt_glyf_decode_point_from_stream/,
    "alloc/gui/font/sfnt/glyf point decode must reuse F4g point stream validation",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_read_u8_in_stream_range[\s\S]*gui_sfnt_glyf_stream_relative_range_is_valid[\s\S]*GuiSfntParseErrorKind::MalformedGlyfRecord[\s\S]*gui_sfnt_glyf_read_i16_in_stream_range[\s\S]*gui_sfnt_glyf_stream_relative_range_is_valid/,
    "alloc/gui/font/sfnt/glyf point decode must read flags and coordinates only inside F4g-derived ranges",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_decode_x_delta[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+2[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+16[\s\S]*sub\s+0\s+byte[\s\S]*gui_sfnt_glyf_read_i16_in_stream_range[\s\S]*gui_sfnt_glyf_decode_y_delta[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+4[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+32[\s\S]*sub\s+0\s+byte/,
    "alloc/gui/font/sfnt/glyf point decode must implement short/same signed coordinate delta semantics",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_decode_flag_run_state[\s\S]*current_x[\s\S]*next_x\s+%i32\s+add\s+current_x[\s\S]*next_y\s+%i32\s+add\s+current_y[\s\S]*sub\s+remaining\s+1[\s\S]*gui_sfnt_glyf_decode_point_state_from_flag_run[\s\S]*target_count\s+%i32\s+add\s+sub\s+target_index\s+logical_index\s+1/,
    "alloc/gui/font/sfnt/glyf point decode must accumulate coordinates through repeated flag runs up to the target point",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /or\s+lt\s+point_index\s+0\s+ge\s+point_index\s+point_count[\s\S]*GuiSfntParseErrorKind::MissingGlyphOutline[\s\S]*gui_sfnt_glyf_point_is_contour_end[\s\S]*gui_sfnt_glyf_flag_has_bit\s+flag\s+1/,
    "alloc/gui/font/sfnt/glyf point decode must split out-of-range point requests and derive contour/on-curve state",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_contour_span_from_topology[\s\S]*or\s+lt\s+contour_index\s+0\s+ge\s+contour_index\s+contour_count[\s\S]*GuiSfntParseErrorKind::MissingGlyphOutline[\s\S]*gui_sfnt_glyf_read_contour_endpoint[\s\S]*previous_endpoint[\s\S]*start_point_index\s+%i32\s+add\s+previous_endpoint\s+1[\s\S]*point_count\s+%i32\s+add\s+sub\s+end_point_index\s+start_point_index\s+1/,
    "alloc/gui/font/sfnt/glyf contour span lookup must split out-of-range contour requests and derive inclusive point ranges",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_simple_contour_span_with_tables[\s\S]*gui_sfnt_glyf_simple_topology_with_tables[\s\S]*gui_sfnt_glyf_contour_span_from_topology/,
    "alloc/gui/font/sfnt/glyf contour span lookup must reuse F4f topology validation",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphPoint\b|\bpush\s+.*GuiSfntSimpleGlyphPoint\b/,
    "alloc/gui/font/sfnt/glyf F4h must not allocate or build a full point Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphContourSpan\b|\bpush\s+.*GuiSfntSimpleGlyphContourSpan\b/,
    "alloc/gui/font/sfnt/glyf F4i must not allocate or build a full contour span Vec",
);
const contourSpanWithTables = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_simple_contour_span_with_tables");
assertNoMatch(
    contourSpanWithTables,
    /\bgui_sfnt_glyf_simple_point_stream_with_tables\b|\bgui_sfnt_lookup_simple_glyph_point_stream\b|\bgui_sfnt_lookup_simple_glyph_point\b/,
    "alloc/gui/font/sfnt/glyf F4i table helper must not depend on F4g/F4h point decoding",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_parse_names\b/,
    "gui_sfnt_parse_metadata must remain independent from name table decoding",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_glyph_id\b/,
    "gui_sfnt_parse_metadata must remain independent from cmap glyph lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_horizontal_metric\b/,
    "gui_sfnt_parse_metadata must remain independent from hmtx metric lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_glyph_bounds\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf bounds lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_topology\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf topology lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_point_stream\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf point stream lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_point\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf point lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_contour_span\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf contour span lookup",
);
assertNoMatch(
    allocFontSfntHmtxImpl,
    /\bgui_sfnt_parse_names\b|\bgui_sfnt_lookup_glyph_id\b/,
    "alloc/gui/font/sfnt/hmtx must not use name or cmap parsing as a metric substitute",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bgui_sfnt_parse_names\b|\bgui_sfnt_lookup_glyph_id\b|\bgui_sfnt_lookup_horizontal_metric\b/,
    "alloc/gui/font/sfnt/glyf must not use name, cmap, or hmtx parsing as a bounds substitute",
);
assertNoMatch(
    allocFontSfntImpl,
    /^\s*#import\s+"(?:std\/fs|std\/gui|platforms\/|web\/)/m,
    "alloc/gui/font/sfnt must not import resource loading or platform modules",
);
assertNoMatch(
    allocFontSfnt,
    /\b(?:DOM|Canvas|FontFace|CoreText|DirectWrite|fontconfig|HWND|UIKit|AndroidView|HtmlCanvas)\b/,
    "alloc/gui/font/sfnt must not mention concrete platform font or surface APIs",
);
assertNoMatch(
    allocFontSfnt,
    /\b(?:MockTextMeasurer|HostTextMeasurer|host_text_measurer|host_text_measurer_fixed|measure_text)\b/,
    "alloc/gui/font/sfnt must not depend on host or fixed-cell text measurement",
);
assertNoMatch(
    allocFontSfnt,
    /\b(?:GuiFontResourcePath|GuiFontResourceRequest|suffix match|display name)\b/,
    "alloc/gui/font/sfnt must not use path, suffix, or display-name authority",
);
assertNoMatch(
    allocFontSfnt,
    /\bfallback\b/i,
    "alloc/gui/font/sfnt implementation must not describe hidden fallback behavior",
);
assertMatch(
    guiFontSfntTests,
    /ByteBuilder[\s\S]*sfnt_push_u8[\s\S]*sfnt_push_u32_be/,
    "gui font sfnt doctest must use ByteBuilder explicit byte fixtures",
);
for (const sfntCase of [
    "valid standalone sfnt metrics",
    "truncated header",
    "missing maxp",
    "invalid table offset",
    "high bit table offset",
    "ttc face required",
    "ttc out of range",
    "ttc oversized face count",
    "single face rejects one",
]) {
    assertMatch(
        guiFontSfntTests,
        new RegExp(sfntCase.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
        `gui font sfnt doctest must cover ${sfntCase}`,
    );
}
assertMatch(
    guiFontSfntTests,
    /gui_sfnt_parse_error_kind[\s\S]*GuiSfntParseErrorKind::MissingTable[\s\S]*GuiSfntParseErrorKind::InvalidTableOffset[\s\S]*GuiSfntParseErrorKind::FaceIndexRequired[\s\S]*GuiSfntParseErrorKind::InvalidFaceIndex/,
    "gui font sfnt doctest must match typed parser error kinds",
);
for (const cmapCase of [
    "cmap glyph array A",
    "cmap glyph array zero rejected",
    "cmap glyph array range malformed",
    "cmap subtable overlaps records",
    "short cmap header",
    "short cmap subtable",
]) {
    assertMatch(
        guiFontSfntTests,
        new RegExp(cmapCase.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
        `gui font sfnt doctest must cover ${cmapCase}`,
    );
}
for (const hmtxCase of [
    "hmtx glyph1 advance",
    "hmtx glyph3 lsb",
    "missing hmtx table",
    "short hhea for hmtx",
    "zero hmetrics count",
    "too many hmetrics",
    "hmtx glyph outside maxp",
    "short hmtx declared length",
]) {
    assertMatch(
        guiFontSfntTests,
        new RegExp(hmtxCase.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
        `gui font sfnt doctest must cover ${hmtxCase}`,
    );
}
for (const glyfCase of [
    "glyf glyph1 x min",
    "glyf glyph1 y max",
    "glyf long loca x max",
    "topology contour count",
    "topology point count",
    "topology instruction length",
    "topology point data offset",
    "topology point data length",
    "point stream no-repeat flag offset",
    "point stream no-repeat raw flag length",
    "point stream no-repeat x offset",
    "point stream no-repeat x length",
    "point stream no-repeat y offset",
    "point stream no-repeat y length",
    "point stream no-repeat trailing offset",
    "point stream no-repeat trailing length",
    "point stream repeat raw flag length",
    "point stream repeat x offset",
    "point stream repeat x length",
    "point stream repeat y length",
    "point stream repeat trailing length",
    "point stream repeat zero raw flag length",
    "point stream repeat zero x offset",
    "point stream repeat zero x length",
    "point stream repeat zero y length",
    "point stream repeat zero trailing length",
    "point decode no-repeat point0 x",
    "point decode no-repeat point0 y",
    "point decode no-repeat point0 off curve",
    "point decode no-repeat point0 not contour end",
    "point decode no-repeat endpoint index",
    "point decode no-repeat endpoint contour end",
    "point decode repeat cumulative x",
    "point decode repeat cumulative y",
    "point decode repeat off curve",
    "point decode repeat middle not contour end",
    "point decode repeat zero x",
    "point decode repeat zero y",
    "point decode repeat zero contour end",
    "point decode signed x",
    "point decode signed y",
    "point decode signed on curve",
    "contour span first start",
    "contour span first end",
    "contour span first count",
    "contour span second start",
    "contour span second end",
    "contour span second count",
    "contour span single start",
    "contour span single end",
    "contour span single count",
    "missing loca table",
    "missing glyf table",
    "short head for glyf",
    "unsupported loca format",
    "long loca high bit",
    "short loca declared length",
    "decreasing glyph offset",
    "empty glyph outline",
    "short glyf header",
    "inverted glyph bounds",
    "composite glyph unsupported",
    "zero contour topology",
    "non increasing endpoint",
    "short endpoint array",
    "short instruction length",
    "instruction overrun",
    "missing point data",
    "point stream repeat overrun",
    "point stream missing repeat byte",
    "point stream x coordinate overrun",
    "point stream y coordinate overrun",
    "point decode negative index missing",
    "point decode index count missing",
    "point decode x coordinate overrun",
    "point decode y coordinate overrun",
    "contour span negative index missing",
    "contour span index count missing",
    "contour span malformed endpoint observed",
]) {
    assertMatch(
        guiFontSfntTests,
        new RegExp(glyfCase.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
        `gui font sfnt doctest must cover ${glyfCase}`,
    );
}

assertMatch(coreGuiFacade, /#import\s+"\.\/gui\/font"\s+as\s+@merge/, "core/gui facade must export font contract");
assertMatch(coreGuiFacade, /#import\s+"\.\/gui\/render_style"\s+as\s+@merge/, "core/gui facade must export render style contract");
assertMatch(coreGuiPrelude, /#import\s+"\.\/font"\s+as\s+@merge/, "core/gui prelude must export font contract");
assertMatch(coreGuiPrelude, /#import\s+"\.\/render_style"\s+as\s+@merge/, "core/gui prelude must export render style contract");
assertMatch(stdGuiFacade, /#import\s+"\.\/gui\/font_resource"\s+as\s+\*/, "std/gui facade must export font resource boundary");
assertMatch(guiCoreTests, /gui_core_font_metrics_and_glyph_paint_contract/, "gui_core doctests must cover font metrics and glyph paint");
assertMatch(guiStdTests, /gui_font_resource_request_is_typed_boundary/, "gui_std doctests must cover font resource request boundary");

assertMatch(
    webFontResourceVfs,
    /export type GuiFontResourcePathErrorReason[\s\S]*Empty[\s\S]*Absolute[\s\S]*Backslash[\s\S]*EmptySegment[\s\S]*DotSegment[\s\S]*ParentSegment[\s\S]*VfsPathMismatch/,
    "Web font resource VFS must classify invalid resource paths as typed reasons",
);
assertMatch(
    webFontResourceVfs,
    /export const GUI_FONT_RESOURCE_ROOT = 'fonts';[\s\S]*HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH = `\$\{GUI_FONT_RESOURCE_ROOT\}\/HackGenConsoleNF-Regular\.ttf`/,
    "Web font resource VFS must keep canonical paths slashless",
);
assertNoMatch(
    webFontResourceVfs,
    /GUI_FONT_RESOURCE_ROOT\s*=\s*'\/fonts'/,
    "Web font resource root must not use VFS absolute path as canonical resource path",
);
assertMatch(
    webFontResourceVfs,
    /resourcePath: HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH[\s\S]*vfsPath: `\/\$\{HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH\}`[\s\S]*sourceUrl: '\.\/src\/fonts\/HackGenConsoleNF-Regular\.ttf'/,
    "Web bundled font manifest must map HackGen fixture to canonical path and VFS path explicitly",
);
assertMatch(
    webFontResourceVfs,
    /resourcePath: HACKGEN_LICENSE_RESOURCE_PATH[\s\S]*vfsPath: `\/\$\{HACKGEN_LICENSE_RESOURCE_PATH\}`[\s\S]*sourceUrl: '\.\/src\/fonts\/HackGen-LICENSE\.txt'/,
    "Web bundled font manifest must include the license text as a normal resource",
);
assertMatch(
    webFontResourceVfs,
    /export function normalizeGuiFontResourcePath[\s\S]*rawPath\.startsWith\('\/'\)[\s\S]*rawPath\.includes\('\\\\'\)[\s\S]*segment === '\.'[\s\S]*segment === '\.\.'/,
    "Web font resource path normalization must reject absolute, backslash, dot, and parent segments",
);
assertMatch(
    webFontResourceVfs,
    /export function guiFontResourceVfsPath[\s\S]*return `\/\$\{path\}`/,
    "Web font resource VFS path must be derived from canonical path with a single leading slash",
);
for (const errorKind of [
    "FetchUnavailable",
    "InvalidResourcePath",
    "NetworkError",
    "HttpError",
    "InvalidBytes",
    "InvalidText",
    "VfsWriteFailed",
]) {
    assertMatch(
        webFontResourceVfs,
        new RegExp(`kind: '${errorKind}'`),
        `Web font resource mount must expose typed ${errorKind} error`,
    );
}
assertMatch(
    webFontResourceVfs,
    /options: \{ fetch\?: GuiFontResourceFetch; resources\?: readonly GuiBundledFontResource\[\] \}[\s\S]*const resources = options\.resources \?\? BUNDLED_GUI_FONT_RESOURCES/,
    "Web font resource mount must support injected resource manifests without adding HackGen-specific APIs",
);
assertMatch(
    webFontResourceVfs,
    /vfs\.writeFile\(payload\.vfsPath, payload\.content, \{ force: true \}\)[\s\S]*vfs\.setReadOnly\(payload\.vfsPath, true\)[\s\S]*rollbackMountedFontResource\(vfs, payload\.vfsPath\)[\s\S]*rollbackMountedFontResource\(vfs, mountedPath\)/,
    "Web font resource mount must write read-only VFS files and roll back partial writes",
);
assertNoMatch(
    webFontResourceVfs,
    /\b(?:FontFace|CanvasRenderingContext2D|document|localStorage|indexedDB)\b/,
    "Web font resource VFS must not depend on browser font or persistent storage APIs",
);
assertNoMatch(
    webFontResourceVfs,
    /\b(?:fallback|Fallback)\b/,
    "Web font resource VFS must not describe hidden fallback behavior",
);
assertMatch(
    webMain,
    /const guiFontResourceMountPromise: Promise<GuiFontResourceMountResult> = mountBundledGuiFontResources\(vfs\)/,
    "Web main must start GUI font resource mounting at startup",
);
assertMatch(
    webMain,
    /beforeWasmExecution: guiFontResourceExecutionPreflight/,
    "Web main must pass GUI font resource preflight into terminal runtime",
);
assertMatch(
    webMain,
    /async function runCurrentFile\(\)[\s\S]*await ensureGuiFontResourcesForRun\(\)[\s\S]*executeCommand\(`neplg2 run -i \$\{activePath\}`\)/,
    "Web run path must await GUI font resource mount before execution",
);
assertMatch(
    webMain,
    /function compileCurrentFile\(\)[\s\S]*executeCommand\(`neplg2 build --emit wat -i \$\{activePath\}`\)/,
    "Web compile path must remain independent from runtime font resource mounting",
);
assertMatch(
    webMain,
    /function formatGuiFontResourceMountError\(error: GuiFontResourceMountError\): string[\s\S]*case 'FetchUnavailable'[\s\S]*case 'InvalidResourcePath'[\s\S]*case 'NetworkError'[\s\S]*case 'HttpError'[\s\S]*case 'InvalidBytes'[\s\S]*case 'InvalidText'[\s\S]*case 'VfsWriteFailed'/,
    "Web main must display all GUI font resource mount errors through typed branches",
);
assertMatch(
    webPanelManager,
    /beforeWasmExecution\?: \(\) => Promise<string \| null>[\s\S]*this\.beforeWasmExecution = options\.beforeWasmExecution \|\| \(async \(\) => null\)[\s\S]*beforeWasmExecution: this\.beforeWasmExecution/,
    "Panel manager must route wasm execution preflight into CanvasTerminal",
);
assertMatch(
    webTerminal,
    /new Shell\(this, \(options as any\)\.vfs \|\| null, \{[\s\S]*getCompilerMode: \(options as any\)\.getCompilerMode,[\s\S]*beforeWasmExecution: \(options as any\)\.beforeWasmExecution/,
    "Canvas terminal must forward wasm execution preflight into Shell",
);
assertMatch(
    webShell,
    /beforeWasmExecution\?: \(\) => Promise<string \| null>[\s\S]*beforeWasmExecutionOption[\s\S]*if \(request\.type === 'run-wasm'\)[\s\S]*await this\.beforeWasmExecutionOption\(\)[\s\S]*throw new Error\(blockedMessage\)/,
    "Shell must guard every run-wasm worker request, including typed terminal commands",
);
assertMatch(
    webVfs,
    /serializeForCompile\(\): Record<string, string>[\s\S]*this\.readOnlyFiles\.has\(path\)[\s\S]*!path\.endsWith\('\.nepl'\)[\s\S]*typeof content !== 'string'/,
    "Web VFS compile serialization must exclude read-only resources, non-NEPL files, and binary payloads",
);
assertMatch(
    webIndex,
    /<link data-trunk rel="copy-dir" href="src">/,
    "Web index must copy bundled font files under src/fonts for Trunk output",
);
assertMatch(
    webFontResourceBehaviorTest,
    /mountBundledGuiFontResources[\s\S]*serializeForCompile/,
    "Web font resource behavior test must cover mount success, typed failures, rollback, and compile overlay exclusion",
);
for (const behaviorKind of ["HttpError", "InvalidBytes", "InvalidText", "InvalidResourcePath", "VfsWriteFailed"]) {
    assertMatch(
        webFontResourceBehaviorTest,
        new RegExp(`"${behaviorKind}"`),
        `Web font resource behavior test must cover ${behaviorKind}`,
    );
}

console.log("web GUI font rendering contract passed");

