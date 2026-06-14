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
    const candidates = [
        source.indexOf("\nfn ", start + 1),
        source.indexOf("\npub fn ", start + 1),
        source.indexOf("\npub struct ", start + 1),
        source.indexOf("\npub enum ", start + 1),
        source.indexOf("\nimpl ", start + 1),
    ].filter((index) => index >= 0);
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
const guiFontSfntPathTests = read("tests/stdlib/gui_font_sfnt_glyf_path.n.md");
const guiFontSfntCurveLookupTests = read("tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md");
const guiFontSfntTests = [
    read("tests/stdlib/gui_font_sfnt.n.md"),
    read("tests/stdlib/gui_font_sfnt_glyf.n.md"),
    read("tests/stdlib/gui_font_sfnt_glyf_curve.n.md"),
    guiFontSfntPathTests,
    guiFontSfntCurveLookupTests,
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
    spec,
    /SFNT simple glyph contour point lookup[\s\S]*GuiSfntSimpleGlyphContourPoint:[\s\S]*span GuiSfntSimpleGlyphContourSpan[\s\S]*contour_point_index i32[\s\S]*point GuiSfntSimpleGlyphPoint[\s\S]*gui_sfnt_lookup_simple_glyph_contour_point:[\s\S]*Result GuiSfntSimpleGlyphContourPoint GuiSfntParseError[\s\S]*contour_point_index[\s\S]*contour-local[\s\S]*point\.point_index[\s\S]*absolute logical point index[\s\S]*absolute_point_index = span\.start_point_index \+ contour_point_index[\s\S]*validate contour_point_index[\s\S]*point decode[\s\S]*MissingGlyphOutline/,
    "font spec must define F4j contour-local point contract, absolute point formula, local-before-point order, and typed errors",
);
for (const fragment of [
    "SFNT simple glyph contour edge lookup",
    "描画される直線 segment ではない",
    "GuiSfntSimpleGlyphContourEdge:",
    "start GuiSfntSimpleGlyphContourPoint",
    "end GuiSfntSimpleGlyphContourPoint",
    "edge_index i32",
    "next_contour_point_index i32",
    "gui_sfnt_lookup_simple_glyph_contour_edge:",
    "Result GuiSfntSimpleGlyphContourEdge GuiSfntParseError",
    "start.contour_point_index == edge_index",
    "end.contour_point_index == next_contour_point_index",
    "contour span lookup",
    "validate edge_index",
    "compute next_contour_point_index",
    "decode start contour point",
    "decode end contour point",
    "MissingGlyphOutline",
    "span.point_count == 1",
    "自己 wrap",
    "full edge `Vec`",
]) {
    assert(spec.includes(fragment), `font spec F4k contour edge contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph curve segment classification",
    "GuiSfntSimpleGlyphCurveNoSegmentReason:",
    "SinglePointContour",
    "OffCurveStart",
    "MissingLookahead",
    "GuiSfntSimpleGlyphLineSegment:",
    "start_x2 i32",
    "end_y2 i32",
    "GuiSfntSimpleGlyphQuadraticSegment:",
    "control_x2 i32",
    "end_is_implied bool",
    "GuiSfntSimpleGlyphCurveSegment:",
    "NoSegment GuiSfntSimpleGlyphCurveNoSegment",
    "Line GuiSfntSimpleGlyphLineSegment",
    "Quadratic GuiSfntSimpleGlyphQuadraticSegment",
    "`*_x2` / `*_y2` は font unit の 2 倍",
    "end_x2 = control.x + lookahead.x",
    "整数除算による丸めや fallback を行わない",
    "gui_sfnt_classify_simple_glyph_curve_segment:",
    "gui_sfnt_lookup_simple_glyph_curve_segment:",
    "Result GuiSfntSimpleGlyphCurveSegment GuiSfntParseError",
    "NoSegment` は parse error ではない",
]) {
    assert(spec.includes(fragment), `font spec F4l curve segment contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path command projection",
    "GuiSfntSimpleGlyphPathMoveTo:",
    "GuiSfntSimpleGlyphPathLineTo:",
    "GuiSfntSimpleGlyphPathQuadraticTo:",
    "GuiSfntSimpleGlyphPathSkipNoSegment:",
    "GuiSfntSimpleGlyphPathCommand:",
    "MoveTo GuiSfntSimpleGlyphPathMoveTo",
    "LineTo GuiSfntSimpleGlyphPathLineTo",
    "QuadraticTo GuiSfntSimpleGlyphPathQuadraticTo",
    "SkipNoSegment GuiSfntSimpleGlyphPathSkipNoSegment",
    "gui_sfnt_simple_glyph_curve_segment_move_to_command:",
    "gui_sfnt_simple_glyph_curve_segment_draw_command:",
    "-> GuiSfntSimpleGlyphPathCommand",
    "`gui_sfnt_simple_glyph_curve_segment_move_to_command` は `Line` / `Quadratic` を segment start の `MoveTo` に写す",
    "`gui_sfnt_simple_glyph_curve_segment_draw_command` は `Line` を `LineTo`、`Quadratic` を `QuadraticTo` に写す",
    "`NoSegment` はどちらの関数でも `SkipNoSegment` に写す",
    "この API は command index を受け取らず、`Option` や `Result` も返さない",
    "`SkipNoSegment` は fallback drawing ではなく",
    "F4m は `Vec GuiSfntSimpleGlyphPathCommand` を作らない",
]) {
    assert(spec.includes(fragment), `font spec F4m path command contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path command public lookup",
    "gui_sfnt_lookup_simple_glyph_move_to_command:",
    "gui_sfnt_lookup_simple_glyph_draw_command:",
    "-> Result GuiSfntSimpleGlyphPathCommand GuiSfntParseError",
    "F4n は `gui_sfnt_lookup_simple_glyph_curve_segment` を呼び",
    "F4m の `move_to_command` または `draw_command` に渡す",
    "`NoSegment` は parse error ではないため、F4n でも `Result::Ok (SkipNoSegment ...)`",
]) {
    assert(spec.includes(fragment), `font spec F4n path command lookup contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path command pair lookup",
    "GuiSfntSimpleGlyphPathCommandPair:",
    "move_command GuiSfntSimpleGlyphPathCommand",
    "draw_command GuiSfntSimpleGlyphPathCommand",
    "gui_sfnt_simple_glyph_curve_segment_path_command_pair:",
    "gui_sfnt_lookup_simple_glyph_path_command_pair:",
    "-> Result GuiSfntSimpleGlyphPathCommandPair GuiSfntParseError",
    "contour stream、command sequence、full outline、sink trait ではない",
    "command index、count、next pointer、current point state は導入しない",
    "`gui_sfnt_lookup_simple_glyph_curve_segment` を 1 回だけ呼び",
    "`NoSegment` は pair 内の `move_command` と `draw_command` の両方で `SkipNoSegment`",
    "F4o は `Vec GuiSfntSimpleGlyphPathCommand` を作らない",
]) {
    assert(spec.includes(fragment), `font spec F4o path command pair contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink event adapter",
    "GuiSfntSimpleGlyphPathSinkEvent:",
    "Command GuiSfntSimpleGlyphPathCommand",
    "GuiSfntSimpleGlyphPathSinkEventPair:",
    "first_event GuiSfntSimpleGlyphPathSinkEvent",
    "second_event GuiSfntSimpleGlyphPathSinkEvent",
    "single-edge adapter",
    "これは F5 の contour stream ではなく",
    "新しい path command 表現ではない",
    "payload を再定義しない",
    "pure projection は total",
    "`Option` や `Result` を返さない",
    "`SkipNoSegment` も `Command (SkipNoSegment ...)` として保持",
    "F4p は `Vec GuiSfntSimpleGlyphPathSinkEvent` を作らない",
]) {
    assert(spec.includes(fragment), `font spec F4p path sink event adapter contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink event kind classification",
    "GuiSfntSimpleGlyphPathSinkEventKind:",
    "MoveTo",
    "LineTo",
    "QuadraticTo",
    "SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason",
    "GuiSfntSimpleGlyphPathSinkEventKindPair:",
    "first_kind GuiSfntSimpleGlyphPathSinkEventKind",
    "second_kind GuiSfntSimpleGlyphPathSinkEventKind",
    "dispatch 用分類値",
    "描画座標、source contour / edge",
    "実 payload は常に `GuiSfntSimpleGlyphPathSinkEvent` から `GuiSfntSimpleGlyphPathCommand`",
    "diagnostics、skip counting、branch selection",
    "source contour / edge を復元する値ではない",
    "kind ではなく既存 command payload",
    "F4q の pure helper は total",
    "`Vec GuiSfntSimpleGlyphPathSinkEventKind`",
]) {
    assert(spec.includes(fragment), `font spec F4q path sink event kind contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink event indexed selection",
    "GuiSfntSimpleGlyphPathSinkEventSlot:",
    "First",
    "Second",
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at:",
    "gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at:",
    "gui_sfnt_simple_glyph_path_sink_event_pair_kind_at:",
    "存在しない third event や負の index は型として表現できない",
    "`event_pair_kind_at` は `event_pair_event_at` と `gui_sfnt_simple_glyph_path_sink_event_kind` の合成",
    "numeric `i32` index",
    "contour traversal",
    "render2d",
    "platform API",
]) {
    assert(spec.includes(fragment), `font spec F4r path sink event indexed selection contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path contour traversal step",
    "GuiSfntSimpleGlyphPathContourCursor:",
    "GuiSfntSimpleGlyphPathContourNext:",
    "Continue GuiSfntSimpleGlyphPathContourCursor",
    "EndContour",
    "GuiSfntSimpleGlyphPathContourStep:",
    "gui_sfnt_lookup_simple_glyph_path_contour_step:",
    "Result GuiSfntSimpleGlyphPathContourStep GuiSfntParseError",
    "gui_sfnt_lookup_simple_glyph_contour_span",
    "gui_sfnt_lookup_simple_glyph_path_command_pair",
    "gui_sfnt_simple_glyph_path_command_pair_sink_event_pair",
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at",
    "gui_sfnt_simple_glyph_path_sink_event_kind",
    "slot First -> same edge Second",
    "slot Second -> edge + 1 First or EndContour",
    "step.next = EndContour",
    "private helper",
    "SkipNoSegment OffCurveStart",
    "full outline allocation",
    "platform API",
]) {
    assert(spec.includes(fragment), `font spec F4s path contour traversal step contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph allocation-free path sink ownership boundary",
    "GuiSfntSimpleGlyphPathOffCurveStartPolicy:",
    "KeepTypedSkip",
    "RejectUnsupported",
    "GuiSfntSimpleGlyphPathClosurePolicy:",
    "EmitCloseAfterFinalEvent",
    "GuiSfntSimpleGlyphPathSinkPolicy:",
    "GuiSfntSimpleGlyphPathSinkRejectReason:",
    "UnsupportedOffCurveStart",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction:",
    "EmitEvent GuiSfntSimpleGlyphPathSinkEvent",
    "Reject GuiSfntSimpleGlyphPathSinkRejectReason",
    "GuiSfntSimpleGlyphPathSinkTailAction:",
    "NoTailAction",
    "CloseContour GuiSfntSimpleGlyphPathContourClose",
    "policy reject は `GuiSfntParseError` ではない",
    "`SinglePointContour` と `MissingLookahead`",
    "reject と close contour は同時に発生しない",
    "primary = Reject _",
    "step.next = Continue _",
    "step.next = EndContour",
    "gui_sfnt_simple_glyph_path_sink_step_from_contour_step:",
    "gui_sfnt_lookup_simple_glyph_path_sink_step:",
    "gui_sfnt_lookup_simple_glyph_path_contour_step",
]) {
    assert(spec.includes(fragment), `font spec F4t path sink ownership boundary contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action selection projection",
    "GuiSfntSimpleGlyphPathSinkActionSlot:",
    "Primary",
    "Tail",
    "GuiSfntSimpleGlyphPathSinkAction:",
    "EmitEvent GuiSfntSimpleGlyphPathSinkEvent",
    "Reject GuiSfntSimpleGlyphPathSinkRejectReason",
    "CloseContour GuiSfntSimpleGlyphPathContourClose",
    "NoAction",
    "`GuiSfntSimpleGlyphPathSinkActionSlot` は F4r/F4s の `GuiSfntSimpleGlyphPathSinkEventSlot` とは別の軸",
    "`NoAction` は `GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction` の明示的な projection",
    "primary action projection は `NoAction` を返さず",
    "gui_sfnt_simple_glyph_path_sink_step_action_at:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action:",
    "byte-backed helper は `gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び",
]) {
    assert(spec.includes(fragment), `font spec F4u path sink action projection contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action traversal step",
    "Primary -> Tail -> F4s source next",
    "GuiSfntSimpleGlyphPathSinkActionCursor:",
    "contour_cursor GuiSfntSimpleGlyphPathContourCursor",
    "action_slot GuiSfntSimpleGlyphPathSinkActionSlot",
    "GuiSfntSimpleGlyphPathSinkActionNext:",
    "Continue GuiSfntSimpleGlyphPathSinkActionCursor",
    "EndContour",
    "GuiSfntSimpleGlyphPathSinkActionStep:",
    "next の規則は action payload と独立",
    "action_slot = Primary",
    "Continue same contour_cursor Tail",
    "action_slot = Tail and sink_step.source_step.next = Continue next_cursor",
    "Continue next_cursor Primary",
    "Tail -> source_step.next",
    "gui_sfnt_simple_glyph_path_sink_action_next_from_step:",
    "gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step:",
    "F4u の `gui_sfnt_simple_glyph_path_sink_step_action_at` に委譲",
    "byte-backed helper は action cursor から contour cursor と action slot を読み",
]) {
    assert(spec.includes(fragment), `font spec F4v path sink action traversal contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action start cursor",
    "contour edge `0`、event slot `First`、action slot `Primary`",
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor:",
    "unchecked value constructor",
    "byte-backed entry point",
    "`gui_sfnt_lookup_simple_glyph_contour_span` を 1 回だけ呼ぶ",
    "F4v action step lookup、F4t sink step lookup、F4s contour step lookup",
    "pure constructor が contour の存在や byte 妥当性を証明するものとして document してはならない",
]) {
    assert(spec.includes(fragment), `font spec F4w path sink action start cursor contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action start step",
    "contour の first action step",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step:",
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy",
    "byte-backed start cursor helper は呼ばない",
    "contour span 検証が二重になる",
    "`Result::Err` は下位 action step lookup の parse/range error",
    "policy reject は `Result::Err` ではなく",
    "F4x は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`",
]) {
    assert(spec.includes(fragment), `font spec F4x path sink action start step contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action step advance",
    "GuiSfntSimpleGlyphPathSinkActionStepAdvance:",
    "Continue GuiSfntSimpleGlyphPathSinkActionStep",
    "EndContour",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance:",
    "helper は `gui_sfnt_simple_glyph_path_sink_action_step_next step` を読み",
    "next = Continue cursor",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step",
    "next = EndContour",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour",
    "policy reject は次 step の `action = Reject` payload",
    "F4y は start cursor helper",
]) {
    assert(spec.includes(fragment), `font spec F4y path sink action step advance contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action step item",
    "GuiSfntSimpleGlyphPathSinkActionStepItem:",
    "step GuiSfntSimpleGlyphPathSinkActionStep",
    "advance GuiSfntSimpleGlyphPathSinkActionStepAdvance",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item:",
    "helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy` にだけ委譲する",
    "現在 step を明示コピーして",
    "F4z helper は action payload を見ない",
    "start step composition",
    "F4z は start cursor/start step helper",
]) {
    assert(spec.includes(fragment), `font spec F4z path sink action step item contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action start item",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_item:",
    "-> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntParseError",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy",
    "`gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` は 1 回だけ呼び",
    "`gui_sfnt_lookup_simple_glyph_path_sink_action_step_item` も 1 回だけ呼ぶ",
    "F4aa helper 自体は start cursor を作らず",
    "action payload を読まず",
    "F4aa は action start cursor helper",
]) {
    assert(spec.includes(fragment), `font spec F4aa path sink action start item contract must mention ${fragment}`);
}
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
    detailedDesign,
    /SFNT simple glyph contour point lookup[\s\S]*GuiSfntSimpleGlyphContourPoint:[\s\S]*span GuiSfntSimpleGlyphContourSpan[\s\S]*contour_point_index i32[\s\S]*point GuiSfntSimpleGlyphPoint[\s\S]*gui_sfnt_lookup_simple_glyph_contour_point:[\s\S]*Result GuiSfntSimpleGlyphContourPoint GuiSfntParseError[\s\S]*gui_sfnt_glyf_simple_contour_span_with_tables[\s\S]*validate contour-local contour_point_index[\s\S]*absolute_point_index = span\.start_point_index \+ contour_point_index[\s\S]*gui_sfnt_glyf_simple_point_with_tables[\s\S]*point\.point_index[\s\S]*absolute[\s\S]*validate local point range before calling point decode[\s\S]*MissingGlyphOutline/,
    "font detailed design must define F4j local point flow, internal helper reuse, absolute point invariant, and local-before-point validation",
);
for (const fragment of [
    "SFNT simple glyph contour edge lookup",
    "topology point pair",
    "not a drawable line segment",
    "GuiSfntSimpleGlyphContourEdge:",
    "gui_sfnt_lookup_simple_glyph_contour_edge:",
    "Result GuiSfntSimpleGlyphContourEdge GuiSfntParseError",
    "gui_sfnt_glyf_simple_contour_span_with_tables",
    "validate edge_index against span.point_count",
    "next_contour_point_index = wrap(edge_index + 1, span.point_count)",
    "gui_sfnt_glyf_simple_contour_point_with_tables for start",
    "gui_sfnt_glyf_simple_contour_point_with_tables for end",
    "start.contour_point_index",
    "end.contour_point_index",
    "One-point contours",
    "self-wrapping topology edge",
    "must not allocate `Vec GuiSfntSimpleGlyphContourEdge`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4k contour edge contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph curve segment classification",
    "enum payloads instead of a shared struct with inactive fields",
    "GuiSfntSimpleGlyphCurveNoSegmentReason:",
    "GuiSfntSimpleGlyphLineSegment:",
    "GuiSfntSimpleGlyphQuadraticSegment:",
    "GuiSfntSimpleGlyphCurveSegment:",
    "Coordinate fields are doubled font units",
    "end_x2 = control.x + lookahead.x",
    "must not compute implied midpoint with integer division",
    "Pure classifier flow",
    "return NoSegment SinglePointContour",
    "return NoSegment OffCurveStart",
    "return Quadratic with implied doubled midpoint end",
    "Byte lookup flow",
    "if start is on-curve and end is off-curve",
    "gui_sfnt_glyf_simple_contour_point_with_tables for lookahead",
    "NoSegment` is a successful classification state",
    "must not allocate `Vec GuiSfntSimpleGlyphCurveSegment`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4l curve segment contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path command projection",
    "first sink-facing projection layer above curve segment classification",
    "GuiSfntSimpleGlyphPathMoveTo:",
    "GuiSfntSimpleGlyphPathLineTo:",
    "GuiSfntSimpleGlyphPathQuadraticTo:",
    "GuiSfntSimpleGlyphPathSkipNoSegment:",
    "GuiSfntSimpleGlyphPathCommand:",
    "MoveTo GuiSfntSimpleGlyphPathMoveTo",
    "LineTo GuiSfntSimpleGlyphPathLineTo",
    "QuadraticTo GuiSfntSimpleGlyphPathQuadraticTo",
    "SkipNoSegment GuiSfntSimpleGlyphPathSkipNoSegment",
    "gui_sfnt_simple_glyph_curve_segment_move_to_command",
    "gui_sfnt_simple_glyph_curve_segment_draw_command",
    "There is no command index in this API",
    "Returning `GuiSfntSimpleGlyphPathCommand` directly",
    "SkipNoSegment` is a typed command",
    "must not allocate `Vec GuiSfntSimpleGlyphPathCommand`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4m path command contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path command public lookup",
    "thin public composition layer over F4l and F4m",
    "gui_sfnt_lookup_simple_glyph_move_to_command bytes face_index glyph contour_index edge_index",
    "gui_sfnt_lookup_simple_glyph_draw_command bytes face_index glyph contour_index edge_index",
    "The implementation must not call `gui_sfnt_parse_metadata`",
    "Result::Ok segment",
    "Result::Ok path_command",
    "`NoSegment` remains a successful path command state",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4n path command lookup contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path command pair lookup",
    "F4o is a single-edge pair boundary",
    "not a contour stream",
    "GuiSfntSimpleGlyphPathCommandPair:",
    "gui_sfnt_simple_glyph_curve_segment_path_command_pair segment",
    "GuiSfntSimpleGlyphPathCommandPair move draw",
    "gui_sfnt_lookup_simple_glyph_path_command_pair bytes face_index glyph contour_index edge_index",
    "must call `gui_sfnt_lookup_simple_glyph_curve_segment` exactly once",
    "must not call the separate move and draw public lookup helpers",
    "The pair is not a list",
    "does not expose `command_index`, `count`, `next`, mutable current point state",
    "Both `move_command` and `draw_command` are `SkipNoSegment`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4o path command pair contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink event adapter",
    "F4p is a single-edge adapter",
    "not the real sink",
    "GuiSfntSimpleGlyphPathSinkEvent:",
    "Command GuiSfntSimpleGlyphPathCommand",
    "GuiSfntSimpleGlyphPathSinkEventPair:",
    "gui_sfnt_simple_glyph_path_command_sink_event command",
    "gui_sfnt_simple_glyph_path_command_pair_sink_event_pair pair",
    "uses only the F4o accessors",
    "must not return `Option` or `Result`",
    "must not allocate `Vec GuiSfntSimpleGlyphPathSinkEvent`",
    "`SkipNoSegment` remains a typed event",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4p path sink event adapter contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink event kind classification",
    "F4q adds a dispatch classification boundary",
    "not a replacement path representation",
    "not a compact payload enum",
    "The authority for coordinates, contour index, edge index",
    "GuiSfntSimpleGlyphPathSinkEventKind:",
    "SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason",
    "GuiSfntSimpleGlyphPathSinkEventKindPair:",
    "The event kind helper is total",
    "uses exhaustive `match`",
    "carries only `GuiSfntSimpleGlyphCurveNoSegmentReason`",
    "not enough to recover the source contour/edge",
    "must use only F4p event pair accessors",
    "must not add `contour_index`, `edge_index`, coordinate fields",
    "must not return `Option` or `Result`",
    "Vec GuiSfntSimpleGlyphPathSinkEventKind",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4q path sink event kind contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink event indexed selection",
    "F4r adds typed slot selection",
    "not a contour iterator",
    "GuiSfntSimpleGlyphPathSinkEventSlot:",
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at pair slot",
    "match slot:",
    "First -> gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair",
    "Second -> gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair",
    "must not accept an `i32` event index",
    "helpers therefore must not return `Option` or `Result`",
    "event_at` and the F4q kind helper",
    "must not allocate `Vec GuiSfntSimpleGlyphPathSinkEvent`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4r path sink event indexed selection contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path contour traversal step",
    "F4s adds the first cursor-shaped traversal boundary",
    "GuiSfntSimpleGlyphPathContourCursor:",
    "GuiSfntSimpleGlyphPathContourNext:",
    "GuiSfntSimpleGlyphPathContourStep:",
    "gui_sfnt_lookup_simple_glyph_path_contour_step bytes face_index cursor",
    "span = gui_sfnt_lookup_simple_glyph_contour_span",
    "pair = gui_sfnt_lookup_simple_glyph_path_command_pair",
    "event_pair = gui_sfnt_simple_glyph_path_command_pair_sink_event_pair",
    "event = gui_sfnt_simple_glyph_path_sink_event_pair_event_at",
    "kind = gui_sfnt_simple_glyph_path_sink_event_kind",
    "private validated cursor-next helper",
    "First  -> Continue same glyph / same contour / same edge / Second",
    "Second -> Continue same glyph / same contour / next edge / First",
    "Second on final edge -> EndContour",
    "The next helper must remain private unless it is changed to return `Result`",
    "SkipNoSegment OffCurveStart",
    "Contour closure insertion",
    "platform presentation",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4s path contour traversal step contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph allocation-free path sink ownership boundary",
    "F4t turns an F4s contour step into a sink-facing ownership decision",
    "GuiSfntSimpleGlyphPathOffCurveStartPolicy:",
    "GuiSfntSimpleGlyphPathClosurePolicy:",
    "GuiSfntSimpleGlyphPathSinkPolicy:",
    "GuiSfntSimpleGlyphPathSinkRejectReason:",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction:",
    "GuiSfntSimpleGlyphPathSinkTailAction:",
    "off_curve_start_policy` only applies to `GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment OffCurveStart",
    "`Reject` is not `GuiSfntParseError`",
    "if primary_action is Reject:",
    "tail_action = NoTailAction",
    "else if step.next is Continue:",
    "else if step.next is EndContour and closure_policy is EmitCloseAfterFinalEvent:",
    "CloseContour glyph contour_index",
    "gui_sfnt_simple_glyph_path_sink_step_from_contour_step policy step",
    "gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index cursor policy",
    "It must not re-parse metadata itself",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4t path sink ownership boundary contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action selection projection",
    "F4u projects an F4t sink step into one explicitly selected action",
    "selection/projection layer for a future sink",
    "GuiSfntSimpleGlyphPathSinkActionSlot:",
    "GuiSfntSimpleGlyphPathSinkAction:",
    "NoAction` is only the explicit projection of `GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction`",
    "Primary action projection must never return `NoAction`",
    "gui_sfnt_simple_glyph_path_sink_primary_action_as_action primary_action",
    "gui_sfnt_simple_glyph_path_sink_tail_action_as_action tail_action",
    "gui_sfnt_simple_glyph_path_sink_step_action_at step slot",
    "must not return `Option` or `Result`, expose `command_index`, accept a numeric action index",
    "gui_sfnt_lookup_simple_glyph_path_sink_action bytes face_index cursor policy slot",
    "must call `gui_sfnt_lookup_simple_glyph_path_sink_step` exactly once",
    "policy rejection remains `Result::Ok GuiSfntSimpleGlyphPathSinkAction::Reject`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4u path sink action projection contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action traversal step",
    "F4v turns F4u action selection into a typed traversal step",
    "not a real sink",
    "GuiSfntSimpleGlyphPathSinkActionCursor:",
    "GuiSfntSimpleGlyphPathSinkActionNext:",
    "GuiSfntSimpleGlyphPathSinkActionStep:",
    "F4v must not add a new numeric action index, command index, loop index, count field, or ad-hoc traversal counter",
    "EndContour` is a successful terminal state",
    "`action` and `next` are separate facts",
    "Next-state computation must not inspect whether the action is `EmitEvent`, `Reject`, `CloseContour`, or `NoAction`",
    "gui_sfnt_simple_glyph_path_sink_action_next_from_step sink_step action_slot",
    "source_step.cursor -> Continue same contour_cursor Tail",
    "source_step.next = Continue next_cursor",
    "Continue next_cursor Primary",
    "This means `Primary -> Tail` happens even when the primary action is `Reject`",
    "It also means `Tail -> source_step.next` happens even when the tail action is `NoAction`",
    "must compose F4u rather than duplicate it",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy",
    "must call `gui_sfnt_lookup_simple_glyph_path_sink_step` exactly once",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4v path sink action traversal contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action start cursor",
    "F4w adds the entry position for the F4v action traversal stream",
    "edge_index = 0",
    "event_slot = First",
    "action_slot = Primary",
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index",
    "This constructor is intentionally unchecked",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor bytes face_index glyph contour_index",
    "gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index",
    "must not call F4v action-step lookup, F4t sink-step lookup, F4s contour-step lookup",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4w path sink action start cursor contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action start step",
    "F4x adds the first-step entry point",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy",
    "start_cursor = gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy",
    "deliberately does not call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`",
    "duplicate the contour span validation",
    "F4x is not a new authority",
    "parse/range/table error",
    "policy reject",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4x path sink action start step contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action step advance",
    "F4y resolves one `GuiSfntSimpleGlyphPathSinkActionNext` value",
    "GuiSfntSimpleGlyphPathSinkActionStepAdvance:",
    "The type is separate from `GuiSfntSimpleGlyphPathSinkActionNext`",
    "Returning `Option GuiSfntSimpleGlyphPathSinkActionStep` would lose the domain reason",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy",
    "next = gui_sfnt_simple_glyph_path_sink_action_step_next step",
    "Ok next_step -> Ok Continue next_step",
    "EndContour:",
    "Ok EndContour",
    "F4y does not inspect `step.action`",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4y path sink action step advance contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action step item",
    "F4z packages the current action step and the checked advance result",
    "GuiSfntSimpleGlyphPathSinkActionStepItem:",
    "step GuiSfntSimpleGlyphPathSinkActionStep",
    "advance GuiSfntSimpleGlyphPathSinkActionStepAdvance",
    "not a contour iterator",
    "F4z itself does not interpret the action payload",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index step policy",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy",
    "stored_step = *step",
    "must not store a borrowed reference",
    "F4z must not call start cursor helpers",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4z path sink action step item contract must mention ${fragment}`);
}
for (const fragment of [
    "SFNT simple glyph path sink action start item",
    "F4aa adds a first-item entry point above F4x and F4z",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy",
    "F4aa does not introduce a new data type",
    "The result type is the F4z `GuiSfntSimpleGlyphPathSinkActionStepItem`",
    "must call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` exactly once",
    "must not call the pure start-cursor helper",
    "F4aa itself does not inspect the action payload",
]) {
    assert(detailedDesign.includes(fragment), `font detailed design F4aa path sink action start item contract must mention ${fragment}`);
}
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
    implementationPlan,
    /Phase F4j:[\s\S]*GuiSfntSimpleGlyphContourPoint[\s\S]*gui_sfnt_lookup_simple_glyph_contour_point[\s\S]*span GuiSfntSimpleGlyphContourSpan[\s\S]*contour_point_index i32[\s\S]*point GuiSfntSimpleGlyphPoint[\s\S]*absolute_point_index = span\.start_point_index \+ contour_point_index[\s\S]*contour span lookup -> validate contour_point_index -> compute absolute_point_index -> point decode[\s\S]*MissingGlyphOutline[\s\S]*gui_sfnt_glyf_simple_contour_span_with_tables[\s\S]*gui_sfnt_glyf_simple_point_with_tables[\s\S]*Source policy[\s\S]*no Vec allocation/,
    "font implementation plan must define F4j contour point implementation, local-before-point order, internal helper reuse, source policy gates, and doctest coverage",
);
for (const fragment of [
    "Phase F4k:",
    "topology pair",
    "quadratic curve classification",
    "full edge `Vec`",
    "GuiSfntSimpleGlyphContourEdge",
    "gui_sfnt_lookup_simple_glyph_contour_edge",
    "start GuiSfntSimpleGlyphContourPoint",
    "end GuiSfntSimpleGlyphContourPoint",
    "edge_index i32",
    "next_contour_point_index i32",
    "start.contour_point_index == edge_index",
    "end.contour_point_index == next_contour_point_index",
    "contour span lookup -> validate edge_index -> compute next_contour_point_index -> decode start contour point -> decode end contour point",
    "span.point_count == 1",
    "topology self-wrap",
    "gui_sfnt_glyf_simple_contour_edge_with_tables",
    "Source policy",
    "no Vec allocation",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4k contour edge contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4l:",
    "line / quadratic / no-segment",
    "`x2` / `y2`",
    "GuiSfntSimpleGlyphCurveNoSegmentReason",
    "GuiSfntSimpleGlyphCurveNoSegment",
    "GuiSfntSimpleGlyphLineSegment",
    "GuiSfntSimpleGlyphQuadraticSegment",
    "GuiSfntSimpleGlyphCurveSegment",
    "gui_sfnt_classify_simple_glyph_curve_segment",
    "gui_sfnt_lookup_simple_glyph_curve_segment",
    "payload 付き enum",
    "inactive field",
    "end_x2 = control.x + lookahead.x",
    "`div_s ... 2` や丸めは使わない",
    "NoSegment SinglePointContour",
    "NoSegment OffCurveStart",
    "NoSegment MissingLookahead",
    "gui_sfnt_glyf_simple_curve_segment_with_tables",
    "tests/stdlib/gui_font_sfnt_glyf_curve.n.md",
    "no curve segment `Vec` allocation",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4l curve segment contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4m:",
    "path command projection",
    "GuiSfntSimpleGlyphPathMoveTo",
    "GuiSfntSimpleGlyphPathLineTo",
    "GuiSfntSimpleGlyphPathQuadraticTo",
    "GuiSfntSimpleGlyphPathSkipNoSegment",
    "GuiSfntSimpleGlyphPathCommand",
    "gui_sfnt_simple_glyph_curve_segment_move_to_command",
    "gui_sfnt_simple_glyph_curve_segment_draw_command",
    "`Line` は `move_to_command` で `MoveTo`、`draw_command` で `LineTo`",
    "`Quadratic` は `move_to_command` で `MoveTo`、`draw_command` で `QuadraticTo`",
    "`NoSegment` はどちらの関数でも `SkipNoSegment`",
    "command index を受け取らず、`Option` / `Result` も返さない",
    "no `Vec GuiSfntSimpleGlyphPathCommand` allocation",
    "no metadata parse",
    "tests/stdlib/gui_font_sfnt_glyf_path.n.md",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4m path command contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4n:",
    "sfnt simple glyph path command public lookup",
    "gui_sfnt_lookup_simple_glyph_move_to_command",
    "gui_sfnt_lookup_simple_glyph_draw_command",
    "F4l の byte-backed curve segment lookup と F4m の path command projection を合成するだけ",
    "`Result::Err` は同じ `GuiSfntParseError` として伝播",
    "F4n では `gui_sfnt_parse_metadata`",
    "`NoSegment` は `Result::Ok SkipNoSegment`",
    "no `Vec GuiSfntSimpleGlyphPathCommand`",
    "tests/stdlib/gui_font_sfnt_glyf_path.n.md",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4n path command lookup contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4o:",
    "sfnt simple glyph path command pair lookup",
    "GuiSfntSimpleGlyphPathCommandPair",
    "gui_sfnt_simple_glyph_path_command_pair",
    "gui_sfnt_simple_glyph_path_command_pair_move_command",
    "gui_sfnt_simple_glyph_path_command_pair_draw_command",
    "gui_sfnt_simple_glyph_curve_segment_path_command_pair",
    "gui_sfnt_lookup_simple_glyph_path_command_pair",
    "O(1) value",
    "`gui_sfnt_lookup_simple_glyph_curve_segment` を 1 回だけ呼び",
    "`NoSegment` は pair 内の move / draw の両方で `SkipNoSegment`",
    "command index、count、next、current point state",
    "no list / no sink / no metadata unwrap / no table-helper bypass",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4o path command pair contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4p:",
    "sfnt simple glyph path sink event adapter",
    "GuiSfntSimpleGlyphPathSinkEvent",
    "GuiSfntSimpleGlyphPathSinkEventPair",
    "gui_sfnt_simple_glyph_path_command_sink_event",
    "gui_sfnt_simple_glyph_path_sink_event_command",
    "gui_sfnt_simple_glyph_path_sink_event_pair",
    "gui_sfnt_simple_glyph_path_sink_event_pair_first_event",
    "gui_sfnt_simple_glyph_path_sink_event_pair_second_event",
    "gui_sfnt_simple_glyph_path_command_pair_sink_event_pair",
    "`Command GuiSfntSimpleGlyphPathCommand` の thin wrapper",
    "payload を再定義しない",
    "`first_event` と `second_event` だけを持つ O(1) value",
    "Option` / `Result`",
    "`Vec GuiSfntSimpleGlyphPathSinkEvent`",
    "no duplicate payload enum",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4p path sink event adapter contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4r:",
    "sfnt simple glyph path sink event indexed selection",
    "GuiSfntSimpleGlyphPathSinkEventSlot",
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at",
    "gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at",
    "gui_sfnt_simple_glyph_path_sink_event_pair_kind_at",
    "`First` と `Second` だけを持ち",
    "slot を明示 `match`",
    "catch-all arm は使わない",
    "`i32` event index",
    "no allocation/stream state",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4r path sink event indexed selection contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4s:",
    "sfnt simple glyph path contour traversal step",
    "GuiSfntSimpleGlyphPathContourCursor",
    "GuiSfntSimpleGlyphPathContourNext",
    "GuiSfntSimpleGlyphPathContourStep",
    "private `gui_sfnt_simple_glyph_path_contour_next_from_cursor`",
    "public `gui_sfnt_lookup_simple_glyph_path_contour_step`",
    "`span_point_count > 0`",
    "`0 <= edge_index < span_point_count`",
    "`First` なら same edge `Second`",
    "`edge + 1` の `First` または `EndContour`",
    "gui_sfnt_lookup_simple_glyph_contour_span",
    "gui_sfnt_lookup_simple_glyph_path_command_pair",
    "gui_sfnt_simple_glyph_path_command_pair_sink_event_pair",
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at",
    "gui_sfnt_simple_glyph_path_sink_event_kind",
    "no fallback/no allocation/no renderer/no platform",
    "SkipNoSegment OffCurveStart",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4s path contour traversal step contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4t:",
    "sfnt simple glyph allocation-free path sink ownership boundary",
    "GuiSfntSimpleGlyphPathOffCurveStartPolicy",
    "GuiSfntSimpleGlyphPathClosurePolicy",
    "GuiSfntSimpleGlyphPathSinkPolicy",
    "GuiSfntSimpleGlyphPathSinkRejectReason",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction",
    "GuiSfntSimpleGlyphPathContourClose",
    "GuiSfntSimpleGlyphPathSinkTailAction",
    "GuiSfntSimpleGlyphPathSinkStep",
    "gui_sfnt_simple_glyph_path_sink_step_from_contour_step",
    "gui_sfnt_lookup_simple_glyph_path_sink_step",
    "`RejectUnsupported` は `SkipNoSegment OffCurveStart` だけ",
    "`Reject` なら常に `NoTailAction`",
    "`Continue` なら常に `NoTailAction`",
    "`EndContour` かつ `EmitCloseAfterFinalEvent` かつ primary が emit",
    "policy reject は `Result::Err` ではなく",
    "F4s の skipped public lookup fixture",
    "no fallback/no allocation/no renderer/no platform",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4t path sink ownership boundary contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4u:",
    "sfnt simple glyph path sink action selection projection",
    "GuiSfntSimpleGlyphPathSinkActionSlot",
    "GuiSfntSimpleGlyphPathSinkAction",
    "primary action projection は `EmitEvent` / `Reject` だけを返し、`NoAction` を返さない",
    "tail action projection は `NoTailAction -> NoAction`、`CloseContour -> CloseContour` だけを行う",
    "gui_sfnt_simple_glyph_path_sink_step_action_at",
    "gui_sfnt_lookup_simple_glyph_path_sink_action",
    "F4u は `Vec`、`push`、numeric action index",
    "byte-backed helper は F4t lookup にだけ委譲し",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4u path sink action projection contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4v:",
    "sfnt simple glyph path sink action traversal step",
    "GuiSfntSimpleGlyphPathSinkActionCursor",
    "GuiSfntSimpleGlyphPathSinkActionNext",
    "GuiSfntSimpleGlyphPathSinkActionStep",
    "gui_sfnt_simple_glyph_path_sink_action_next_from_step",
    "gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step",
    "`Primary` は action payload に関係なく同じ contour cursor の `Tail` へ進む",
    "`Tail` は action payload に関係なく `sink_step.source_step.next` に従う",
    "source_step.next = Continue next_cursor",
    "`gui_sfnt_simple_glyph_path_sink_step_action_at` を使い",
    "F4v は `Vec`、`push`、numeric action index",
    "byte-backed helper は F4t lookup にだけ委譲し",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4v path sink action traversal contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4w:",
    "sfnt simple glyph path sink action start cursor",
    "`edge 0` / `First` / `Primary`",
    "pure constructor と byte-backed validated entry point",
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor",
    "unchecked value constructor",
    "byte-backed helper は `gui_sfnt_lookup_simple_glyph_contour_span` を 1 回だけ呼び",
    "F4v action-step lookup、F4t sink-step lookup、F4s contour-step lookup",
    "追加 NEPL body に括弧がない",
    "contour `3`、edge `0`、event slot `First`、action slot `Primary`",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4w path sink action start cursor contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4x:",
    "sfnt simple glyph path sink action start step",
    "contour の first action step",
    "既存 action step lookup の Result 境界を再利用",
    "contour span 検証の二重実行を避ける",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step",
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy",
    "`Result::Err error` / `Result::Ok action_step`",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor",
    "contour `0`、edge `0`、event slot `First`、action slot `Primary`",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4x path sink action start step contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4y:",
    "sfnt simple glyph path sink action step advance",
    "GuiSfntSimpleGlyphPathSinkActionStepAdvance",
    "Continue GuiSfntSimpleGlyphPathSinkActionStep",
    "EndContour",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance",
    "gui_sfnt_simple_glyph_path_sink_action_step_next step",
    "Continue cursor",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy",
    "Result::Ok next_step",
    "GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour",
    "helper は action payload を見ない",
    "start cursor/start step helper",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4y path sink action step advance contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4z:",
    "sfnt simple glyph path sink action step item",
    "GuiSfntSimpleGlyphPathSinkActionStepItem",
    "gui_sfnt_simple_glyph_path_sink_action_step_item",
    "gui_sfnt_simple_glyph_path_sink_action_step_item_step",
    "gui_sfnt_simple_glyph_path_sink_action_step_item_advance",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item",
    "step GuiSfntSimpleGlyphPathSinkActionStep",
    "advance GuiSfntSimpleGlyphPathSinkActionStepAdvance",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy",
    "let stored_step %GuiSfntSimpleGlyphPathSinkActionStep *step",
    "helper は action payload を見ない",
    "start cursor/start step helper",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4z path sink action step item contract must mention ${fragment}`);
}
for (const fragment of [
    "Phase F4aa:",
    "sfnt simple glyph path sink action start item",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_item",
    "-> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntParseError",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy",
    "helper は action payload を見ない",
    "start cursor helper",
    "F4v action step lookup",
    "F4y advance helper",
    "hidden fallback",
]) {
    assert(implementationPlan.includes(fragment), `font implementation plan F4aa path sink action start item contract must mention ${fragment}`);
}

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
    /pub\s+struct\s+GuiSfntSimpleGlyphContourPoint:[\s\S]*span\s+%GuiSfntSimpleGlyphContourSpan[\s\S]*contour_point_index\s+%i32[\s\S]*point\s+%GuiSfntSimpleGlyphPoint/,
    "alloc/gui/font/sfnt/glyf must expose contour-local simple glyph points as typed nested data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_contour_point\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphContourPoint\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf contour point lookup must take borrowed ByteBuf, checked GuiGlyphId, contour index, and contour-local point index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphContourEdge:[\s\S]*start\s+%GuiSfntSimpleGlyphContourPoint[\s\S]*end\s+%GuiSfntSimpleGlyphContourPoint[\s\S]*edge_index\s+%i32[\s\S]*next_contour_point_index\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose contour edge topology as typed nested point data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_contour_edge\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphContourEdge\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf contour edge lookup must take borrowed ByteBuf, checked GuiGlyphId, contour index, and contour-local edge index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphCurveNoSegmentReason:[\s\S]*SinglePointContour[\s\S]*OffCurveStart[\s\S]*MissingLookahead/,
    "alloc/gui/font/sfnt/glyf must expose no-segment reasons as enum variants",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphCurveNoSegment:[\s\S]*edge\s+%GuiSfntSimpleGlyphContourEdge[\s\S]*reason\s+%GuiSfntSimpleGlyphCurveNoSegmentReason/,
    "alloc/gui/font/sfnt/glyf must expose no-segment payload as typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphLineSegment:[\s\S]*edge\s+%GuiSfntSimpleGlyphContourEdge[\s\S]*start_x2\s+%i32[\s\S]*start_y2\s+%i32[\s\S]*end_x2\s+%i32[\s\S]*end_y2\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose line segment doubled coordinates",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphQuadraticSegment:[\s\S]*edge\s+%GuiSfntSimpleGlyphContourEdge[\s\S]*lookahead\s+%GuiSfntSimpleGlyphContourPoint[\s\S]*start_x2\s+%i32[\s\S]*start_y2\s+%i32[\s\S]*control_x2\s+%i32[\s\S]*control_y2\s+%i32[\s\S]*end_x2\s+%i32[\s\S]*end_y2\s+%i32[\s\S]*end_is_implied\s+%bool/,
    "alloc/gui/font/sfnt/glyf must expose quadratic segment doubled coordinates and implied-end state",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphCurveSegment:[\s\S]*NoSegment\s+%GuiSfntSimpleGlyphCurveNoSegment[\s\S]*Line\s+%GuiSfntSimpleGlyphLineSegment[\s\S]*Quadratic\s+%GuiSfntSimpleGlyphQuadraticSegment/,
    "alloc/gui/font/sfnt/glyf must expose curve segment as payload enum states",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_classify_simple_glyph_curve_segment\s+%fn\s+GuiSfntSimpleGlyphContourEdge\s+fn\s+Option\s+GuiSfntSimpleGlyphContourPoint\s+GuiSfntSimpleGlyphCurveSegment/,
    "alloc/gui/font/sfnt/glyf must expose pure curve segment classifier over an edge and optional lookahead",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_curve_segment\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphCurveSegment\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf curve segment lookup must take borrowed ByteBuf, checked GuiGlyphId, contour index, and contour-local edge index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathMoveTo:[\s\S]*contour_index\s+%i32[\s\S]*edge_index\s+%i32[\s\S]*x2\s+%i32[\s\S]*y2\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose path MoveTo as compact typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathLineTo:[\s\S]*contour_index\s+%i32[\s\S]*edge_index\s+%i32[\s\S]*x2\s+%i32[\s\S]*y2\s+%i32/,
    "alloc/gui/font/sfnt/glyf must expose path LineTo as compact typed data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathQuadraticTo:[\s\S]*contour_index\s+%i32[\s\S]*edge_index\s+%i32[\s\S]*control_x2\s+%i32[\s\S]*control_y2\s+%i32[\s\S]*end_x2\s+%i32[\s\S]*end_y2\s+%i32[\s\S]*end_is_implied\s+%bool/,
    "alloc/gui/font/sfnt/glyf must expose path QuadraticTo as compact doubled-coordinate data",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSkipNoSegment:[\s\S]*contour_index\s+%i32[\s\S]*edge_index\s+%i32[\s\S]*reason\s+%GuiSfntSimpleGlyphCurveNoSegmentReason/,
    "alloc/gui/font/sfnt/glyf must expose explicit compact path SkipNoSegment payload",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathCommand:[\s\S]*MoveTo\s+%GuiSfntSimpleGlyphPathMoveTo[\s\S]*LineTo\s+%GuiSfntSimpleGlyphPathLineTo[\s\S]*QuadraticTo\s+%GuiSfntSimpleGlyphPathQuadraticTo[\s\S]*SkipNoSegment\s+%GuiSfntSimpleGlyphPathSkipNoSegment/,
    "alloc/gui/font/sfnt/glyf must expose path command as payload enum states",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathCommandPair:[\s\S]*move_command\s+%GuiSfntSimpleGlyphPathCommand[\s\S]*draw_command\s+%GuiSfntSimpleGlyphPathCommand/,
    "alloc/gui/font/sfnt/glyf must expose path command pair as a compact two-command value",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_curve_segment_move_to_command\s+%fn\s+&GuiSfntSimpleGlyphCurveSegment\s+GuiSfntSimpleGlyphPathCommand/,
    "alloc/gui/font/sfnt/glyf must expose pure move-to path command projection for one curve segment",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_curve_segment_draw_command\s+%fn\s+&GuiSfntSimpleGlyphCurveSegment\s+GuiSfntSimpleGlyphPathCommand/,
    "alloc/gui/font/sfnt/glyf must expose pure draw path command projection for one curve segment",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_move_to_command\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphPathCommand\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf move command lookup must take borrowed ByteBuf, checked GuiGlyphId, contour index, and contour-local edge index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_draw_command\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphPathCommand\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf draw command lookup must take borrowed ByteBuf, checked GuiGlyphId, contour index, and contour-local edge index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_curve_segment_path_command_pair\s+%fn\s+&GuiSfntSimpleGlyphCurveSegment\s+GuiSfntSimpleGlyphPathCommandPair/,
    "alloc/gui/font/sfnt/glyf must expose pure path command pair projection for one curve segment",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_lookup_simple_glyph_path_command_pair\s+%fn\s+&ByteBuf\s+fn\s+Option\s+i32\s+fn\s+GuiGlyphId\s+fn\s+i32\s+fn\s+i32\s+Result\s+GuiSfntSimpleGlyphPathCommandPair\s+GuiSfntParseError/,
    "alloc/gui/font/sfnt/glyf path command pair lookup must take borrowed ByteBuf, checked GuiGlyphId, contour index, and contour-local edge index",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkEvent:\s+Command\s+%GuiSfntSimpleGlyphPathCommand/,
    "alloc/gui/font/sfnt/glyf F4p must expose sink event as a thin path command wrapper",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkEvent:[\s\S]*\b(?:MoveTo|LineTo|QuadraticTo|SkipNoSegment)\s+%GuiSfntSimpleGlyphPath/,
    "alloc/gui/font/sfnt/glyf F4p must not duplicate path command payload variants in the sink event type",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkEventPair:[\s\S]*first_event\s+%GuiSfntSimpleGlyphPathSinkEvent[\s\S]*second_event\s+%GuiSfntSimpleGlyphPathSinkEvent/,
    "alloc/gui/font/sfnt/glyf F4p must expose path sink event pair as a compact two-event value",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_command_pair_sink_event_pair\s+%fn\s+&GuiSfntSimpleGlyphPathCommandPair\s+GuiSfntSimpleGlyphPathSinkEventPair/,
    "alloc/gui/font/sfnt/glyf F4p must expose pure command-pair to sink-event-pair adapter",
);
assertMatch(
    implementationPlan,
    /Phase F4q:[\s\S]*GuiSfntSimpleGlyphPathSinkEventKind[\s\S]*SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason[\s\S]*contour_index[\s\S]*edge_index[\s\S]*gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair[\s\S]*Vec GuiSfntSimpleGlyphPathSinkEventKind/,
    "font implementation plan must define F4q path sink event kind classification with no payload duplication or allocation",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkEventKind:\s+MoveTo\s+LineTo\s+QuadraticTo\s+SkipNoSegment\s+%GuiSfntSimpleGlyphCurveNoSegmentReason/,
    "alloc/gui/font/sfnt/glyf F4q must expose sink event kind as a dispatch-only enum",
);
const pathSinkEventKindDeclStart = allocFontSfntGlyfImpl.indexOf("pub enum GuiSfntSimpleGlyphPathSinkEventKind:");
const pathSinkEventKindDeclEnd = allocFontSfntGlyfImpl.indexOf("impl Clone for GuiSfntSimpleGlyphPathSinkEventKind:", pathSinkEventKindDeclStart);
assert(pathSinkEventKindDeclStart >= 0 && pathSinkEventKindDeclEnd > pathSinkEventKindDeclStart, "alloc/gui/font/sfnt/glyf F4q sink event kind declaration must be present");
const pathSinkEventKindDecl = allocFontSfntGlyfImpl.slice(pathSinkEventKindDeclStart, pathSinkEventKindDeclEnd);
assertNoMatch(
    pathSinkEventKindDecl,
    /\b(?:contour_index|edge_index|x2|y2|control_x2|control_y2|end_x2|end_y2)\b|%GuiSfntSimpleGlyphPath(?:MoveTo|LineTo|QuadraticTo|SkipNoSegment)/,
    "alloc/gui/font/sfnt/glyf F4q sink event kind must not duplicate path command coordinates or source indices",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkEventKindPair:[\s\S]*first_kind\s+%GuiSfntSimpleGlyphPathSinkEventKind[\s\S]*second_kind\s+%GuiSfntSimpleGlyphPathSinkEventKind/,
    "alloc/gui/font/sfnt/glyf F4q must expose path sink event kind pair as a compact two-kind value",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair\s+%fn\s+&GuiSfntSimpleGlyphPathSinkEventPair\s+GuiSfntSimpleGlyphPathSinkEventKindPair/,
    "alloc/gui/font/sfnt/glyf F4q must expose pure event-pair to kind-pair adapter",
);
assertMatch(
    implementationPlan,
    /Phase F4r:[\s\S]*GuiSfntSimpleGlyphPathSinkEventSlot[\s\S]*gui_sfnt_simple_glyph_path_sink_event_pair_event_at[\s\S]*gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at[\s\S]*gui_sfnt_simple_glyph_path_sink_event_pair_kind_at[\s\S]*First[\s\S]*Second[\s\S]*`i32` event index[\s\S]*Option[\s\S]*Result[\s\S]*Vec[\s\S]*push/,
    "font implementation plan must define F4r path sink event slot selection without numeric index, allocation, or fallible selection",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkEventSlot:\s+First\s+Second/,
    "alloc/gui/font/sfnt/glyf F4r must expose a two-state slot enum",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkEventSlot:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkEventSlot:/,
    "alloc/gui/font/sfnt/glyf F4r slot enum must implement Clone and Copy",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_event_pair_event_at\s+%fn\s+&GuiSfntSimpleGlyphPathSinkEventPair\s+fn\s+GuiSfntSimpleGlyphPathSinkEventSlot\s+GuiSfntSimpleGlyphPathSinkEvent/,
    "alloc/gui/font/sfnt/glyf F4r must expose event pair slot selection",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at\s+%fn\s+&GuiSfntSimpleGlyphPathSinkEventKindPair\s+fn\s+GuiSfntSimpleGlyphPathSinkEventSlot\s+GuiSfntSimpleGlyphPathSinkEventKind/,
    "alloc/gui/font/sfnt/glyf F4r must expose kind pair slot selection",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_event_pair_kind_at\s+%fn\s+&GuiSfntSimpleGlyphPathSinkEventPair\s+fn\s+GuiSfntSimpleGlyphPathSinkEventSlot\s+GuiSfntSimpleGlyphPathSinkEventKind/,
    "alloc/gui/font/sfnt/glyf F4r must expose event pair slot-to-kind selection",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_read_index_to_loc_format[\s\S]*lt\s+gui_sfnt_table_record_length\s+&head\s+52[\s\S]*add\s+gui_sfnt_table_record_offset\s+&head\s+50/,
    "alloc/gui/font/sfnt/glyf must read head.indexToLocFormat only after head length 52",
);
assert(
    allocFontSfntGlyfImpl.includes("not or eq format 0 eq format 1") &&
        allocFontSfntGlyfImpl.includes("GuiSfntParseErrorKind::UnsupportedLocaFormat"),
    "alloc/gui/font/sfnt/glyf must reject unsupported loca formats as typed unsupported",
);
const readU32I32Be = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_read_u32_i32_be");
assert(
    readU32I32Be.includes("high_byte_limit %i32 add 64 64") &&
        readU32I32Be.includes("ge b0 high_byte_limit") &&
        readU32I32Be.includes("Result::Err gui_sfnt_parse_error kind offset"),
    "alloc/gui/font/sfnt/glyf must reject long loca offsets outside i32 range",
);
const locaRequiredLength = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_loca_required_length_is_valid");
assert(
    locaRequiredLength.includes("entry_count %i32 add num_glyphs 1") &&
        locaRequiredLength.includes("eq format 0") &&
        locaRequiredLength.includes("mul entry_count 2") &&
        locaRequiredLength.includes("eq format 1") &&
        locaRequiredLength.includes("mul entry_count 4"),
    "alloc/gui/font/sfnt/glyf must validate declared loca length for short and long formats",
);
const checkedGlyphRaw = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_checked_glyph_raw");
assert(
    checkedGlyphRaw.includes("le raw 0") &&
        checkedGlyphRaw.includes("ge raw num_glyphs") &&
        checkedGlyphRaw.includes("GuiSfntParseErrorKind::MissingGlyphOutline"),
    "alloc/gui/font/sfnt/glyf must reject glyph 0 and glyphs outside maxp.numGlyphs",
);
const validateGlyfOffsets = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_validate_offsets");
assert(
    validateGlyfOffsets.includes("gt start end") &&
        validateGlyfOffsets.includes("gt end gui_sfnt_table_record_length &glyf") &&
        validateGlyfOffsets.includes("eq start end") &&
        validateGlyfOffsets.includes("MissingGlyphOutline") &&
        validateGlyfOffsets.includes("lt sub end start 10"),
    "alloc/gui/font/sfnt/glyf must validate glyf declared bounds and empty/short glyph ranges",
);
const boundsFromHeader = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_bounds_from_header");
assert(
    boundsFromHeader.includes("add file_offset 2") &&
        boundsFromHeader.includes("add file_offset 4") &&
        boundsFromHeader.includes("add file_offset 6") &&
        boundsFromHeader.includes("add file_offset 8") &&
        boundsFromHeader.includes("or gt x_min x_max gt y_min y_max"),
    "alloc/gui/font/sfnt/glyf must read x/y bounds from the glyf header and reject inverted bounds",
);
const readLastEndpoint = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_read_last_endpoint");
assert(
    readLastEndpoint.includes("not gui_sfnt_glyf_glyph_relative_range_is_valid start end endpoint_offset 2") &&
        readLastEndpoint.includes("le endpoint previous_endpoint"),
    "alloc/gui/font/sfnt/glyf must validate simple glyph endpoints inside glyph range and strict increasing",
);
const topologyFromSpan = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_topology_from_span");
assert(
    topologyFromSpan.includes("lt contour_count 0") &&
        topologyFromSpan.includes("GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat") &&
        topologyFromSpan.includes("eq contour_count 0") &&
        topologyFromSpan.includes("GuiSfntParseErrorKind::MissingGlyphOutline") &&
        topologyFromSpan.includes("instruction_length_offset %i32 add endpoint_array_offset endpoint_array_length") &&
        topologyFromSpan.includes("instruction_start %i32 add instruction_length_offset 2") &&
        topologyFromSpan.includes("point_data_offset %i32 add instruction_start instruction_length") &&
        topologyFromSpan.includes("point_data_length %i32 sub end point_data_offset") &&
        topologyFromSpan.includes("le point_data_length 0"),
    "alloc/gui/font/sfnt/glyf must split contour count errors and validate instruction/point data range",
);
const scanFlagStream = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_scan_flag_stream");
assert(
    scanFlagStream.includes("eq logical_count point_count") &&
        scanFlagStream.includes("sub cursor flag_start") &&
        scanFlagStream.includes("gui_sfnt_glyf_flag_has_bit flag 8") &&
        scanFlagStream.includes("repeat_count_offset %i32 add cursor 1") &&
        scanFlagStream.includes("run_count %i32 add repeat_count 1") &&
        scanFlagStream.includes("gt next_logical_count point_count") &&
        scanFlagStream.includes("GuiSfntParseErrorKind::MalformedGlyfRecord"),
    "alloc/gui/font/sfnt/glyf must scan raw flag bytes with repeat count semantics and reject repeat overrun",
);
const flagXByteLength = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_flag_x_byte_length");
const flagYByteLength = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_flag_y_byte_length");
assert(
    flagXByteLength.includes("gui_sfnt_glyf_flag_has_bit flag 2") &&
        flagXByteLength.includes("gui_sfnt_glyf_flag_has_bit flag 16") &&
        flagXByteLength.includes("2") &&
        flagYByteLength.includes("gui_sfnt_glyf_flag_has_bit flag 4") &&
        flagYByteLength.includes("gui_sfnt_glyf_flag_has_bit flag 32") &&
        flagYByteLength.includes("2"),
    "alloc/gui/font/sfnt/glyf must derive x/y coordinate byte lengths from short and same bits",
);
const pointStreamFromTopology = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_point_stream_from_topology");
for (const fragment of [
    "flag_data_offset %i32 gui_sfnt_simple_glyph_topology_point_data_offset &topology",
    "flag_data_length %i32 gui_sfnt_simple_glyph_flag_scan_raw_length &scan",
    "x_data_offset %i32 add flag_data_offset flag_data_length",
    "y_data_offset %i32 add x_data_offset x_data_length",
    "trailing_data_offset %i32 add y_data_offset y_data_length",
    "trailing_data_length %i32 sub point_data_end trailing_data_offset",
    "lt trailing_data_length 0",
]) {
    assert(pointStreamFromTopology.includes(fragment), `alloc/gui/font/sfnt/glyf point stream offset derivation must include ${fragment}`);
}
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
assertMatch(
    allocFontSfntGlyfImpl,
    /gui_sfnt_glyf_simple_contour_point_with_tables[\s\S]*gui_sfnt_glyf_simple_contour_span_with_tables[\s\S]*or\s+lt\s+contour_point_index\s+0\s+ge\s+contour_point_index\s+span_point_count[\s\S]*GuiSfntParseErrorKind::MissingGlyphOutline[\s\S]*absolute_point_index\s+%i32\s+add\s+gui_sfnt_simple_glyph_contour_span_start_point_index\s+&span\s+contour_point_index[\s\S]*gui_sfnt_glyf_simple_point_with_tables/,
    "alloc/gui/font/sfnt/glyf contour point lookup must validate local index before point decode and compute the absolute point index from the span start",
);
const contourEdgeWithTables = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_simple_contour_edge_with_tables");
for (const fragment of [
    "gui_sfnt_glyf_simple_contour_span_with_tables",
    "or lt edge_index 0 ge edge_index span_point_count",
    "GuiSfntParseErrorKind::MissingGlyphOutline",
    "eq add edge_index 1 span_point_count",
    "then:\n                            0",
    "else:\n                            add edge_index 1",
    "gui_sfnt_glyf_simple_contour_point_with_tables",
    "gui_sfnt_simple_glyph_contour_edge",
]) {
    assert(contourEdgeWithTables.includes(fragment), `alloc/gui/font/sfnt/glyf contour edge helper must include ${fragment}`);
}
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
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphContourPoint\b|\bpush\s+.*GuiSfntSimpleGlyphContourPoint\b/,
    "alloc/gui/font/sfnt/glyf F4j must not allocate or build a full contour point Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphContourEdge\b|\bpush\s+.*GuiSfntSimpleGlyphContourEdge\b/,
    "alloc/gui/font/sfnt/glyf F4k must not allocate or build a full contour edge Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphCurveSegment\b|\bpush\s+.*GuiSfntSimpleGlyphCurveSegment\b/,
    "alloc/gui/font/sfnt/glyf F4l must not allocate or build a full curve segment Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphPathCommand\b|\bpush\s+.*GuiSfntSimpleGlyphPathCommand\b/,
    "alloc/gui/font/sfnt/glyf F4m must not allocate or build a full path command Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphPathCommandPair\b|\bpush\s+.*GuiSfntSimpleGlyphPathCommandPair\b/,
    "alloc/gui/font/sfnt/glyf F4o must not allocate or build a full path command pair Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /\bVec\s+GuiSfntSimpleGlyphPathSinkEvent\b|\bpush\s+.*GuiSfntSimpleGlyphPathSinkEvent\b/,
    "alloc/gui/font/sfnt/glyf F4p must not allocate or build a full path sink event Vec",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPath(?:MoveTo|LineTo|QuadraticTo|SkipNoSegment):\s*\n\s+(?:edge|line|quadratic|no_segment)\s+%GuiSfntSimpleGlyph/,
    "alloc/gui/font/sfnt/glyf F4m path command payloads must stay compact instead of storing full segment values",
);
const pathMoveProjection = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_curve_segment_move_to_command");
for (const fragment of [
    "GuiSfntSimpleGlyphCurveSegment::NoSegment",
    "GuiSfntSimpleGlyphPathCommand::SkipNoSegment",
    "GuiSfntSimpleGlyphCurveSegment::Line",
    "GuiSfntSimpleGlyphPathCommand::MoveTo",
    "GuiSfntSimpleGlyphCurveSegment::Quadratic",
]) {
    assert(pathMoveProjection.includes(fragment), `alloc/gui/font/sfnt/glyf move path command projection must include ${fragment}`);
}
const pathDrawProjection = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_curve_segment_draw_command");
for (const fragment of [
    "GuiSfntSimpleGlyphCurveSegment::NoSegment",
    "GuiSfntSimpleGlyphPathCommand::SkipNoSegment",
    "GuiSfntSimpleGlyphCurveSegment::Line",
    "GuiSfntSimpleGlyphPathCommand::LineTo",
    "GuiSfntSimpleGlyphCurveSegment::Quadratic",
    "GuiSfntSimpleGlyphPathCommand::QuadraticTo",
]) {
    assert(pathDrawProjection.includes(fragment), `alloc/gui/font/sfnt/glyf draw path command projection must include ${fragment}`);
}
const pathCommandProjection = `${pathMoveProjection}\n${pathDrawProjection}`;
assertNoMatch(
    pathCommandProjection,
    /\b(?:Option|Result|command_index)\b/,
    "alloc/gui/font/sfnt/glyf F4m path command projections must return direct typed commands without invalid-index state",
);
assertNoMatch(
    pathCommandProjection,
    /\b(?:gui_sfnt_parse_metadata|gui_sfnt_lookup_|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4m projection must stay pure and must not parse metadata, render, rasterize, or call host/platform text APIs",
);
const moveCommandLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_move_to_command");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index",
    "Result::Err error",
    "Result::Ok segment",
    "gui_sfnt_simple_glyph_curve_segment_move_to_command &segment",
    "Result::Ok command",
]) {
    assert(moveCommandLookup.includes(fragment), `alloc/gui/font/sfnt/glyf move command lookup must include ${fragment}`);
}
assertNoMatch(
    moveCommandLookup,
    /\b(?:gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_curve_segment_with_tables|gui_sfnt_glyf_simple_contour_|gui_sfnt_glyf_simple_point_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4n move command lookup must stay a thin F4l/F4m composition layer",
);
assertNoMatch(
    moveCommandLookup,
    /\bgui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_span|contour_point|contour_edge)\b/,
    "alloc/gui/font/sfnt/glyf F4n move command lookup must not bypass F4l through lower public lookup helpers",
);
const drawCommandLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_draw_command");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index",
    "Result::Err error",
    "Result::Ok segment",
    "gui_sfnt_simple_glyph_curve_segment_draw_command &segment",
    "Result::Ok command",
]) {
    assert(drawCommandLookup.includes(fragment), `alloc/gui/font/sfnt/glyf draw command lookup must include ${fragment}`);
}
assertNoMatch(
    drawCommandLookup,
    /\b(?:gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_curve_segment_with_tables|gui_sfnt_glyf_simple_contour_|gui_sfnt_glyf_simple_point_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4n draw command lookup must stay a thin F4l/F4m composition layer",
);
assertNoMatch(
    drawCommandLookup,
    /\bgui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_span|contour_point|contour_edge)\b/,
    "alloc/gui/font/sfnt/glyf F4n draw command lookup must not bypass F4l through lower public lookup helpers",
);
const pathCommandPairProjection = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_curve_segment_path_command_pair");
for (const fragment of [
    "gui_sfnt_simple_glyph_curve_segment_move_to_command segment",
    "gui_sfnt_simple_glyph_curve_segment_draw_command segment",
    "gui_sfnt_simple_glyph_path_command_pair move_command draw_command",
]) {
    assert(pathCommandPairProjection.includes(fragment), `alloc/gui/font/sfnt/glyf path command pair projection must include ${fragment}`);
}
assertNoMatch(
    pathCommandPairProjection,
    /\b(?:Option|Result|command_index|count|next|current_point|Vec|push|gui_sfnt_parse_metadata|gui_sfnt_lookup_|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4o pure pair projection must stay O(1), direct, and renderer/platform independent",
);
const pathCommandPairLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_command_pair");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index",
    "Result::Err error",
    "Result::Ok segment",
    "gui_sfnt_simple_glyph_curve_segment_path_command_pair &segment",
    "Result::Ok pair",
]) {
    assert(pathCommandPairLookup.includes(fragment), `alloc/gui/font/sfnt/glyf path command pair lookup must include ${fragment}`);
}
assert(
    (pathCommandPairLookup.match(/\bgui_sfnt_lookup_simple_glyph_curve_segment\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4o pair lookup must call curve segment lookup exactly once",
);
assertNoMatch(
    pathCommandPairLookup,
    /\b(?:gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_curve_segment_with_tables|gui_sfnt_glyf_simple_contour_|gui_sfnt_glyf_simple_point_|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_lookup_simple_glyph_move_to_command|gui_sfnt_lookup_simple_glyph_draw_command|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4o pair lookup must not decode twice, bypass F4l, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathCommandPairLookup,
    /\bgui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_span|contour_point|contour_edge)\b/,
    "alloc/gui/font/sfnt/glyf F4o pair lookup must not bypass F4l through lower public lookup helpers",
);
const pathCommandSinkEvent = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_command_sink_event");
assert(
    pathCommandSinkEvent.includes("GuiSfntSimpleGlyphPathSinkEvent::Command command"),
    "alloc/gui/font/sfnt/glyf F4p path command sink event wrapper must keep the existing path command payload",
);
assertNoMatch(
    pathCommandSinkEvent,
    /\b(?:Option|Result|command_index|count|next|current_point|Vec|push|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4p command event wrapper must stay total, allocation-free, and renderer/platform independent",
);
const pathCommandSinkEventPair = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_command_pair_sink_event_pair");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_command_pair_move_command pair",
    "gui_sfnt_simple_glyph_path_command_pair_draw_command pair",
    "gui_sfnt_simple_glyph_path_command_sink_event move_command",
    "gui_sfnt_simple_glyph_path_command_sink_event draw_command",
    "gui_sfnt_simple_glyph_path_sink_event_pair first_event second_event",
]) {
    assert(pathCommandSinkEventPair.includes(fragment), `alloc/gui/font/sfnt/glyf F4p sink event pair adapter must include ${fragment}`);
}
assertNoMatch(
    pathCommandSinkEventPair,
    /\b(?:Option|Result|command_index|count|next|current_point|Vec|push|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|raster|Raster|platform|Canvas|DOM|FontFace|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4p sink event pair adapter must not lookup, parse, allocate, render, rasterize, or call host/platform APIs",
);
const pathSinkEventKind = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_event_kind");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_event_command event",
    "GuiSfntSimpleGlyphPathCommand::MoveTo",
    "GuiSfntSimpleGlyphPathSinkEventKind::MoveTo",
    "GuiSfntSimpleGlyphPathCommand::LineTo",
    "GuiSfntSimpleGlyphPathSinkEventKind::LineTo",
    "GuiSfntSimpleGlyphPathCommand::QuadraticTo",
    "GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo",
    "GuiSfntSimpleGlyphPathCommand::SkipNoSegment",
    "gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip",
    "GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason",
]) {
    assert(pathSinkEventKind.includes(fragment), `alloc/gui/font/sfnt/glyf F4q sink event kind helper must include ${fragment}`);
}
assertNoMatch(
    pathSinkEventKind,
    /\b(?:Option|Result|command_index|count|next|current_point|Vec|push|closure|winding|fill|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4q sink event kind helper must stay total, dispatch-only, and renderer/platform independent",
);
assertNoMatch(
    pathSinkEventKind,
    /\b(?:gui_sfnt_simple_glyph_path_(?:move_to|line_to|quadratic_to)_(?:contour_index|edge_index|x2|y2|control_x2|control_y2|end_x2|end_y2))\b/,
    "alloc/gui/font/sfnt/glyf F4q sink event kind helper must not read path command coordinates or source indices",
);
const pathSinkEventPairKindPair = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair",
    "gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair",
    "gui_sfnt_simple_glyph_path_sink_event_kind &first_event",
    "gui_sfnt_simple_glyph_path_sink_event_kind &second_event",
    "gui_sfnt_simple_glyph_path_sink_event_kind_pair first_kind second_kind",
]) {
    assert(pathSinkEventPairKindPair.includes(fragment), `alloc/gui/font/sfnt/glyf F4q sink event kind pair adapter must include ${fragment}`);
}
assertNoMatch(
    pathSinkEventPairKindPair,
    /\b(?:Option|Result|command_index|count|next|current_point|Vec|push|closure|winding|fill|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_command_pair_|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4q sink event kind pair adapter must use only event pair accessors and kind helper",
);
const pathSinkEventPairEventAt = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_event_pair_event_at");
for (const fragment of [
    "match slot:",
    "GuiSfntSimpleGlyphPathSinkEventSlot::First:",
    "gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair",
    "GuiSfntSimpleGlyphPathSinkEventSlot::Second:",
    "gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair",
]) {
    assert(pathSinkEventPairEventAt.includes(fragment), `alloc/gui/font/sfnt/glyf F4r event slot selection must include ${fragment}`);
}
assertNoMatch(
    pathSinkEventPairEventAt,
    /\b(?:i32|Option|Result|command_index|count|next|current_point|Vec|push|closure|winding|fill|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4r event slot selection must stay total, typed, allocation-free, and renderer/platform independent",
);
const pathSinkEventKindPairKindAt = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at");
for (const fragment of [
    "match slot:",
    "GuiSfntSimpleGlyphPathSinkEventSlot::First:",
    "gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind pair",
    "GuiSfntSimpleGlyphPathSinkEventSlot::Second:",
    "gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind pair",
]) {
    assert(pathSinkEventKindPairKindAt.includes(fragment), `alloc/gui/font/sfnt/glyf F4r kind pair slot selection must include ${fragment}`);
}
assertNoMatch(
    pathSinkEventKindPairKindAt,
    /\b(?:i32|Option|Result|command_index|count|next|current_point|Vec|push|closure|winding|fill|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_event_kind\b|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4r kind pair slot selection must use only kind pair accessors",
);
const pathSinkEventPairKindAt = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_event_pair_kind_at");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at pair slot",
    "gui_sfnt_simple_glyph_path_sink_event_kind &event",
]) {
    assert(pathSinkEventPairKindAt.includes(fragment), `alloc/gui/font/sfnt/glyf F4r event pair slot kind selection must include ${fragment}`);
}
assertNoMatch(
    pathSinkEventPairKindAt,
    /\b(?:i32|Option|Result|command_index|count|next|current_point|Vec|push|closure|winding|fill|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair|gui_sfnt_simple_glyph_path_sink_event_kind_pair_|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4r event pair slot kind selection must compose event_at and event kind helper only",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathContourCursor:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*contour_index\s+%i32[\s\S]*edge_index\s+%i32[\s\S]*slot\s+%GuiSfntSimpleGlyphPathSinkEventSlot/,
    "alloc/gui/font/sfnt/glyf F4s must expose a typed path contour cursor",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathContourNext:\s+Continue\s+%GuiSfntSimpleGlyphPathContourCursor\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4s must expose typed Continue/EndContour next states",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathContourStep:[\s\S]*cursor\s+%GuiSfntSimpleGlyphPathContourCursor[\s\S]*event\s+%GuiSfntSimpleGlyphPathSinkEvent[\s\S]*kind\s+%GuiSfntSimpleGlyphPathSinkEventKind[\s\S]*next\s+%GuiSfntSimpleGlyphPathContourNext/,
    "alloc/gui/font/sfnt/glyf F4s must expose a contour step value with event, kind, and next",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathContourCursor:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathContourCursor:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathContourNext:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathContourNext:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathContourStep:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathContourStep:/,
    "alloc/gui/font/sfnt/glyf F4s cursor, next, and step values must implement Clone and Copy",
);
assertNoMatch(
    allocFontSfntGlyfImpl,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_contour_next_from_cursor/,
    "alloc/gui/font/sfnt/glyf F4s cursor next helper must stay private unless it returns Result",
);
const pathContourNextFromCursor = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_contour_next_from_cursor");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_contour_cursor_glyph cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_contour_index cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_edge_index cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_slot cursor",
    "match slot:",
    "GuiSfntSimpleGlyphPathSinkEventSlot::First:",
    "gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index edge_index GuiSfntSimpleGlyphPathSinkEventSlot::Second",
    "GuiSfntSimpleGlyphPathContourNext::Continue next_cursor",
    "GuiSfntSimpleGlyphPathSinkEventSlot::Second:",
    "eq add edge_index 1 span_point_count",
    "GuiSfntSimpleGlyphPathContourNext::EndContour",
    "gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index add edge_index 1 GuiSfntSimpleGlyphPathSinkEventSlot::First",
]) {
    assert(pathContourNextFromCursor.includes(fragment), `alloc/gui/font/sfnt/glyf F4s cursor next helper must include ${fragment}`);
}
assertNoMatch(
    pathContourNextFromCursor,
    /\b(?:pub|Option|Result|command_index|current_point|Vec|push|closure|winding|fill|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4s cursor next helper must stay private, pure, allocation-free, and renderer/platform independent",
);
const pathContourStepLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_contour_step");
assert(
    guiFontSfntPathTests.includes("path contour step public lookup follows cursor next contract") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none first_cursor") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none second_cursor") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none final_cursor") &&
        guiFontSfntPathTests.includes("GuiSfntParseErrorKind::MissingGlyphOutline"),
    "gui font sfnt path doctest must directly cover F4s public contour step lookup and its typed edge-out-of-range error",
);
for (const fragment of [
    "gui_sfnt_simple_glyph_path_contour_cursor_glyph &cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_contour_index &cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_edge_index &cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_slot &cursor",
    "gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index",
    "gui_sfnt_simple_glyph_contour_span_point_count &span",
    "gui_sfnt_lookup_simple_glyph_path_command_pair bytes face_index glyph contour_index edge_index",
    "gui_sfnt_simple_glyph_path_command_pair_sink_event_pair &pair",
    "gui_sfnt_simple_glyph_path_sink_event_pair_event_at &event_pair slot",
    "gui_sfnt_simple_glyph_path_sink_event_kind &event",
    "gui_sfnt_simple_glyph_path_contour_next_from_cursor &cursor span_point_count",
    "gui_sfnt_simple_glyph_path_contour_step cursor event kind next",
    "Result::Ok step",
]) {
    assert(pathContourStepLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4s contour step lookup must include ${fragment}`);
}
assertNoMatch(
    pathContourStepLookup,
    /\b(?:Option::None|Option::Some|Vec|push|command_index|current_point|closure|winding|fill|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4s contour step lookup must not fallback, allocate, bypass metadata, render, rasterize, or call host/platform APIs",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathOffCurveStartPolicy:\s+KeepTypedSkip\s+RejectUnsupported/,
    "alloc/gui/font/sfnt/glyf F4t must expose off-curve start policy as a two-state enum",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathClosurePolicy:\s+KeepOpen\s+EmitCloseAfterFinalEvent/,
    "alloc/gui/font/sfnt/glyf F4t must expose closure policy as a two-state enum",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkPolicy:[\s\S]*off_curve_start_policy\s+%GuiSfntSimpleGlyphPathOffCurveStartPolicy[\s\S]*closure_policy\s+%GuiSfntSimpleGlyphPathClosurePolicy/,
    "alloc/gui/font/sfnt/glyf F4t must expose sink policy as separate off-curve and closure policies",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkRejectReason:\s+UnsupportedOffCurveStart/,
    "alloc/gui/font/sfnt/glyf F4t must expose dedicated sink reject reasons",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkPrimaryAction:\s+EmitEvent\s+%GuiSfntSimpleGlyphPathSinkEvent\s+Reject\s+%GuiSfntSimpleGlyphPathSinkRejectReason/,
    "alloc/gui/font/sfnt/glyf F4t must expose primary action as emit-or-reject enum",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathContourClose:[\s\S]*glyph\s+%GuiGlyphId[\s\S]*contour_index\s+%i32/,
    "alloc/gui/font/sfnt/glyf F4t must expose close contour marker without coordinates",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkTailAction:\s+NoTailAction\s+CloseContour\s+%GuiSfntSimpleGlyphPathContourClose/,
    "alloc/gui/font/sfnt/glyf F4t must expose tail action as no-tail or close-contour enum",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkStep:[\s\S]*source_step\s+%GuiSfntSimpleGlyphPathContourStep[\s\S]*primary_action\s+%GuiSfntSimpleGlyphPathSinkPrimaryAction[\s\S]*tail_action\s+%GuiSfntSimpleGlyphPathSinkTailAction/,
    "alloc/gui/font/sfnt/glyf F4t must expose sink step with source step, primary action, and tail action",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathOffCurveStartPolicy:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathOffCurveStartPolicy:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathClosurePolicy:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathClosurePolicy:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkPolicy:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkPolicy:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkRejectReason:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkRejectReason:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkPrimaryAction:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkPrimaryAction:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathContourClose:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathContourClose:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkTailAction:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkTailAction:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkStep:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkStep:/,
    "alloc/gui/font/sfnt/glyf F4t sink policy/action values must implement Clone and Copy",
);
assert(
    guiFontSfntPathTests.includes("path sink policy keeps reject and close tail exclusive") &&
        guiFontSfntPathTests.includes("primary_is_reject_off_curve") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_step &bytes none final_cursor &sink_policy"),
    "gui font sfnt path doctest must cover F4t reject/close exclusivity, OffCurveStart-only reject policy, and byte-backed sink step lookup call",
);
const pathSinkPrimaryActionFromStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_primary_action_from_contour_step");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_contour_step_event step",
    "gui_sfnt_simple_glyph_path_contour_step_kind step",
    "GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent event",
    "GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:",
    "GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:",
    "GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:",
    "gui_sfnt_simple_glyph_path_sink_policy_off_curve_start_policy policy",
    "GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported:",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart",
    "GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:",
]) {
    assert(pathSinkPrimaryActionFromStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4t primary action helper must include ${fragment}`);
}
assertNoMatch(
    pathSinkPrimaryActionFromStep,
    /\b(?:Result|GuiSfntParseError|Vec|push|command_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4t primary action helper must stay pure, allocation-free, and policy-only",
);
const pathSinkTailActionFromStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_tail_action_from_contour_step");
for (const fragment of [
    "match *primary_action:",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject reason:",
    "GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent event:",
    "gui_sfnt_simple_glyph_path_contour_step_next step",
    "GuiSfntSimpleGlyphPathContourNext::Continue cursor:",
    "GuiSfntSimpleGlyphPathContourNext::EndContour:",
    "gui_sfnt_simple_glyph_path_sink_policy_closure_policy policy",
    "GuiSfntSimpleGlyphPathClosurePolicy::KeepOpen:",
    "GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent:",
    "gui_sfnt_simple_glyph_path_contour_step_cursor step",
    "gui_sfnt_simple_glyph_path_contour_cursor_glyph &cursor",
    "gui_sfnt_simple_glyph_path_contour_cursor_contour_index &cursor",
    "gui_sfnt_simple_glyph_path_contour_close glyph contour_index",
    "GuiSfntSimpleGlyphPathSinkTailAction::CloseContour close",
]) {
    assert(pathSinkTailActionFromStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4t tail action helper must include ${fragment}`);
}
assertNoMatch(
    pathSinkTailActionFromStep,
    /\b(?:Result|GuiSfntParseError|Vec|push|command_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4t tail action helper must not parse, allocate, render, rasterize, or call host/platform APIs",
);
const pathSinkStepFromContourStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_step_from_contour_step");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_primary_action_from_contour_step policy step",
    "gui_sfnt_simple_glyph_path_sink_tail_action_from_contour_step policy step &primary_action",
    "source_step %GuiSfntSimpleGlyphPathContourStep *step",
    "gui_sfnt_simple_glyph_path_sink_step source_step primary_action tail_action",
]) {
    assert(pathSinkStepFromContourStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4t sink step helper must include ${fragment}`);
}
assertNoMatch(
    pathSinkStepFromContourStep,
    /\b(?:Result|GuiSfntParseError|Vec|push|command_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4t sink step helper must compose pure helpers only",
);
const pathSinkStepLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_step");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_path_contour_step bytes face_index cursor",
    "Result::Err error",
    "Result::Ok contour_step",
    "gui_sfnt_simple_glyph_path_sink_step_from_contour_step policy &contour_step",
    "Result::Ok sink_step",
]) {
    assert(pathSinkStepLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4t sink step lookup must include ${fragment}`);
}
assert(
    (pathSinkStepLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_contour_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4t sink step lookup must call F4s contour step lookup exactly once",
);
assertNoMatch(
    pathSinkStepLookup,
    /\b(?:Option::None|Option::Some|Vec|push|command_index|current_point|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_span|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4t sink step lookup must only delegate to F4s lookup and pure sink-step conversion",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionSlot:\s+Primary\s+Tail/,
    "alloc/gui/font/sfnt/glyf F4u must expose primary/tail action slots as an enum",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkAction:\s+EmitEvent\s+%GuiSfntSimpleGlyphPathSinkEvent\s+Reject\s+%GuiSfntSimpleGlyphPathSinkRejectReason\s+CloseContour\s+%GuiSfntSimpleGlyphPathContourClose\s+NoAction/,
    "alloc/gui/font/sfnt/glyf F4u must expose unified sink action variants",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionSlot:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionSlot:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkAction:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkAction:/,
    "alloc/gui/font/sfnt/glyf F4u action slot and action values must implement Clone and Copy",
);
assert(
    guiFontSfntPathTests.includes("action_projection_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_step_action_at step GuiSfntSimpleGlyphPathSinkActionSlot::Primary") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_step_action_at step GuiSfntSimpleGlyphPathSinkActionSlot::Tail") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkAction::NoAction"),
    "gui font sfnt path doctest must cover F4u primary/tail action projection and explicit NoAction",
);
const pathSinkPrimaryActionAsAction = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_primary_action_as_action");
for (const fragment of [
    "match *action:",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent event:",
    "GuiSfntSimpleGlyphPathSinkAction::EmitEvent event",
    "GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject reason:",
    "GuiSfntSimpleGlyphPathSinkAction::Reject reason",
]) {
    assert(pathSinkPrimaryActionAsAction.includes(fragment), `alloc/gui/font/sfnt/glyf F4u primary action projection must include ${fragment}`);
}
assertNoMatch(
    pathSinkPrimaryActionAsAction,
    /\b(?:NoAction|Result|GuiSfntParseError|Vec|push|command_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4u primary action projection must not produce NoAction, parse, allocate, render, rasterize, or call host/platform APIs",
);
const pathSinkTailActionAsAction = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_tail_action_as_action");
for (const fragment of [
    "match *action:",
    "GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction:",
    "GuiSfntSimpleGlyphPathSinkAction::NoAction",
    "GuiSfntSimpleGlyphPathSinkTailAction::CloseContour close:",
    "GuiSfntSimpleGlyphPathSinkAction::CloseContour close",
]) {
    assert(pathSinkTailActionAsAction.includes(fragment), `alloc/gui/font/sfnt/glyf F4u tail action projection must include ${fragment}`);
}
assertNoMatch(
    pathSinkTailActionAsAction,
    /\b(?:Result|GuiSfntParseError|Vec|push|command_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4u tail action projection must not parse, allocate, render, rasterize, or call host/platform APIs",
);
const pathSinkStepActionAt = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_step_action_at");
for (const fragment of [
    "match slot:",
    "GuiSfntSimpleGlyphPathSinkActionSlot::Primary:",
    "gui_sfnt_simple_glyph_path_sink_step_primary_action step",
    "gui_sfnt_simple_glyph_path_sink_primary_action_as_action &primary_action",
    "GuiSfntSimpleGlyphPathSinkActionSlot::Tail:",
    "gui_sfnt_simple_glyph_path_sink_step_tail_action step",
    "gui_sfnt_simple_glyph_path_sink_tail_action_as_action &tail_action",
]) {
    assert(pathSinkStepActionAt.includes(fragment), `alloc/gui/font/sfnt/glyf F4u step action selection must include ${fragment}`);
}
assertNoMatch(
    pathSinkStepActionAt,
    /\b(?:Option|Result|GuiSfntParseError|Vec|push|command_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4u step action selection must stay total, pure, allocation-free, and platform-free",
);
const pathSinkActionLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index cursor policy",
    "Result::Err error",
    "Result::Ok sink_step",
    "gui_sfnt_simple_glyph_path_sink_step_action_at &sink_step slot",
    "Result::Ok action",
]) {
    assert(pathSinkActionLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4u sink action lookup must include ${fragment}`);
}
assert(
    (pathSinkActionLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4u sink action lookup must call F4t sink step lookup exactly once",
);
assertNoMatch(
    pathSinkActionLookup,
    /\b(?:Option::None|Option::Some|Vec|push|command_index|current_point|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_span|contour_point|contour_edge|curve_segment|path_command_pair|path_contour_step)|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4u sink action lookup must only delegate to F4t lookup and pure action selection",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionCursor:[\s\S]*contour_cursor\s+%GuiSfntSimpleGlyphPathContourCursor[\s\S]*action_slot\s+%GuiSfntSimpleGlyphPathSinkActionSlot/,
    "alloc/gui/font/sfnt/glyf F4v must expose sink action cursor as contour cursor plus action slot",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionNext:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionCursor\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4v must expose typed continue/end next state",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionStep:[\s\S]*cursor\s+%GuiSfntSimpleGlyphPathSinkActionCursor[\s\S]*sink_step\s+%GuiSfntSimpleGlyphPathSinkStep[\s\S]*action\s+%GuiSfntSimpleGlyphPathSinkAction[\s\S]*next\s+%GuiSfntSimpleGlyphPathSinkActionNext/,
    "alloc/gui/font/sfnt/glyf F4v must expose action step with cursor, sink step, action, and next",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionCursor:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionCursor:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionNext:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionNext:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionStep:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionStep:/,
    "alloc/gui/font/sfnt/glyf F4v traversal values must implement Clone and Copy",
);
assert(
    guiFontSfntPathTests.includes("action_traversal_ok") &&
        guiFontSfntPathTests.includes("action_step_primary_next_is_tail_same_cursor") &&
        guiFontSfntPathTests.includes("action_step_tail_continues_to_primary") &&
        guiFontSfntPathTests.includes("action_step_tail_ends_contour"),
    "gui font sfnt path doctest must cover F4v primary/tail traversal rules",
);
const pathSinkActionNextFromStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_next_from_step");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_step_source_step sink_step",
    "match action_slot:",
    "GuiSfntSimpleGlyphPathSinkActionSlot::Primary:",
    "gui_sfnt_simple_glyph_path_contour_step_cursor &source_step",
    "gui_sfnt_simple_glyph_path_sink_action_cursor contour_cursor GuiSfntSimpleGlyphPathSinkActionSlot::Tail",
    "GuiSfntSimpleGlyphPathSinkActionNext::Continue next_action_cursor",
    "GuiSfntSimpleGlyphPathSinkActionSlot::Tail:",
    "gui_sfnt_simple_glyph_path_contour_step_next &source_step",
    "GuiSfntSimpleGlyphPathContourNext::Continue next_cursor:",
    "gui_sfnt_simple_glyph_path_sink_action_cursor next_cursor GuiSfntSimpleGlyphPathSinkActionSlot::Primary",
    "GuiSfntSimpleGlyphPathContourNext::EndContour:",
    "GuiSfntSimpleGlyphPathSinkActionNext::EndContour",
]) {
    assert(pathSinkActionNextFromStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4v action next helper must include ${fragment}`);
}
assertNoMatch(
    pathSinkActionNextFromStep,
    /\b(?:gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|gui_sfnt_simple_glyph_path_sink_step_action_at|GuiSfntSimpleGlyphPathSinkPrimaryAction|GuiSfntSimpleGlyphPathSinkTailAction|GuiSfntSimpleGlyphPathSinkAction::|Result|GuiSfntParseError|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4v next helper must derive traversal only from action slot and source step next",
);
const pathSinkActionStepFromSinkStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_step_source_step sink_step",
    "gui_sfnt_simple_glyph_path_contour_step_cursor &source_step",
    "gui_sfnt_simple_glyph_path_sink_action_cursor contour_cursor action_slot",
    "gui_sfnt_simple_glyph_path_sink_step_action_at sink_step action_slot",
    "gui_sfnt_simple_glyph_path_sink_action_next_from_step sink_step action_slot",
    "stored_sink_step %GuiSfntSimpleGlyphPathSinkStep *sink_step",
    "gui_sfnt_simple_glyph_path_sink_action_step cursor stored_sink_step action next",
]) {
    assert(pathSinkActionStepFromSinkStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4v action step projection must include ${fragment}`);
}
assertNoMatch(
    pathSinkActionStepFromSinkStep,
    /\b(?:GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|GuiSfntSimpleGlyphPathSinkAction::|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4v action step projection must reuse F4u projection and avoid payload reclassification",
);
const pathSinkActionStepLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_step");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &cursor",
    "gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &cursor",
    "gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index contour_cursor policy",
    "Result::Err error",
    "Result::Ok sink_step",
    "gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step &sink_step action_slot",
    "Result::Ok action_step",
]) {
    assert(pathSinkActionStepLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4v sink action step lookup must include ${fragment}`);
}
assert(
    (pathSinkActionStepLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4v sink action step lookup must call F4t sink step lookup exactly once",
);
assertNoMatch(
    pathSinkActionStepLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_span|contour_point|contour_edge|curve_segment|path_command_pair|path_contour_step|path_sink_action\b)|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4v sink action step lookup must only delegate to F4t lookup and pure action-step projection",
);
assert(
    guiFontSfntPathTests.includes("start_cursor_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph 3") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkEventSlot::First") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionSlot::Primary"),
    "gui font sfnt path doctest must cover F4w start cursor at contour 3 edge 0 First Primary",
);
const pathSinkActionStartCursor = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_start_cursor");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index 0 GuiSfntSimpleGlyphPathSinkEventSlot::First",
    "gui_sfnt_simple_glyph_path_sink_action_cursor contour_cursor GuiSfntSimpleGlyphPathSinkActionSlot::Primary",
]) {
    assert(pathSinkActionStartCursor.includes(fragment), `alloc/gui/font/sfnt/glyf F4w pure start cursor must include ${fragment}`);
}
assertNoMatch(
    pathSinkActionStartCursor,
    /\b(?:Option|Result|GuiSfntParseError|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|GuiSfntSimpleGlyphPathSinkPolicy|gui_sfnt_simple_glyph_path_sink_step|gui_sfnt_simple_glyph_path_contour_step|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4w pure start cursor must remain unchecked, allocation-free, and platform-free",
);
assertNoMatch(
    pathSinkActionStartCursor,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4w pure start cursor body must preserve NEPL prefix style without parentheses",
);
const pathSinkActionStartCursorLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index",
    "Result::Err error",
    "Result::Ok _span",
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index",
    "Result::Ok start_cursor",
]) {
    assert(pathSinkActionStartCursorLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4w start cursor lookup must include ${fragment}`);
}
assert(
    (pathSinkActionStartCursorLookup.match(/\bgui_sfnt_lookup_simple_glyph_contour_span\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4w start cursor lookup must call contour span lookup exactly once",
);
assertNoMatch(
    pathSinkActionStartCursorLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair|path_contour_step|path_sink_step|path_sink_action\b)|gui_sfnt_lookup_simple_glyph_path_sink_action_step|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_classify_simple_glyph_curve_segment|GuiSfntSimpleGlyphPathSinkPolicy|gui_sfnt_simple_glyph_path_sink_step|gui_sfnt_simple_glyph_path_contour_step|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4w start cursor lookup must only validate contour span and delegate to pure start cursor",
);
assertNoMatch(
    pathSinkActionStartCursorLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4w start cursor lookup body must preserve NEPL prefix style without parentheses",
);
assert(
    guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_start_step &bytes none glyph 0 &sink_policy") &&
        guiFontSfntPathTests.includes("Result::Ok action_step") &&
        guiFontSfntPathTests.includes("start_step_ok") &&
        guiFontSfntPathTests.includes("sfnt_action_slot_is_primary") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkEventSlot::First"),
    "gui font sfnt path doctest must cover F4x byte-backed start action step fixture",
);
const pathSinkActionStartStepLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy",
    "Result::Err error",
    "Result::Ok action_step",
    "Result::Ok action_step",
]) {
    assert(pathSinkActionStartStepLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4x start step lookup must include ${fragment}`);
}
assert(
    (pathSinkActionStartStepLookup.match(/\bgui_sfnt_simple_glyph_path_sink_action_start_cursor\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4x start step lookup must build pure start cursor exactly once",
);
assert(
    (pathSinkActionStartStepLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4x start step lookup must call checked action-step lookup exactly once",
);
assertNoMatch(
    pathSinkActionStartStepLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|GuiSfntSimpleGlyphPathSinkPolicy::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4x start step lookup must only compose pure start cursor and checked action-step lookup",
);
assertNoMatch(
    pathSinkActionStartStepLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4x start step lookup body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionStepAdvance:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionStep\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4y must expose typed action-step advance result",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionStepAdvance:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionStepAdvance:/,
    "alloc/gui/font/sfnt/glyf F4y advance result must implement Clone and Copy",
);
assert(
    guiFontSfntPathTests.includes("action_step_advance_ok") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance &bytes none &action_step &sink_policy") &&
        guiFontSfntPathTests.includes("start_advance_ok") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionSlot::Tail"),
    "gui font sfnt path doctest must cover F4y terminal advance enum and byte-backed next action step",
);
const pathSinkActionStepAdvanceLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_step_next step",
    "GuiSfntSimpleGlyphPathSinkActionNext::Continue cursor:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy",
    "Result::Err error",
    "Result::Ok next_step",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step",
    "GuiSfntSimpleGlyphPathSinkActionNext::EndContour:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour",
]) {
    assert(pathSinkActionStepAdvanceLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4y action step advance lookup must include ${fragment}`);
}
assert(
    (pathSinkActionStepAdvanceLookup.match(/\bgui_sfnt_simple_glyph_path_sink_action_step_next\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4y action step advance lookup must read typed next state exactly once",
);
assert(
    (pathSinkActionStepAdvanceLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4y action step advance lookup must call checked action-step lookup exactly once",
);
assertNoMatch(
    pathSinkActionStepAdvanceLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action_start_step|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_action_step_action|gui_sfnt_simple_glyph_path_sink_step_action_at|gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4y action step advance lookup must only resolve the typed next state and checked next action step",
);
assertNoMatch(
    pathSinkActionStepAdvanceLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4y action step advance lookup body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionStepItem:\s+step\s+%GuiSfntSimpleGlyphPathSinkActionStep\s+advance\s+%GuiSfntSimpleGlyphPathSinkActionStepAdvance/,
    "alloc/gui/font/sfnt/glyf F4z must expose action-step item as step plus checked advance",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionStepItem:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionStepItem:/,
    "alloc/gui/font/sfnt/glyf F4z action-step item must implement Clone and Copy",
);
for (const pattern of [
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_action_step_item\s+%fn\s+GuiSfntSimpleGlyphPathSinkActionStep\s+fn\s+GuiSfntSimpleGlyphPathSinkActionStepAdvance\s+GuiSfntSimpleGlyphPathSinkActionStepItem/,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_action_step_item_step\s+%fn\s+&GuiSfntSimpleGlyphPathSinkActionStepItem\s+GuiSfntSimpleGlyphPathSinkActionStep/,
    /pub\s+fn\s+gui_sfnt_simple_glyph_path_sink_action_step_item_advance\s+%fn\s+&GuiSfntSimpleGlyphPathSinkActionStepItem\s+GuiSfntSimpleGlyphPathSinkActionStepAdvance/,
]) {
    assertMatch(allocFontSfntGlyfImpl, pattern, "alloc/gui/font/sfnt/glyf F4z must expose constructor and accessors for action-step item");
}
assert(
    guiFontSfntPathTests.includes("action_step_item_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_step_item item_step GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour") &&
        guiFontSfntPathTests.includes("action_step_item_keeps_step_and_end") &&
        guiFontSfntPathTests.includes("start_item_ok"),
    "gui font sfnt path doctest must cover F4z synthetic action-step item and F4aa start-item fixture",
);
const pathSinkActionStepItemLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy",
    "Result::Err error",
    "Result::Ok advance:",
    "let stored_step %GuiSfntSimpleGlyphPathSinkActionStep *step",
    "gui_sfnt_simple_glyph_path_sink_action_step_item stored_step advance",
    "Result::Ok item",
]) {
    assert(pathSinkActionStepItemLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4z action step item lookup must include ${fragment}`);
}
assert(
    (pathSinkActionStepItemLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_advance\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4z action step item lookup must call F4y advance lookup exactly once",
);
assertNoMatch(
    pathSinkActionStepItemLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action_start_step|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step\b|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_action_step_action|gui_sfnt_simple_glyph_path_sink_step_action_at|gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4z action step item lookup must only compose current step copy and F4y checked advance",
);
assertNoMatch(
    pathSinkActionStepItemLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4z action step item lookup body must preserve NEPL prefix style without parentheses",
);
assert(
    guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy") &&
        guiFontSfntPathTests.includes("let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step &item") &&
        guiFontSfntPathTests.includes("let advance %GuiSfntSimpleGlyphPathSinkActionStepAdvance gui_sfnt_simple_glyph_path_sink_action_step_item_advance &item") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step"),
    "gui font sfnt path doctest must cover F4aa byte-backed start action item fixture",
);
const pathSinkActionStartItemLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_start_item");
for (const fragment of [
    "gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy",
    "Result::Err error",
    "Result::Ok start_step:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy",
    "Result::Ok item",
]) {
    assert(pathSinkActionStartItemLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4aa action start item lookup must include ${fragment}`);
}
assert(
    (pathSinkActionStartItemLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4aa action start item lookup must call F4x start step lookup exactly once",
);
assert(
    (pathSinkActionStartItemLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4aa action start item lookup must call F4z action step item lookup exactly once",
);
assertNoMatch(
    pathSinkActionStartItemLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_action_step_action|gui_sfnt_simple_glyph_path_sink_step_action_at|gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4aa action start item lookup must only compose F4x start-step and F4z step-item lookups",
);
assertNoMatch(
    pathSinkActionStartItemLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4aa action start item lookup body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionItemNext:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionStepItem\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4ab must expose typed action item next result",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionItemNext:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionItemNext:/,
    "alloc/gui/font/sfnt/glyf F4ab action item next result must implement Clone and Copy",
);
assert(
    guiFontSfntPathTests.includes("terminal_item_next_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_item_next &bytes none &terminal_item &sink_policy") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour") &&
        guiFontSfntPathTests.includes("start_item_next_ok") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionSlot::Tail"),
    "gui font sfnt path doctest must cover F4ab terminal item next and byte-backed continued item next",
);
const pathSinkActionItemNextLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_item_next");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_step_item_advance item",
    "GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &next_step policy",
    "Result::Err error",
    "Result::Ok next_item",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item",
    "GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour",
]) {
    assert(pathSinkActionItemNextLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4ab action item next lookup must include ${fragment}`);
}
assert(
    (pathSinkActionItemNextLookup.match(/\bgui_sfnt_simple_glyph_path_sink_action_step_item_advance\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ab action item next lookup must read item advance exactly once",
);
assert(
    (pathSinkActionItemNextLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ab action item next lookup must call F4z action step item lookup exactly once",
);
assertNoMatch(
    pathSinkActionItemNextLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_simple_glyph_path_sink_action_step_item_step|gui_sfnt_simple_glyph_path_sink_action_step_cursor|gui_sfnt_simple_glyph_path_sink_action_step_action|gui_sfnt_simple_glyph_path_sink_action_step_next|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action_start_step|gui_sfnt_lookup_simple_glyph_path_sink_action_start_item|gui_sfnt_lookup_simple_glyph_path_sink_action_step\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_step_action_at|gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ab action item next lookup must only read item advance and resolve the next checked item",
);
assertNoMatch(
    pathSinkActionItemNextLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ab action item next lookup body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionConsumerItem:\s+action\s+%GuiSfntSimpleGlyphPathSinkAction\s+next\s+%GuiSfntSimpleGlyphPathSinkActionItemNext/,
    "alloc/gui/font/sfnt/glyf F4ac must expose typed action consumer item",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerItem:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerItem:/,
    "alloc/gui/font/sfnt/glyf F4ac action consumer item must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_item %fn GuiSfntSimpleGlyphPathSinkAction fn GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntSimpleGlyphPathSinkActionConsumerItem") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_item_action %fn &GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntSimpleGlyphPathSinkAction") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_item_next %fn &GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntSimpleGlyphPathSinkActionItemNext"),
    "alloc/gui/font/sfnt/glyf F4ac action consumer item must expose constructor and accessors",
);
assert(
    guiFontSfntPathTests.includes("start_consumer_item_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item &bytes none &item &sink_policy") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_item_action &consumer") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_item_next &consumer") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item"),
    "gui font sfnt path doctest must cover F4ac byte-backed action consumer item",
);
const pathSinkActionConsumerItemLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item");
for (const fragment of [
    "let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step item",
    "let action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_action_step_action &stored_step",
    "match gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy:",
    "Result::Err error",
    "Result::Ok next:",
    "let consumer_item %GuiSfntSimpleGlyphPathSinkActionConsumerItem gui_sfnt_simple_glyph_path_sink_action_consumer_item action next",
    "Result::Ok consumer_item",
]) {
    assert(pathSinkActionConsumerItemLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4ac action consumer item lookup must include ${fragment}`);
}
assert(
    (pathSinkActionConsumerItemLookup.match(/\bgui_sfnt_simple_glyph_path_sink_action_step_item_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ac action consumer item lookup must read item step exactly once",
);
assert(
    (pathSinkActionConsumerItemLookup.match(/\bgui_sfnt_simple_glyph_path_sink_action_step_action\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ac action consumer item lookup must read current action exactly once",
);
assert(
    (pathSinkActionConsumerItemLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_item_next\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ac action consumer item lookup must call F4ab item next lookup exactly once",
);
assertNoMatch(
    pathSinkActionConsumerItemLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_simple_glyph_path_sink_action_step_item_advance|gui_sfnt_simple_glyph_path_sink_action_step_cursor|gui_sfnt_simple_glyph_path_sink_action_step_sink_step|gui_sfnt_simple_glyph_path_sink_action_step_next|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action_start_step|gui_sfnt_lookup_simple_glyph_path_sink_action_start_item|gui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_step_action_at|gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ac action consumer item lookup must only copy current action and resolve F4ab next state",
);
assertNoMatch(
    pathSinkActionConsumerItemLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ac action consumer item lookup body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionConsumerItem\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4ad must expose typed action consumer item next result",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:/,
    "alloc/gui/font/sfnt/glyf F4ad action consumer item next result must implement Clone and Copy",
);
assert(
    guiFontSfntPathTests.includes("terminal_consumer_item_next_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next &bytes none &terminal_consumer &sink_policy") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour") &&
        guiFontSfntPathTests.includes("start_consumer_item_next_ok") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkAction::NoAction"),
    "gui font sfnt path doctest must cover F4ad terminal and byte-backed continued consumer item next",
);
const pathSinkActionConsumerItemNextLookup = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item",
    "GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy",
    "Result::Err error",
    "Result::Ok next_consumer_item",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer_item",
    "GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour",
]) {
    assert(pathSinkActionConsumerItemNextLookup.includes(fragment), `alloc/gui/font/sfnt/glyf F4ad action consumer item next lookup must include ${fragment}`);
}
assert(
    (pathSinkActionConsumerItemNextLookup.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_next\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ad action consumer item next lookup must read consumer item next exactly once",
);
assert(
    (pathSinkActionConsumerItemNextLookup.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ad action consumer item next lookup must call F4ac consumer item lookup exactly once",
);
assertNoMatch(
    pathSinkActionConsumerItemNextLookup,
    /\b(?:Option::None|Option::Some|Vec|push|action_index|command_index|loop_index|current_point|gui_sfnt_simple_glyph_path_sink_action_consumer_item_action|gui_sfnt_simple_glyph_path_sink_action_step_item_step|gui_sfnt_simple_glyph_path_sink_action_step_item_advance|gui_sfnt_simple_glyph_path_sink_action_step_cursor|gui_sfnt_simple_glyph_path_sink_action_step_action|gui_sfnt_simple_glyph_path_sink_action_step_sink_step|gui_sfnt_simple_glyph_path_sink_action_step_next|gui_sfnt_lookup_simple_glyph_path_sink_action_item_next|gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor|gui_sfnt_lookup_simple_glyph_path_sink_action_start_step|gui_sfnt_lookup_simple_glyph_path_sink_action_start_item|gui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step\b|gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance|gui_sfnt_lookup_simple_glyph_path_sink_action\b|gui_sfnt_lookup_simple_glyph_path_sink_step|gui_sfnt_lookup_simple_glyph_path_contour_step|gui_sfnt_lookup_simple_glyph_contour_span|gui_sfnt_parse_metadata|gui_sfnt_glyf_simple_|gui_sfnt_lookup_simple_glyph_(?:topology|point_stream|point|contour_point|contour_edge|curve_segment|path_command_pair)|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_step_action_at|gui_sfnt_simple_glyph_path_sink_step_primary_action|gui_sfnt_simple_glyph_path_sink_step_tail_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ad action consumer item next lookup must only read consumer next and resolve the next F4ac consumer item",
);
assertNoMatch(
    pathSinkActionConsumerItemNextLookup,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ad action consumer item next lookup body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action apply state[\s\S]*GuiSfntSimpleGlyphPathSinkActionApplyStatus[\s\S]*`NoAction` は silent no-op ではなく[\s\S]*traversal authority ではない/,
    "font spec must define F4ae action apply status as explicit consumed status without traversal authority",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action apply state[\s\S]*first boundary that consumes the current action payload[\s\S]*`Rejected` is a domain status, not `Result::Err`[\s\S]*`NoAction` is also a consumed status/,
    "font detailed design must define F4ae action consumption as typed status, not Result or no-op",
);
assertMatch(
    implementationPlan,
    /Phase F4ae: sfnt simple glyph path sink action apply state[\s\S]*`Reject` は `Result::Err` へ変換しない[\s\S]*`NoAction` は silent no-op ではない[\s\S]*traversal authority として使わない/,
    "font implementation plan must define F4ae action apply state and forbid hidden no-op traversal state",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionApplyStatus:\s+EmittedEvent\s+%GuiSfntSimpleGlyphPathSinkEvent\s+Rejected\s+%GuiSfntSimpleGlyphPathSinkRejectReason\s+ClosedContour\s+%GuiSfntSimpleGlyphPathContourClose\s+NoAction/,
    "alloc/gui/font/sfnt/glyf F4ae must expose typed action apply status",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionApplyState:\s+emitted_event_count\s+%i32\s+reject_count\s+%i32\s+close_contour_count\s+%i32\s+no_action_count\s+%i32/,
    "alloc/gui/font/sfnt/glyf F4ae must expose four-count action apply state",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionApplyStep:\s+state\s+%GuiSfntSimpleGlyphPathSinkActionApplyState\s+status\s+%GuiSfntSimpleGlyphPathSinkActionApplyStatus/,
    "alloc/gui/font/sfnt/glyf F4ae must expose state plus status apply step",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionApplyStatus:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionApplyStatus:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionApplyState:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionApplyState:[\s\S]*impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionApplyStep:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionApplyStep:/,
    "alloc/gui/font/sfnt/glyf F4ae apply status/state/step must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_apply_state %fn i32 fn i32 fn i32 fn i32 GuiSfntSimpleGlyphPathSinkActionApplyState") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_apply_state_new %fn void GuiSfntSimpleGlyphPathSinkActionApplyState") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_apply_step %fn GuiSfntSimpleGlyphPathSinkActionApplyState fn GuiSfntSimpleGlyphPathSinkActionApplyStatus GuiSfntSimpleGlyphPathSinkActionApplyStep") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_apply_step_state %fn &GuiSfntSimpleGlyphPathSinkActionApplyStep GuiSfntSimpleGlyphPathSinkActionApplyState") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_apply_step_status %fn &GuiSfntSimpleGlyphPathSinkActionApplyStep GuiSfntSimpleGlyphPathSinkActionApplyStatus"),
    "alloc/gui/font/sfnt/glyf F4ae must expose apply state/step constructors and accessors",
);
assert(
    guiFontSfntPathTests.includes("action_apply_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction") &&
        guiFontSfntPathTests.includes("apply_state_counts_match &no_action_apply_state 1 1 1 1"),
    "gui font sfnt path doctest must cover F4ae explicit action apply status and NoAction count",
);
const pathSinkActionApply = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action");
for (const fragment of [
    "let emitted_event_count %i32 gui_sfnt_simple_glyph_path_sink_action_apply_state_emitted_event_count &state",
    "let reject_count %i32 gui_sfnt_simple_glyph_path_sink_action_apply_state_reject_count &state",
    "let close_contour_count %i32 gui_sfnt_simple_glyph_path_sink_action_apply_state_close_contour_count &state",
    "let no_action_count %i32 gui_sfnt_simple_glyph_path_sink_action_apply_state_no_action_count &state",
    "GuiSfntSimpleGlyphPathSinkAction::EmitEvent event:",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent event",
    "GuiSfntSimpleGlyphPathSinkAction::Reject reason:",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected reason",
    "GuiSfntSimpleGlyphPathSinkAction::CloseContour close:",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour close",
    "GuiSfntSimpleGlyphPathSinkAction::NoAction:",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction",
]) {
    assert(pathSinkActionApply.includes(fragment), `alloc/gui/font/sfnt/glyf F4ae action apply helper must include ${fragment}`);
}
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_apply_state add emitted_event_count 1 reject_count close_contour_count no_action_count",
    "gui_sfnt_simple_glyph_path_sink_action_apply_state emitted_event_count add reject_count 1 close_contour_count no_action_count",
    "gui_sfnt_simple_glyph_path_sink_action_apply_state emitted_event_count reject_count add close_contour_count 1 no_action_count",
    "gui_sfnt_simple_glyph_path_sink_action_apply_state emitted_event_count reject_count close_contour_count add no_action_count 1",
]) {
    assert(pathSinkActionApply.includes(fragment), `alloc/gui/font/sfnt/glyf F4ae action apply helper must update exactly one count for ${fragment}`);
}
assertNoMatch(
    pathSinkActionApply,
    /\b(?:Result|Option|Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkActionConsumerItem|GuiSfntSimpleGlyphPathSinkActionItemNext|GuiSfntSimpleGlyphPathSinkActionConsumerItemNext|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ae action apply helper must not allocate, traverse, lookup, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionApply,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ae action apply helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer apply step[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep[\s\S]*GuiSfntSimpleGlyphPathSinkActionItemNext[\s\S]*`GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず/,
    "font spec must define F4af as consumer apply step that stores action item next without constructing consumer item next",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer apply step[\s\S]*composes F4ac and F4ae without taking over F4ad traversal[\s\S]*must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`/,
    "font detailed design must keep F4af from taking over F4ad traversal",
);
assertMatch(
    implementationPlan,
    /Phase F4af: sfnt simple glyph path sink action consumer apply step[\s\S]*`GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず[\s\S]*payload 解釈は F4ae helper だけに委譲する/,
    "font implementation plan must define F4af consumer apply step and forbid F4ad lookup/payload matching",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:\s+apply_step\s+%GuiSfntSimpleGlyphPathSinkActionApplyStep\s+next\s+%GuiSfntSimpleGlyphPathSinkActionItemNext/,
    "alloc/gui/font/sfnt/glyf F4af must expose apply step plus stored action item next",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:/,
    "alloc/gui/font/sfnt/glyf F4af consumer apply step must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step %fn GuiSfntSimpleGlyphPathSinkActionApplyStep fn GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep GuiSfntSimpleGlyphPathSinkActionApplyStep") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep GuiSfntSimpleGlyphPathSinkActionItemNext"),
    "alloc/gui/font/sfnt/glyf F4af must expose consumer apply step constructor and accessors",
);
assert(
    guiFontSfntPathTests.includes("consumer_apply_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply apply_state0 &apply_consumer_item") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour") &&
        guiFontSfntPathTests.includes("consumer_apply_next_is_end"),
    "gui font sfnt path doctest must cover F4af consumer apply step and stored EndContour next",
);
const pathSinkActionConsumerItemApply = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply");
for (const fragment of [
    "let action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_action_consumer_item_action item",
    "let next %GuiSfntSimpleGlyphPathSinkActionItemNext gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item",
    "let apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action",
    "gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step apply_step next",
]) {
    assert(pathSinkActionConsumerItemApply.includes(fragment), `alloc/gui/font/sfnt/glyf F4af consumer item apply helper must include ${fragment}`);
}
assert(
    (pathSinkActionConsumerItemApply.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_action\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4af consumer item apply helper must read consumer action exactly once",
);
assert(
    (pathSinkActionConsumerItemApply.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_next\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4af consumer item apply helper must read stored action item next exactly once",
);
assert(
    (pathSinkActionConsumerItemApply.match(/\bgui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4af consumer item apply helper must delegate payload consumption to F4ae exactly once",
);
assertNoMatch(
    pathSinkActionConsumerItemApply,
    /\b(?:Result|Option|Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkActionConsumerItemNext|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4af consumer item apply helper must not allocate, traverse, lookup, match payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumerItemApply,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4af consumer item apply helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer apply terminal[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal[\s\S]*Rejected[\s\S]*EndContour[\s\S]*NoAction/,
    "font spec must define F4ag terminal classification without treating NoAction as implicit terminal",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer apply terminal[\s\S]*Rejected` has priority over stored next state[\s\S]*must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`/,
    "font detailed design must keep F4ag as classification and forbid F4ad next lookup takeover",
);
assertMatch(
    implementationPlan,
    /Phase F4ag: sfnt simple glyph path sink action consumer apply terminal[\s\S]*`Continue` \/ `Rejected` \/ `EndContour`[\s\S]*F4ag は next consumer item lookup や traversal loop を実装しない/,
    "font implementation plan must define F4ag terminal classification and forbid traversal loop implementation",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep\s+Rejected\s+%GuiSfntSimpleGlyphPathSinkRejectReason\s+EndContour\s+%GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep/,
    "alloc/gui/font/sfnt/glyf F4ag must expose typed consumer apply terminal states",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:/,
    "alloc/gui/font/sfnt/glyf F4ag consumer apply terminal must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_reject_reason %fn &GuiSfntSimpleGlyphPathSinkActionApplyStatus Option GuiSfntSimpleGlyphPathSinkRejectReason") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal"),
    "alloc/gui/font/sfnt/glyf F4ag must expose reject reason and terminal classification helpers",
);
assert(
    guiFontSfntPathTests.includes("consumer_apply_terminal_ok") &&
        guiFontSfntPathTests.includes("consumer_apply_terminal_continues") &&
        guiFontSfntPathTests.includes("consumer_apply_terminal_rejects_off_curve") &&
        guiFontSfntPathTests.includes("consumer_apply_terminal_ends_contour") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step"),
    "gui font sfnt path doctest must cover F4ag continue, rejected, and end-contour classification",
);
const pathSinkActionConsumerApplyTerminalRejectReason = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_reject_reason");
for (const fragment of [
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected reason:",
    "Option::Some reason",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:",
    "GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:",
]) {
    assert(pathSinkActionConsumerApplyTerminalRejectReason.includes(fragment), `alloc/gui/font/sfnt/glyf F4ag reject reason helper must include ${fragment}`);
}
assertNoMatch(
    pathSinkActionConsumerApplyTerminalRejectReason,
    /\b(?:Result|Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkActionConsumerItem|GuiSfntSimpleGlyphPathSinkActionItemNext|GuiSfntSimpleGlyphPathSinkActionConsumerItemNext|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ag reject reason helper must not allocate, traverse, lookup, match payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumerApplyTerminalRejectReason,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ag reject reason helper body must preserve NEPL prefix style without parentheses",
);
const pathSinkActionConsumerApplyTerminalFromStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step");
for (const fragment of [
    "let apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step step",
    "let status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_apply_step_status &apply_step",
    "gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_reject_reason &status",
    "Option::Some reason:",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Rejected reason",
    "let next %GuiSfntSimpleGlyphPathSinkActionItemNext gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next step",
    "GuiSfntSimpleGlyphPathSinkActionItemNext::Continue _item:",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Continue *step",
    "GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::EndContour *step",
]) {
    assert(pathSinkActionConsumerApplyTerminalFromStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4ag terminal helper must include ${fragment}`);
}
assert(
    (pathSinkActionConsumerApplyTerminalFromStep.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ag terminal helper must read stored action item next exactly once",
);
assertNoMatch(
    pathSinkActionConsumerApplyTerminalFromStep,
    /\b(?:Result|Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkActionConsumerItem\b|GuiSfntSimpleGlyphPathSinkActionConsumerItemNext|gui_sfnt_lookup_|gui_sfnt_parse_metadata|gui_sfnt_glyf_|gui_sfnt_classify_simple_glyph_curve_segment|gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ag terminal helper must not allocate, traverse, lookup, match payload variants, re-apply payloads, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumerApplyTerminalFromStep,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ag terminal helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer apply advance[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance[\s\S]*F4ag terminal[\s\S]*F4ac lookup/,
    "font spec must define F4ah as one-step apply advance through F4ag terminal and F4ac lookup",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer apply advance[\s\S]*not a direct F4ad call[\s\S]*must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`/,
    "font detailed design must forbid F4ah from becoming a direct F4ad wrapper",
);
assertMatch(
    implementationPlan,
    /Phase F4ah: sfnt simple glyph path sink action consumer apply advance[\s\S]*F4ac lookup[\s\S]*F4ah は F4ad next helper や contour-wide loop を実装しない/,
    "font implementation plan must define F4ah one-step apply advance and forbid F4ad next helper",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionConsumerItem\s+Rejected\s+%GuiSfntSimpleGlyphPathSinkRejectReason\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4ah must expose typed apply advance states",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:/,
    "alloc/gui/font/sfnt/glyf F4ah apply advance must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance %fn &ByteBuf fn Option i32 fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep fn &GuiSfntSimpleGlyphPathSinkPolicy Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntParseError"),
    "alloc/gui/font/sfnt/glyf F4ah must expose byte-backed consumer apply advance helper",
);
assert(
    guiFontSfntPathTests.includes("path sink consumer apply advance keeps domain terminals as ok values") &&
        guiFontSfntPathTests.includes("apply_advance_rejects_off_curve") &&
        guiFontSfntPathTests.includes("apply_advance_ends_contour") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance"),
    "gui font sfnt path doctest must cover F4ah domain terminal Ok values",
);
const pathSinkActionConsumerApplyAdvance = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance");
for (const fragment of [
    "gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Continue continue_step:",
    "let next %GuiSfntSimpleGlyphPathSinkActionItemNext gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next &continue_step",
    "GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item:",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue next_consumer_item",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Rejected reason:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected reason",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::EndContour _end_step:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour",
]) {
    assert(pathSinkActionConsumerApplyAdvance.includes(fragment), `alloc/gui/font/sfnt/glyf F4ah apply advance helper must include ${fragment}`);
}
assert(
    (pathSinkActionConsumerApplyAdvance.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ah helper must call F4ag terminal helper exactly once",
);
assert(
    (pathSinkActionConsumerApplyAdvance.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ah helper must read stored action item next exactly once",
);
assert(
    (pathSinkActionConsumerApplyAdvance.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ah helper must call F4ac consumer item lookup exactly once",
);
assertNoMatch(
    pathSinkActionConsumerApplyAdvance,
    /\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_advance\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step\b|\bgui_sfnt_lookup_simple_glyph_path_sink_step\b|\bgui_sfnt_lookup_simple_glyph_path_contour_step\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b/,
    "alloc/gui/font/sfnt/glyf F4ah helper must not call F4ad next helper or lower lookup helpers directly",
);
assertNoMatch(
    pathSinkActionConsumerApplyAdvance,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ah helper must not allocate, loop, re-apply payloads, match payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumerApplyAdvance,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ah helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer consume once[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep[\s\S]*apply_step[\s\S]*advance/,
    "font spec must define F4ai consume once as preserving both apply step and advance",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer consume once[\s\S]*apply_step` is not redundant[\s\S]*must not call F4ag directly/,
    "font detailed design must explain why F4ai preserves apply_step and delegates terminal classification to F4ah",
);
assertMatch(
    implementationPlan,
    /Phase F4ai: sfnt simple glyph path sink action consumer item consume once[\s\S]*apply state \/ status を捨てず[\s\S]*F4ai は F4af と F4ah の薄い合成/,
    "font implementation plan must define F4ai consume once and preserve apply state/status",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:\s+apply_step\s+%GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep\s+advance\s+%GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance/,
    "alloc/gui/font/sfnt/glyf F4ai must expose consume step with apply step and advance",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:/,
    "alloc/gui/font/sfnt/glyf F4ai consume step must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step %fn GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep fn GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance"),
    "alloc/gui/font/sfnt/glyf F4ai must expose consume step constructor and accessors",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once %fn &ByteBuf fn Option i32 fn GuiSfntSimpleGlyphPathSinkActionApplyState fn &GuiSfntSimpleGlyphPathSinkActionConsumerItem fn &GuiSfntSimpleGlyphPathSinkPolicy Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError"),
    "alloc/gui/font/sfnt/glyf F4ai must expose byte-backed consume-once helper",
);
assert(
    guiFontSfntPathTests.includes("path sink consumer consume once preserves apply result and advance") &&
        guiFontSfntPathTests.includes("consume_once_reject_ok") &&
        guiFontSfntPathTests.includes("consume_once_end_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once"),
    "gui font sfnt path doctest must cover F4ai consume step preserving apply result and advance",
);
const pathSinkActionConsumerItemConsumeOnce = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once");
for (const fragment of [
    "let apply_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item",
    "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index &apply_step policy",
    "Result::Err error:",
    "Result::Err error",
    "Result::Ok advance:",
    "let consume_step %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step apply_step advance",
    "Result::Ok consume_step",
]) {
    assert(pathSinkActionConsumerItemConsumeOnce.includes(fragment), `alloc/gui/font/sfnt/glyf F4ai consume-once helper must include ${fragment}`);
}
assert(
    (pathSinkActionConsumerItemConsumeOnce.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_apply\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ai helper must call F4af consumer item apply exactly once",
);
assert(
    (pathSinkActionConsumerItemConsumeOnce.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ai helper must call F4ah apply advance exactly once",
);
assert(
    (pathSinkActionConsumerItemConsumeOnce.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_consume_step\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ai helper must construct consume step exactly once",
);
assertNoMatch(
    pathSinkActionConsumerItemConsumeOnce,
    /\bgui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_advance\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step\b|\bgui_sfnt_lookup_simple_glyph_path_sink_step\b|\bgui_sfnt_lookup_simple_glyph_path_contour_step\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b/,
    "alloc/gui/font/sfnt/glyf F4ai helper must not call F4ag directly, F4ad next helper, or lower lookup helpers directly",
);
assertNoMatch(
    pathSinkActionConsumerItemConsumeOnce,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ai helper must not allocate, loop, re-apply payloads, match payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumerItemConsumeOnce,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ai helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action start consumer item[\s\S]*gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item[\s\S]*F4ac は consumer item を作る契約上/,
    "font spec must define F4aj start consumer item and separate F4ac checked next from F4aj advance",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action start consumer item[\s\S]*does not create a new value type[\s\S]*No advance[\s\S]*no F4ad consumer-item-next call/,
    "font detailed design must define F4aj as a thin start-to-consumer boundary",
);
assertMatch(
    implementationPlan,
    /Phase F4aj: sfnt simple glyph path sink action start consumer item[\s\S]*F4aa start item と F4ac consumer item を薄く合成[\s\S]*consumer item next \/ consume \/ apply \/ advance/,
    "font implementation plan must define F4aj start consumer item without loop or consume responsibility",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item %fn &ByteBuf fn Option i32 fn GuiGlyphId fn i32 fn &GuiSfntSimpleGlyphPathSinkPolicy Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError"),
    "alloc/gui/font/sfnt/glyf F4aj must expose byte-backed start consumer item helper",
);
assert(
    guiFontSfntPathTests.includes("start_consumer_item_direct_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item") &&
        guiFontSfntPathTests.includes("path contour step public lookup follows cursor next contract"),
    "gui font sfnt path doctest must cover F4aj direct start consumer item helper",
);
const pathSinkActionStartConsumerItem = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item");
for (const fragment of [
    "match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy:",
    "Result::Err error:",
    "Result::Err error",
    "Result::Ok item:",
    "match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &item policy:",
    "Result::Ok consumer_item:",
    "Result::Ok consumer_item",
]) {
    assert(pathSinkActionStartConsumerItem.includes(fragment), `alloc/gui/font/sfnt/glyf F4aj start consumer helper must include ${fragment}`);
}
assert(
    (pathSinkActionStartConsumerItem.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4aj helper must call F4aa start item exactly once",
);
assert(
    (pathSinkActionStartConsumerItem.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4aj helper must call F4ac consumer item exactly once",
);
assertNoMatch(
    pathSinkActionStartConsumerItem,
    /\bGuiSfntSimpleGlyphPathSinkActionConsumerItemNext\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once\b|\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_apply\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_advance\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step\b|\bgui_sfnt_lookup_simple_glyph_path_sink_step\b|\bgui_sfnt_lookup_simple_glyph_path_contour_step\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b/,
    "alloc/gui/font/sfnt/glyf F4aj helper must not call consumer next, consume/apply/advance, or lower lookup helpers directly",
);
assertNoMatch(
    pathSinkActionStartConsumerItem,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4aj helper must not allocate, loop, inspect payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionStartConsumerItem,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4aj helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action start consume once[\s\S]*gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once[\s\S]*apply state \/ status と post-consume advance を両方保持/,
    "font spec must define F4ak start consume-once as preserving apply state/status and advance",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action start consume once[\s\S]*not only `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`[\s\S]*F4ak is not a contour loop/,
    "font detailed design must explain why F4ak preserves F4ai consume step and stays non-loop",
);
assertMatch(
    implementationPlan,
    /Phase F4ak: sfnt simple glyph path sink action start consume once[\s\S]*F4aj start consumer item と F4ai consume once を薄く合成[\s\S]*apply state \/ status と post-consume advance/,
    "font implementation plan must define F4ak as F4aj plus F4ai and preserve consume step",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once %fn &ByteBuf fn Option i32 fn GuiSfntSimpleGlyphPathSinkActionApplyState fn GuiGlyphId fn i32 fn &GuiSfntSimpleGlyphPathSinkPolicy Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError"),
    "alloc/gui/font/sfnt/glyf F4ak must expose byte-backed start consume-once helper",
);
assert(
    guiFontSfntPathTests.includes("start_consume_once_ok") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once") &&
        guiFontSfntPathTests.includes("path contour step public lookup follows cursor next contract"),
    "gui font sfnt path doctest must cover F4ak direct start consume-once helper",
);
const pathSinkActionStartConsumeOnce = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once");
for (const fragment of [
    "match gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy:",
    "Result::Err error:",
    "Result::Err error",
    "Result::Ok consumer_item:",
    "match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &consumer_item policy:",
    "Result::Ok consume_step:",
    "Result::Ok consume_step",
]) {
    assert(pathSinkActionStartConsumeOnce.includes(fragment), `alloc/gui/font/sfnt/glyf F4ak start consume-once helper must include ${fragment}`);
}
assert(
    (pathSinkActionStartConsumeOnce.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ak helper must call F4aj start consumer item exactly once",
);
assert(
    (pathSinkActionStartConsumeOnce.match(/\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4ak helper must call F4ai consume-once exactly once",
);
assertNoMatch(
    pathSinkActionStartConsumeOnce,
    /\bGuiSfntSimpleGlyphPathSinkActionConsumerItemNext\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_item\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next\b|\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_apply\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_item\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step_advance\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_step\b|\bgui_sfnt_lookup_simple_glyph_path_sink_step\b|\bgui_sfnt_lookup_simple_glyph_path_contour_step\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b/,
    "alloc/gui/font/sfnt/glyf F4ak helper must not call direct F4aa/F4ac/F4ad/F4af/F4ah or lower lookup helpers",
);
assertNoMatch(
    pathSinkActionStartConsumeOnce,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ak helper must not allocate, loop, inspect payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionStartConsumeOnce,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ak helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer consume step apply summary[\s\S]*gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state[\s\S]*gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status[\s\S]*`advance` を読まない/,
    "font spec must define F4al consume step apply summary helpers and forbid advance reads",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer consume step apply summary[\s\S]*future loop needs the updated apply state and the consumed action status[\s\S]*does not read `advance`/,
    "font detailed design must define F4al as pure state/status projection over consume step",
);
assertMatch(
    implementationPlan,
    /Phase F4al: sfnt simple glyph path sink action consumer consume step apply summary[\s\S]*nested storage layout へ直接依存しない[\s\S]*advance 禁止/,
    "font implementation plan must define F4al apply summary helper and forbid advance reads",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphPathSinkActionApplyState") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphPathSinkActionApplyStatus"),
    "alloc/gui/font/sfnt/glyf F4al must expose consume step apply state/status helpers",
);
assert(
    ((guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status")) ||
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step")) &&
        guiFontSfntPathTests.includes("path sink consumer consume once preserves apply result and advance") &&
        guiFontSfntPathTests.includes("path contour step public lookup follows cursor next contract"),
    "gui font sfnt path doctests must cover F4al consume step apply state/status helpers directly or through F4am summary",
);
const pathSinkActionConsumeStepApplyState = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state");
const pathSinkActionConsumeStepApplyStatus = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status");
for (const [sliceName, source, terminalFragment, terminalAccessor, oppositeAccessor] of [
    [
        "apply state",
        pathSinkActionConsumeStepApplyState,
        "gui_sfnt_simple_glyph_path_sink_action_apply_step_state &inner_apply_step",
        "gui_sfnt_simple_glyph_path_sink_action_apply_step_state",
        "gui_sfnt_simple_glyph_path_sink_action_apply_step_status",
    ],
    [
        "apply status",
        pathSinkActionConsumeStepApplyStatus,
        "gui_sfnt_simple_glyph_path_sink_action_apply_step_status &inner_apply_step",
        "gui_sfnt_simple_glyph_path_sink_action_apply_step_status",
        "gui_sfnt_simple_glyph_path_sink_action_apply_step_state",
    ],
]) {
    for (const fragment of [
        "let consumer_apply_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step",
        "let inner_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step &consumer_apply_step",
        terminalFragment,
    ]) {
        assert(source.includes(fragment), `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must include ${fragment}`);
    }
    assert(
        (source.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step\b/g) || []).length === 1,
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must read consume apply step exactly once`,
    );
    assert(
        (source.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step\b/g) || []).length === 1,
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must read inner apply step exactly once`,
    );
    assert(
        (source.match(new RegExp(`\\b${terminalAccessor}\\b`, "g")) || []).length === 1,
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must read terminal ${terminalAccessor} exactly once`,
    );
    assertNoMatch(
        source,
        new RegExp(`\\b${oppositeAccessor}\\b`),
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must not read opposite terminal accessor ${oppositeAccessor}`,
    );
    assertNoMatch(
        source,
        /\badvance\b|\bgui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance\b|\bResult\b|\bOption\b|\bgui_sfnt_lookup_|\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_apply\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b/,
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must not read advance, return Result/Option, or call lookup/consume/start/lower helpers`,
    );
    assertNoMatch(
        source,
        /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper must not allocate, loop, inspect payload variants, render, rasterize, or call host/platform APIs`,
    );
    assertNoMatch(
        source,
        /[()]/,
        `alloc/gui/font/sfnt/glyf F4al ${sliceName} helper body must preserve NEPL prefix style without parentheses`,
    );
}
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer consume summary[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary[\s\S]*state GuiSfntSimpleGlyphPathSinkActionApplyState[\s\S]*status GuiSfntSimpleGlyphPathSinkActionApplyStatus[\s\S]*advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance[\s\S]*F4am は `advance` enum を新しく解釈しない/,
    "font spec must define F4am consume summary value without reinterpreting advance",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer consume summary[\s\S]*future loop needs three values after each consume step[\s\S]*F4al still forbids `advance`, while F4am requires exactly one call to the existing advance accessor/,
    "font detailed design must distinguish F4al apply summary from F4am full consume summary",
);
assertMatch(
    implementationPlan,
    /Phase F4am: sfnt simple glyph path sink action consumer consume summary value[\s\S]*state \/ status \/ advance の flat summary value[\s\S]*F4al の apply-state\/status helper は引き続き `advance` を読まない/,
    "font implementation plan must define F4am consume summary value and keep F4al advance-free",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+struct\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:\s+state\s+%GuiSfntSimpleGlyphPathSinkActionApplyState\s+status\s+%GuiSfntSimpleGlyphPathSinkActionApplyStatus\s+advance\s+%GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance/,
    "alloc/gui/font/sfnt/glyf F4am must expose consume summary fields",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:/,
    "alloc/gui/font/sfnt/glyf F4am consume summary must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary %fn GuiSfntSimpleGlyphPathSinkActionApplyState fn GuiSfntSimpleGlyphPathSinkActionApplyStatus fn GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntSimpleGlyphPathSinkActionApplyState") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntSimpleGlyphPathSinkActionApplyStatus") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance") &&
        allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary"),
    "alloc/gui/font/sfnt/glyf F4am must expose consume summary constructor, accessors, and from-step helper",
);
assert(
    guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status") &&
        (guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance") ||
            guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal")),
    "gui font sfnt path doctests must use F4am consume summary value directly or through F4an terminal projection",
);
const pathSinkActionConsumeSummaryConstructor = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary");
assert(
    pathSinkActionConsumeSummaryConstructor.includes("GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary state status advance"),
    "alloc/gui/font/sfnt/glyf F4am constructor must preserve state/status/advance order",
);
assertNoMatch(
    pathSinkActionConsumeSummaryConstructor,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4am constructor body must preserve NEPL prefix style without parentheses",
);
for (const [sliceName, helperName, fragment] of [
    ["state", "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state", '*field::get_ref summary "state"'],
    ["status", "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status", '*field::get_ref summary "status"'],
    ["advance", "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance", '*field::get_ref summary "advance"'],
]) {
    const accessorSource = functionSlice(allocFontSfntGlyfImpl, helperName);
    assert(accessorSource.includes(fragment), `alloc/gui/font/sfnt/glyf F4am ${sliceName} accessor must include ${fragment}`);
    assertNoMatch(
        accessorSource,
        /[()]/,
        `alloc/gui/font/sfnt/glyf F4am ${sliceName} accessor body must preserve NEPL prefix style without parentheses`,
    );
}
const pathSinkActionConsumeSummaryFromStep = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step");
for (const fragment of [
    "let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step",
    "let status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step",
    "let advance %GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance step",
    "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary state status advance",
]) {
    assert(pathSinkActionConsumeSummaryFromStep.includes(fragment), `alloc/gui/font/sfnt/glyf F4am summary-from-step helper must include ${fragment}`);
}
for (const [callName, message] of [
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state", "read F4al apply state exactly once"],
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status", "read F4al apply status exactly once"],
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance", "read consume advance exactly once"],
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary", "construct consume summary exactly once"],
]) {
    assert(
        (pathSinkActionConsumeSummaryFromStep.match(new RegExp(`\\b${callName}\\b`, "g")) || []).length === 1,
        `alloc/gui/font/sfnt/glyf F4am summary-from-step helper must ${message}`,
    );
}
assertNoMatch(
    pathSinkActionConsumeSummaryFromStep,
    /\bResult\b|\bOption\b|\bgui_sfnt_lookup_|\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_apply\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b/,
    "alloc/gui/font/sfnt/glyf F4am summary-from-step helper must not return Result/Option or call lookup/consume/start/lower helpers",
);
assertNoMatch(
    pathSinkActionConsumeSummaryFromStep,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4am summary-from-step helper must not allocate, loop, inspect payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumeSummaryFromStep,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4am summary-from-step helper body must preserve NEPL prefix style without parentheses",
);
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer consume summary terminal[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal[\s\S]*Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem[\s\S]*Rejected GuiSfntSimpleGlyphPathSinkRejectReason[\s\S]*F4an は `Result` \/ `Option`、byte-backed lookup/,
    "font spec must define F4an consume summary traversal control projection and constraints",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer consume summary terminal[\s\S]*F4an adds the next pure projection above F4am[\s\S]*Although the public enum name ends with `Terminal`, it is the future loop's traversal control state and includes `Continue`/,
    "font detailed design must distinguish F4an traversal control from terminal-only state",
);
assertMatch(
    implementationPlan,
    /Phase F4an: sfnt simple glyph path sink action consumer consume summary terminal[\s\S]*stored advance の 3 分岐を 1 回だけ読み[\s\S]*summary terminal type は次の 3 variants を持つ/,
    "font implementation plan must define F4an consume summary terminal projection",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionConsumerItem\s+Rejected\s+%GuiSfntSimpleGlyphPathSinkRejectReason\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4an must expose consume summary terminal enum variants",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:/,
    "alloc/gui/font/sfnt/glyf F4an consume summary terminal must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal %fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal") &&
        guiFontSfntPathTests.includes("gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal"),
    "alloc/gui/font/sfnt/glyf and doctests must expose and use F4an consume summary terminal helper",
);
const pathSinkActionConsumeSummaryTerminal = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal");
for (const fragment of [
    "let advance %GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance summary",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue item:",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue item",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected reason:",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected reason",
    "GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour:",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour",
]) {
    assert(pathSinkActionConsumeSummaryTerminal.includes(fragment), `alloc/gui/font/sfnt/glyf F4an terminal helper must include ${fragment}`);
}
assert(
    (pathSinkActionConsumeSummaryTerminal.match(/\bgui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance\b/g) || []).length === 1,
    "alloc/gui/font/sfnt/glyf F4an terminal helper must read summary advance exactly once",
);
assertNoMatch(
    pathSinkActionConsumeSummaryTerminal,
    /\bResult\b|\bOption\b|\bgui_sfnt_lookup_|\bgui_sfnt_simple_glyph_path_sink_action_consumer_item_next\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once\b|\bgui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b|\b_with_tables\b/,
    "alloc/gui/font/sfnt/glyf F4an terminal helper must not return Result/Option or call lookup/consume/start/lower/table helpers",
);
assertNoMatch(
    pathSinkActionConsumeSummaryTerminal,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4an terminal helper must not allocate, loop, inspect payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumeSummaryTerminal,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4an terminal helper body must preserve NEPL prefix style without parentheses",
);
for (const fragment of [
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour",
]) {
    assert(guiFontSfntPathTests.includes(fragment), `gui font sfnt path doctests must cover F4an ${fragment}`);
}
assertMatch(
    spec,
    /SFNT simple glyph path sink action consumer consume summary advance once[\s\S]*`Rejected` と `EndContour` は parse error ではなく[\s\S]*GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance[\s\S]*Continue GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary/,
    "font spec must define F4ao consume summary advance-once and domain terminal contract",
);
assertMatch(
    detailedDesign,
    /SFNT simple glyph path sink action consumer consume summary advance once[\s\S]*first byte-backed boundary above the F4am\/F4an summary projection[\s\S]*`Rejected` and `EndContour` are domain terminals/,
    "font detailed design must define F4ao as one-step summary advance boundary",
);
assertMatch(
    implementationPlan,
    /Phase F4ao: sfnt simple glyph path sink action consumer consume summary advance once[\s\S]*summary advance type は次の 3 variants を持つ[\s\S]*parse error と domain terminal を混同しない/,
    "font implementation plan must define F4ao consume summary advance-once",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /pub\s+enum\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:\s+Continue\s+%GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary\s+Rejected\s+%GuiSfntSimpleGlyphPathSinkRejectReason\s+EndContour/,
    "alloc/gui/font/sfnt/glyf F4ao must expose consume summary advance enum variants",
);
assertMatch(
    allocFontSfntGlyfImpl,
    /impl\s+Clone\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:[\s\S]*impl\s+Copy\s+for\s+GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:/,
    "alloc/gui/font/sfnt/glyf F4ao consume summary advance must implement Clone and Copy",
);
assert(
    allocFontSfntGlyfImpl.includes("pub fn gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once %fn &ByteBuf fn Option i32 fn &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary fn &GuiSfntSimpleGlyphPathSinkPolicy Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance GuiSfntParseError") &&
        guiFontSfntPathTests.includes("GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance") &&
        guiFontSfntPathTests.includes("gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once"),
    "alloc/gui/font/sfnt/glyf and doctests must expose and use F4ao consume summary advance-once helper",
);
const pathSinkActionConsumeSummaryAdvanceOnce = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once");
for (const fragment of [
    "let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary",
    "let terminal %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue item:",
    "match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &item policy:",
    "let next_summary %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Continue next_summary",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected reason:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Rejected reason",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour:",
    "Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::EndContour",
]) {
    assert(pathSinkActionConsumeSummaryAdvanceOnce.includes(fragment), `alloc/gui/font/sfnt/glyf F4ao advance-once helper must include ${fragment}`);
}
for (const [callName, message] of [
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state", "read summary state exactly once"],
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal", "read summary terminal exactly once"],
    ["gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once", "consume next item exactly once"],
    ["gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step", "build next summary exactly once"],
]) {
    assert(
        (pathSinkActionConsumeSummaryAdvanceOnce.match(new RegExp(`\\b${callName}\\b`, "g")) || []).length === 1,
        `alloc/gui/font/sfnt/glyf F4ao advance-once helper must ${message}`,
    );
}
assertNoMatch(
    pathSinkActionConsumeSummaryAdvanceOnce,
    /\bgui_sfnt_lookup_simple_glyph_path_sink_action_start|\bgui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next\b|\bgui_sfnt_parse_metadata\b|\bgui_sfnt_glyf_|\bgui_sfnt_classify_simple_glyph_curve_segment\b|\b_with_tables\b/,
    "alloc/gui/font/sfnt/glyf F4ao advance-once helper must not call start, consumer-next, lower, metadata, or table helpers",
);
assertNoMatch(
    pathSinkActionConsumeSummaryAdvanceOnce,
    /\b(?:Vec|push|action_index|command_index|loop_index|current_point|cursor|next_cursor|GuiSfntSimpleGlyphPathSinkAction::|GuiSfntSimpleGlyphPathSinkPrimaryAction::|GuiSfntSimpleGlyphPathSinkTailAction::|RenderCommand|render_command_|RenderTarget|DrawTarget|render2d|backend|raster|Raster|platform|Canvas|DOM|FontFace|CoreText|DirectWrite|fontconfig|HostTextMeasurer|MockTextMeasurer|host_text_measurer)\b/,
    "alloc/gui/font/sfnt/glyf F4ao advance-once helper must not allocate, loop, inspect payload variants, render, rasterize, or call host/platform APIs",
);
assertNoMatch(
    pathSinkActionConsumeSummaryAdvanceOnce,
    /[()]/,
    "alloc/gui/font/sfnt/glyf F4ao advance-once helper body must preserve NEPL prefix style without parentheses",
);
for (const fragment of [
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Continue",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Rejected",
    "GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::EndContour",
]) {
    assert(guiFontSfntPathTests.includes(fragment), `gui font sfnt path doctests must cover F4ao ${fragment}`);
}
const contourSpanWithTables = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_simple_contour_span_with_tables");
assertNoMatch(
    contourSpanWithTables,
    /\bgui_sfnt_glyf_simple_point_stream_with_tables\b|\bgui_sfnt_lookup_simple_glyph_point_stream\b|\bgui_sfnt_lookup_simple_glyph_point\b/,
    "alloc/gui/font/sfnt/glyf F4i table helper must not depend on F4g/F4h point decoding",
);
const contourPointWithTables = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_simple_contour_point_with_tables");
assertMatch(
    contourPointWithTables,
    /\bgui_sfnt_glyf_simple_contour_span_with_tables\b[\s\S]*\bgui_sfnt_glyf_simple_point_with_tables\b/,
    "alloc/gui/font/sfnt/glyf F4j table helper must compose F4i and F4h internal table helpers",
);
assertNoMatch(
    contourPointWithTables,
    /\bgui_sfnt_lookup_simple_glyph_contour_span\b|\bgui_sfnt_lookup_simple_glyph_point\b|\bgui_sfnt_lookup_simple_glyph_point_stream\b/,
    "alloc/gui/font/sfnt/glyf F4j table helper must not call public wrappers after metadata unwrap",
);
assertMatch(
    contourEdgeWithTables,
    /\bgui_sfnt_glyf_simple_contour_span_with_tables\b[\s\S]*\bgui_sfnt_glyf_simple_contour_point_with_tables\b[\s\S]*\bgui_sfnt_glyf_simple_contour_point_with_tables\b/,
    "alloc/gui/font/sfnt/glyf F4k table helper must compose F4i and F4j internal table helpers",
);
assertNoMatch(
    contourEdgeWithTables,
    /\bgui_sfnt_lookup_simple_glyph_contour_span\b|\bgui_sfnt_lookup_simple_glyph_contour_point\b|\bgui_sfnt_lookup_simple_glyph_point\b|\bgui_sfnt_lookup_simple_glyph_point_stream\b/,
    "alloc/gui/font/sfnt/glyf F4k table helper must not call public wrappers after metadata unwrap",
);
const classifyCurveSegment = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_classify_simple_glyph_curve_segment");
for (const fragment of [
    "eq span_point_count 1",
    "GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour",
    "not start_on_curve",
    "GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart",
    "gui_sfnt_simple_glyph_point_on_curve &end_point",
    "GuiSfntSimpleGlyphCurveSegment::Line",
    "Option::None:",
    "GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead",
    "end_is_implied %bool not gui_sfnt_simple_glyph_point_on_curve &lookahead_point",
    "add end_x lookahead_x",
    "add end_y lookahead_y",
    "GuiSfntSimpleGlyphCurveSegment::Quadratic",
]) {
    assert(classifyCurveSegment.includes(fragment), `alloc/gui/font/sfnt/glyf curve classifier must include ${fragment}`);
}
assertNoMatch(
    classifyCurveSegment,
    /\bdiv_[su]\b/,
    "alloc/gui/font/sfnt/glyf curve classifier must not divide implied midpoint coordinates",
);
const curveSegmentWithTables = functionSlice(allocFontSfntGlyfImpl, "gui_sfnt_glyf_simple_curve_segment_with_tables");
for (const fragment of [
    "gui_sfnt_glyf_simple_contour_edge_with_tables",
    "needs_lookahead %bool",
    "start_on_curve",
    "not end_on_curve",
    "lookahead_contour_point_index",
    "eq add next_contour_point_index 1 span_point_count",
    "gui_sfnt_glyf_simple_contour_point_with_tables",
    "gui_sfnt_classify_simple_glyph_curve_segment edge Option::Some lookahead",
    "gui_sfnt_classify_simple_glyph_curve_segment edge Option::None",
]) {
    assert(curveSegmentWithTables.includes(fragment), `alloc/gui/font/sfnt/glyf curve segment helper must include ${fragment}`);
}
assertNoMatch(
    curveSegmentWithTables,
    /\bgui_sfnt_lookup_simple_glyph_contour_edge\b|\bgui_sfnt_lookup_simple_glyph_contour_point\b|\bgui_sfnt_lookup_simple_glyph_point\b|\bgui_sfnt_lookup_simple_glyph_point_stream\b/,
    "alloc/gui/font/sfnt/glyf F4l table helper must not call public wrappers after metadata unwrap",
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
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_contour_point\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf contour point lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_contour_edge\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf contour edge lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_curve_segment\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf curve segment lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_move_to_command\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf move command lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_draw_command\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf draw command lookup",
);
assertNoMatch(
    allocFontSfntMetadataImpl,
    /\bgui_sfnt_lookup_simple_glyph_path_command_pair\b/,
    "gui_sfnt_parse_metadata must remain independent from glyf path command pair lookup",
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
    "contour point first local index",
    "contour point first absolute index",
    "contour point first x",
    "contour point first y",
    "contour point first not contour end",
    "contour point second span index",
    "contour point second local index",
    "contour point second absolute index",
    "contour point second contour end",
    "contour point signed absolute index",
    "contour point signed x",
    "contour point signed y",
    "contour point signed on curve",
    "contour edge first edge index",
    "contour edge first next local index",
    "contour edge first start absolute index",
    "contour edge first end absolute index",
    "contour edge first end contour end",
    "contour edge wrap next local index",
    "contour edge wrap start absolute index",
    "contour edge wrap end absolute index",
    "contour edge self wrap next local index",
    "contour edge self wrap same absolute index",
    "contour edge signed start absolute index",
    "contour edge signed start x",
    "contour edge signed start y",
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
    "contour point negative local missing",
    "contour point local count missing",
    "contour point x coordinate overrun",
    "contour edge negative local missing",
    "contour edge local count missing",
    "contour edge x coordinate overrun",
]) {
    assertMatch(
        guiFontSfntTests,
        new RegExp(glyfCase.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
        `gui font sfnt doctest must cover ${glyfCase}`,
    );
}
assertMatch(
    guiFontSfntCurveLookupTests,
    /GUI font SFNT glyf curve segment public lookup doctests[\s\S]*neplg2:test\[skip, stdio, normalize_newlines\][\s\S]*gui_sfnt_lookup_simple_glyph_curve_segment[\s\S]*curve lookup implied odd midpoint through public lookup/,
    "gui font sfnt doctest must preserve skipped byte-level public curve lookup smoke",
);
assertMatch(
    guiFontSfntCurveLookupTests,
    /curve_lookup_fixture_byte[\s\S]*byte_builder_push_u8[\s\S]*curve_lookup_fixture_bytes[\s\S]*byte_builder_finish/,
    "gui font sfnt curve lookup smoke must build binary bytes through ByteBuilder, not text conversion",
);
assertNoMatch(
    guiFontSfntCurveLookupTests,
    /\bio_bytebuf_from_str_result\b/,
    "gui font sfnt curve lookup smoke must not build binary SFNT data from UTF-8 text",
);

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

