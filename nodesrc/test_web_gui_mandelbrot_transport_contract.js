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
    const responsiveContractSlice = mandelbrotSource.slice(
        mandelbrotSource.indexOf("fn mandelbrot_responsive_model_contract_ok"),
        mandelbrotSource.indexOf("fn mandelbrot_run_hd_contract_test"),
    );

    assert.match(mandelbrotSource, /fn mandelbrot_model_hd[\s\S]*mandelbrot_model_new 1280 648 1 64 MandelbrotMode::HD/);
    assert.match(mandelbrotSource, /fn mandelbrot_model_detail[\s\S]*mandelbrot_model_new 1280 648 1 96 MandelbrotMode::Detail/);
    assert.match(mandelbrotSource, /gui_web_stdout_rgba_row_begin/);
    assert.match(mandelbrotSource, /gui_web_stdout_rgba_row_pixel/);
    assert.match(mandelbrotSource, /gui_web_stdout_rgba_row_end/);
    assert.match(mandelbrotSource, /--test-hd-contract/);
    assert.match(mandelbrotSource, /--test-responsive-contract/);
    assert.match(mandelbrotSource, /--video-memory-once/);
    assert.match(mandelbrotSource, /--video-memory-resize-once/);
    assert.match(mandelbrotSource, /--video-memory-loop/);
    assert.match(mandelbrotSource, /--video-memory-loop-test/);
    assert.match(mandelbrotSource, /--test-video-memory-contract/);
    assert.match(mandelbrotSource, /MandelbrotMode::Responsive/);
    assert.match(mandelbrotSource, /fn mandelbrot_model_for_surface[\s\S]*mandelbrot_model_new sample_width sample_height 1 64 MandelbrotMode::Responsive/);
    assert.match(mandelbrotSource, /fn mandelbrot_update_window[\s\S]*WindowEventKind::Resized[\s\S]*mandelbrot_model_for_surface width height[\s\S]*WindowEventKind::Focused:[\s\S]*WindowEventKind::Unfocused:[\s\S]*WindowEventKind::CloseRequested:/);
    assert.match(mandelbrotSource, /fn mandelbrot_update_event[\s\S]*gui_web_event_action[\s\S]*gui_web_event_window[\s\S]*mandelbrot_update_window/);
    assert.match(mandelbrotSource, /let model %MandelbrotModel mandelbrot_model_for_surface 1920 1080/);
    assert.match(mandelbrotSource, /let sample_height_ok %bool eq mandelbrot_model_sample_height &model 1008/);
    assert.match(mandelbrotSource, /let command_count_ok %bool eq mandelbrot_command_count &model 1018/);
    assert.match(mandelbrotSource, /let web_event %GuiWebEvent GuiWebEvent host_window point gui_event_window window/);
    assert.doesNotMatch(responsiveContractSlice, /mandelbrot_present_frame|gui_web_stdout_|gui_web_video_memory_|mandelbrot_video_memory_present_model/);
    assert.match(mandelbrotSource, /fn mandelbrot_video_memory_model[\s\S]*mandelbrot_model_new 32 18 1 24 MandelbrotMode::Preview/);
    assert.match(mandelbrotSource, /gui_web_video_memory_create_surface/);
    assert.match(mandelbrotSource, /gui_web_video_memory_write_rgba8888_row/);
    assert.match(mandelbrotSource, /gui_web_video_memory_publish_full/);
    assert.match(mandelbrotSource, /gui_web_video_memory_present_surface/);
    assert.match(mandelbrotSource, /fn mandelbrot_video_memory_open_rendered_surface/);
    assert.match(mandelbrotSource, /fn mandelbrot_video_memory_resize_once[\s\S]*gui_web_wait_event_result/);
    assert.match(mandelbrotSource, /fn mandelbrot_video_memory_resize_once_event[\s\S]*WindowEventKind::Resized[\s\S]*mandelbrot_video_memory_close_and_open_next/);
    assert.match(mandelbrotSource, /fn mandelbrot_video_memory_event_loop_with_limit[\s\S]*while running[\s\S]*gui_web_wait_event_result[\s\S]*WindowEventKind::Resized[\s\S]*mandelbrot_video_memory_close_and_open_next/);
    assert.match(mandelbrotSource, /fn mandelbrot_video_memory_event_loop_with_limit[\s\S]*WindowEventKind::CloseRequested[\s\S]*mandelbrot_video_memory_loop_finish_ok/);
    assert.match(mandelbrotSource, /let row_command_count_ok %bool eq mandelbrot_command_count &model 658/);
    const videoMemorySlice = mandelbrotSource.slice(
        mandelbrotSource.indexOf("fn mandelbrot_video_memory_slot_count"),
        mandelbrotSource.indexOf("fn mandelbrot_present_row_pixel"),
    );
    assert.doesNotMatch(videoMemorySlice, /gui_web_stdout_/);
    assert.doesNotMatch(videoMemorySlice, /mandelbrot_present_frame/);
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
    assert.match(specSource, /`--video-memory-once`/);
    assert.match(specSource, /まだ NEPLg2 program から `DrawCommand` stream や tile \/ bitmap \/ row \/ RLE payload を JS \/ native host へ直接 export する全体正式 ABI ではない/);
    assert.match(specSource, /DrawCommand \/ tile presentation の formal host import ABI/);
    assert.match(planSource, /row payload/);
    assert.match(planSource, /正式 host import ABI/);
    assert.match(planSource, /`--video-memory-once`/);
    assert.match(planSource, /legacy transport/);
    assert.match(planSource, /legacy stdout interactive path は resize event を application update に取り込み/);
    assert.match(planSource, /`--video-memory-resize-once` は finite formal video memory resize\/recreate checkpoint/);
    assert.match(planSource, /`--video-memory-loop` は formal video memory surface を保持して typed event を待つ loop checkpoint/);
    assert.match(planSource, /progressive rendering、FHD 60 fps 実測、formal tiled transport、real scheduler policy は後続 slice/);
    assert.match(specSource, /Mandelbrot の finite video memory resize path は old surface close と resized surface recreate を検査する/);
    assert.match(specSource, /Mandelbrot の formal video memory event loop path は open surface を維持し、typed window resize event で old surface を close して resized surface を recreate する/);
    assert.match(specSource, /formal tiled rendering、real scheduler policy/);

    return {
        ok: true,
        checks: [
            "Mandelbrot HD mode uses 1280x720 logical row payload transport",
            "Mandelbrot source emits typed rgba row payloads from NEPL instead of TS simulation",
            "Mandelbrot app model consumes window resize events as a typed update input",
            "Mandelbrot responsive contract remains pure and does not render",
            "Mandelbrot video memory path has an explicit finite resize/recreate entrypoint",
            "Mandelbrot video memory loop keeps the formal surface alive across typed events",
            "Mandelbrot has an opt-in formal video memory path that does not fallback to stdout transport",
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
