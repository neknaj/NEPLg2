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
const allocFontSfnt = [allocFontSfntFacade, allocFontSfntMetadata, allocFontSfntName, allocFontSfntCmap].join("\n");
const allocFontSfntImpl = withoutComments(allocFontSfnt);
const allocFontSfntMetadataImpl = withoutComments(allocFontSfntMetadata);
const allocFontSfntNameImpl = withoutComments(allocFontSfntName);
const allocFontSfntCmapImpl = withoutComments(allocFontSfntCmap);
const coreGuiFacade = read("stdlib/core/gui.nepl");
const coreGuiPrelude = read("stdlib/core/gui/prelude.nepl");
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiCoreTests = read("tests/stdlib/gui_core.n.md");
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const guiFontSfntTests = read("tests/stdlib/gui_font_sfnt.n.md");
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
    detailedDesign,
    /SFNT cmap table[\s\S]*GuiSfntCmapSubtableRecord[\s\S]*WindowsUnicodeBmpFormat4[\s\S]*idRangeOffset[\s\S]*MissingGlyphMapping/,
    "font detailed design must define F4c cmap format-4 lookup and bounds validation",
);
assertMatch(
    implementationPlan,
    /Phase F4c:[\s\S]*alloc\/gui\/font\/sfnt\/cmap\.nepl[\s\S]*Result GuiGlyphId GuiSfntParseError[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MalformedCmapRecord[\s\S]*MissingGlyphMapping|Phase F4c:[\s\S]*alloc\/gui\/font\/sfnt\/cmap\.nepl[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MalformedCmapRecord[\s\S]*MissingGlyphMapping[\s\S]*Result GuiGlyphId GuiSfntParseError/,
    "font implementation plan must define F4c cmap parser data types and error kinds",
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
assertMatch(
    allocFontSfntImpl,
    /pub\s+enum\s+GuiSfntContainerKind:[\s\S]*TrueTypeSfnt[\s\S]*OpenTypeSfnt[\s\S]*TrueTypeCollection[\s\S]*OpenTypeCollection/,
    "alloc/gui/font/sfnt must expose typed container kinds",
);
assertMatch(
    allocFontSfntImpl,
    /pub\s+enum\s+GuiSfntParseErrorKind:[\s\S]*UnexpectedEof[\s\S]*UnsupportedContainer[\s\S]*InvalidTableDirectory[\s\S]*InvalidTableOffset[\s\S]*MissingTable[\s\S]*InvalidFaceIndex[\s\S]*FaceIndexRequired[\s\S]*UnsupportedNameTableFormat[\s\S]*MalformedNameRecord[\s\S]*UnsupportedNameEncoding[\s\S]*UnsupportedNameCharacter[\s\S]*UnsupportedCmapEncoding[\s\S]*UnsupportedCmapTableFormat[\s\S]*MalformedCmapRecord[\s\S]*MissingGlyphMapping/,
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
    /pub\s+struct\s+GuiSfntDirectory:[\s\S]*head\s+%Option\s+GuiSfntTableRecord[\s\S]*hhea\s+%Option\s+GuiSfntTableRecord[\s\S]*maxp\s+%Option\s+GuiSfntTableRecord[\s\S]*name\s+%Option\s+GuiSfntTableRecord[\s\S]*cmap\s+%Option\s+GuiSfntTableRecord/,
    "alloc/gui/font/sfnt directory must track optional name and cmap tables without requiring decoding",
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

