#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function source(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function precedingDoc(code, declarationNeedle) {
    const index = code.indexOf(declarationNeedle);
    assert.notEqual(index, -1, `missing declaration: ${declarationNeedle}`);
    const before = code.slice(0, index).split("\n");
    const doc = [];
    let cursor = before.length - 1;
    while (cursor >= 0 && before[cursor].trim() === "") {
        cursor -= 1;
    }
    while (cursor >= 0 && before[cursor].trimStart().startsWith("//:")) {
        doc.push(before[cursor]);
        cursor -= 1;
    }
    return doc.reverse().join("\n");
}

function assertIncludes(code, needle, message) {
    assert.ok(code.includes(needle), message);
}

function assertReportDoc(code, declarationNeedle, reportName) {
    const doc = precedingDoc(code, declarationNeedle);
    assertIncludes(doc, "neplg2:test[stdio, normalize_newlines]", `${reportName} must use runnable report doctest metadata`);
    assertIncludes(doc, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
    assertIncludes(doc, "### [契約/けいやく]", `${reportName} must document stable contract`);
    assertIncludes(doc, "### [現在/げんざい]の[実装/じっそう]", `${reportName} must separate current implementation notes`);
    assertIncludes(doc, "### [計算量/けいさんりょう]", `${reportName} must document complexity`);
    return doc;
}

const renderCommand = source("stdlib/core/gui/render_command.nepl");

assert.doesNotMatch(
    renderCommand,
    /^\s*#import\s+"(?:alloc|std|platforms)\//m,
    "core/gui render_command must not import alloc/std/platform modules",
);

for (const [declaration, reportName] of [
    ["pub struct FillRectCommand:", "core_gui_fill_rect_command_doc"],
    ["pub struct StrokeRectCommand:", "core_gui_stroke_rect_command_doc"],
    ["pub struct LineCommand:", "core_gui_line_command_doc"],
    ["pub struct TextRunCommand:", "core_gui_text_run_command_doc"],
    ["pub struct ImageRectCommand:", "core_gui_image_rect_command_doc"],
    ["pub struct AlphaMaskRectCommand:", "core_gui_alpha_mask_rect_command_doc"],
    ["pub struct TextCellRunCommand:", "core_gui_text_cell_run_command_doc"],
    ["pub enum RenderCommand:", "core_gui_render_command_enum_doc"],
]) {
    assertReportDoc(renderCommand, declaration, reportName);
}

for (const snippet of [
    "GUI/TUI [共通/きょうつう] render command",
    "allocator なし",
    "TUI text-grid backend",
    "具体的な output encoding は platform [層/そう]",
    "text/image/alpha mask の[実体/じったい]は `TextRunId` / `ImageId` / `AlphaMaskId`",
    "geometry の[正当性/せいとうせい]や clipping はこの型では[検査/けんさ]しません",
    "validation は renderer/layout [境界/きょうかい]の責務",
    "rasterization algorithm、anti-aliasing、subpixel policy は core の責務ではありません",
    "`TextRunId` の[解決/かいけつ]、font shaping、line breaking、IME composition",
    "missing resource は renderer 側が `Result` / `GuiError`",
    "decode、cache、texture upload、scaling quality は alloc/std/platform [層/そう]の責務",
    "この command は SourceOver [合成/ごうせい]で alpha mask を[塗/ぬ]るための payload",
    "mask alpha と `GuiPaint` alpha",
    "mask byte storage、mask dimensions、resource 解決",
    "GUI と TUI が[共有/きょうゆう]する command stream",
    "文字列そのものや escape sequence は core に[持/も]ち[込/こ]みません",
    "unsupported variant は silent no-op にせず",
    "`GuiError::Unsupported`",
    "platform handle や string sentinel は core public API に[出/だ]しません",
    "backend は `match` で[扱/あつか]える variant だけを[処理/しょり]",
]) {
    assertIncludes(renderCommand, snippet, `render_command docs must pin GUI/TUI substrate and backend boundary: ${snippet}`);
}

for (const snippet of [
    "pub enum GuiStrokeCap:",
    "Butt",
    "Square",
    "Round",
    "pub enum GuiStrokeJoin:",
    "Miter",
    "Bevel",
    "pub enum GuiStrokeDash:",
    "Solid",
    "struct GuiStrokeProof:",
    "fn gui_stroke_proof %fn i32 GuiStrokeProof",
    "pub struct GuiStroke:",
    "cap %GuiStrokeCap",
    "join %GuiStrokeJoin",
    "miter_limit %f32",
    "dash %GuiStrokeDash",
    "proof %GuiStrokeProof",
    "pub fn gui_stroke_new %fn Rgba8888 fn i32 fn GuiStrokeCap fn GuiStrokeJoin fn f32 fn GuiStrokeDash Result GuiStroke GuiError",
    "let width_positive %bool gt width 0",
    "let miter_limit_positive %bool gt miter_limit 0.0",
    "if and width_positive miter_limit_positive",
    "GuiStroke color width cap join miter_limit dash gui_stroke_proof width",
    "GuiError::InvalidCommand",
    "pub fn gui_stroke_color %fn &GuiStroke Rgba8888",
    "pub fn gui_stroke_width %fn &GuiStroke i32",
    "pub fn gui_stroke_cap %fn &GuiStroke GuiStrokeCap",
    "pub fn gui_stroke_join %fn &GuiStroke GuiStrokeJoin",
    "pub fn gui_stroke_miter_limit %fn &GuiStroke f32",
    "pub fn gui_stroke_dash %fn &GuiStroke GuiStrokeDash",
    "core_gui_render_command_gui_stroke_style_doc",
    "nan miter",
    "`GuiStrokeDash::Solid` は dash なしの[明示/めいじ] policy",
]) {
    assertIncludes(renderCommand, snippet, `render_command must expose explicit stroke style contract: ${snippet}`);
}

assert.doesNotMatch(
    renderCommand,
    /pub\s+struct\s+GuiStrokeProof:|le\s+miter_limit\s+0\.0/,
    "core/gui render_command must keep stroke proof private and must not use NaN-accepting miter <= 0 validation",
);

assert.doesNotMatch(
    renderCommand,
    /pub\s+fn\s+gui_stroke_new\s+%fn\s+Rgba8888\s+fn\s+i32\s+GuiStroke\b|gui_stroke_new\s+color\s+\d+\s*(?:\n|$)/,
    "core/gui render_command must not keep the two-argument stroke constructor or doctest call shape",
);

assert.doesNotMatch(
    renderCommand,
    /GuiStroke` は color と width だけ|GuiStroke` は width と color だけ/,
    "core/gui render_command docs must not describe GuiStroke as color/width-only",
);

assert.doesNotMatch(
    renderCommand,
    /\b(?:DOM|Canvas|ANSI|TTY|Win32|UIKit|AndroidView|HtmlCanvas|Ssd1306|Skia|terminal)\b|端末/,
    "core/gui render_command docs must not expose concrete platform implementation names",
);

console.log("stdlib GUI render command doc contract passed");
