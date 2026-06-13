#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function runWebGuiMandelbrotTransportContractRegression() {
    const mandelbrotSource = readRepoFile("examples", "gui_mandelbrot.nepl");
    const webStdoutSource = readRepoFile("stdlib", "platforms", "gui", "web", "stdout_protocol.nepl");
    const commandSource = readRepoFile("web", "src", "gui-preview", "commands.ts");
    const stdoutProtocolSource = readRepoFile("web", "src", "gui-preview", "stdout-protocol.ts");
    const hostBridgeSource = readRepoFile("web", "src", "gui-preview", "host-bridge.ts");
    const bitmapRasterizerSource = readRepoFile("web", "src", "gui-preview", "bitmap-rasterizer.ts");
    const specSource = readRepoFile("doc", "neplg2", "gui_standard_library_spec.md");
    const planSource = readRepoFile("doc", "neplg2", "gui_tui_implementation_plan.md");

    assert.match(mandelbrotSource, /fn mandelbrot_model_hd[\s\S]*mandelbrot_model_new 1280 648 1 64 MandelbrotMode::HD/);
    assert.match(mandelbrotSource, /fn mandelbrot_model_detail[\s\S]*mandelbrot_model_new 1280 648 1 96 MandelbrotMode::Detail/);
    assert.match(mandelbrotSource, /gui_web_stdout_rgba_row_begin/);
    assert.match(mandelbrotSource, /gui_web_stdout_rgba_row_pixel/);
    assert.match(mandelbrotSource, /gui_web_stdout_rgba_row_end/);
    assert.match(mandelbrotSource, /--test-hd-contract/);
    assert.match(mandelbrotSource, /let row_command_count_ok %bool eq mandelbrot_command_count &model 658/);
    assert.doesNotMatch(mandelbrotSource, /fn mandelbrot_present_cell\b/);
    assert.doesNotMatch(mandelbrotSource, /let cells %i32 mul sample_width sample_height/);
    assert.doesNotMatch(mandelbrotSource, /mandelbrot_model_new 160 90 8/);

    assert.match(webStdoutSource, /gui_web_stdout_rgba_row_begin/);
    assert.match(webStdoutSource, /gui_web_stdout_rgba_row_pixel/);
    assert.match(webStdoutSource, /gui_web_stdout_rgba_row_end/);
    assert.match(webStdoutSource, /Result::Err GuiError::InvalidGeometry/);
    assert.doesNotMatch(webStdoutSource, /panic|unreachable/);
    assert.doesNotMatch(webStdoutSource, /fallback/i);

    assert.match(commandSource, /kind: 'rgba-row'/);
    assert.match(commandSource, /pixels: GuiPreviewColor\[\]/);
    assert.match(stdoutProtocolSource, /GUI_STDOUT_RGBA_ROW/);
    assert.match(stdoutProtocolSource, /function parseRgbaRow/);
    assert.match(stdoutProtocolSource, /invalid-rgba-row/);
    assert.match(hostBridgeSource, /decodeGuiWebHostRgbaRow/);
    assert.match(hostBridgeSource, /pixelValues\.value\.length !== sampleWidth\.value/);
    assert.match(bitmapRasterizerSource, /rasterizeGuiPreviewRgbaRow/);
    assert.match(bitmapRasterizerSource, /guiPreviewColorEquals/);
    assert.doesNotMatch(commandSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);
    assert.doesNotMatch(stdoutProtocolSource, /createGuiPreviewScene|JSON\.parse/);
    assert.doesNotMatch(hostBridgeSource, /\bas\b\s*any\b|:\s*any\b|<any>/);

    assert.match(specSource, /rgba-row/);
    assert.match(specSource, /legacy stdout protocol/);
    assert.match(specSource, /formal host import ABI/);
    assert.match(specSource, /まだ NEPLg2 program から `DrawCommand` stream や tile \/ bitmap \/ row \/ RLE payload を JS \/ native host へ直接 export する全体正式 ABI ではない/);
    assert.match(specSource, /DrawCommand \/ tile presentation の formal host import ABI/);
    assert.match(planSource, /row payload/);
    assert.match(planSource, /正式 host import ABI/);
    assert.match(planSource, /legacy transport/);
    assert.match(planSource, /DrawCommand \/ tile formal host import ABI と native `GuiHost\.present` の HD raster contract はまだ未実装である/);

    return {
        ok: true,
        checks: [
            "Mandelbrot HD mode uses 1280x720 logical row payload transport",
            "Mandelbrot source emits typed rgba row payloads from NEPL instead of TS simulation",
            "Web stdout parser, host bridge, and bitmap rasterizer support rgba-row as a typed command",
            "docs keep stdout row payload distinct from the future formal host import ABI",
        ],
    };
}

if (require.main === module) {
    try {
        const result = runWebGuiMandelbrotTransportContractRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runWebGuiMandelbrotTransportContractRegression,
};
