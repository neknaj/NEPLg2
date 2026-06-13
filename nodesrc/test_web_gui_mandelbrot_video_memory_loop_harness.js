#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createGuiVideoMemoryFakeHost } = require("./gui_video_memory_fake_host");
const { createExpectedMandelbrotRgbaRow } = require("./mandelbrot_expected_rows");
const { runSingle } = require("./run_test");

const FIRST_WIDTH = 32;
const FIRST_HEIGHT = 18;
const FIRST_LIMIT = 24;
const SECOND_WIDTH = 40;
const SECOND_HEIGHT = 18;
const SECOND_LIMIT = 64;
const THIRD_WIDTH = 48;
const THIRD_HEIGHT = 24;
const THIRD_LIMIT = 64;

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function sourceBetween(source, startNeedle, endNeedle) {
    const start = source.indexOf(startNeedle);
    const end = source.indexOf(endNeedle, start + startNeedle.length);
    assert.notEqual(start, -1, `missing start needle: ${startNeedle}`);
    assert.notEqual(end, -1, `missing end needle: ${endNeedle}`);
    return source.slice(start, end);
}

async function runWebGuiMandelbrotVideoMemoryLoopHarnessRegression() {
    const source = readRepoFile("examples", "gui_mandelbrot.nepl");
    assert.match(source, /--video-memory-loop/);
    assert.match(source, /--video-memory-loop-test/);
    assert.match(source, /fn mandelbrot_video_memory_event_loop_with_limit[\s\S]*while running/);
    assert.match(source, /fn mandelbrot_video_memory_event_loop_with_limit[\s\S]*WindowEventKind::Resized[\s\S]*mandelbrot_video_memory_close_and_open_next/);
    assert.match(source, /fn mandelbrot_video_memory_event_loop_with_limit[\s\S]*WindowEventKind::Focused:[\s\S]*unit[\s\S]*WindowEventKind::Unfocused:[\s\S]*unit/);
    assert.match(source, /fn mandelbrot_video_memory_event_loop_with_limit[\s\S]*WindowEventKind::CloseRequested:[\s\S]*mandelbrot_video_memory_loop_finish_ok/);

    const loopSlice = sourceBetween(source, "fn mandelbrot_video_memory_loop_wait_ms", "fn mandelbrot_present_row_pixel");
    assert.doesNotMatch(loopSlice, /mandelbrot_present_frame|gui_web_stdout_|gui_web_stdout_frame|presentCommands|command-frame/);
    assert.doesNotMatch(loopSlice, /gui_web_video_memory_write_frame_bytes/);

    const fakeHost = createGuiVideoMemoryFakeHost({
        windowId: 1,
        title: "NEPLg2 Mandelbrot Video Memory",
        surfaces: [
            {
                width: FIRST_WIDTH,
                height: FIRST_HEIGHT,
                slotCount: 2,
                windowId: 1,
                title: "NEPLg2 Mandelbrot Video Memory",
                surfaceId: 1301,
                frameId: 5501,
                expectedRgbaRow: createExpectedMandelbrotRgbaRow(FIRST_WIDTH, FIRST_HEIGHT, FIRST_LIMIT),
            },
            {
                width: SECOND_WIDTH,
                height: SECOND_HEIGHT,
                slotCount: 2,
                windowId: 1,
                title: "NEPLg2 Mandelbrot Video Memory",
                surfaceId: 1302,
                frameId: 5502,
                expectedRgbaRow: createExpectedMandelbrotRgbaRow(SECOND_WIDTH, SECOND_HEIGHT, SECOND_LIMIT),
            },
            {
                width: THIRD_WIDTH,
                height: THIRD_HEIGHT,
                slotCount: 2,
                windowId: 1,
                title: "NEPLg2 Mandelbrot Video Memory",
                surfaceId: 1303,
                frameId: 5503,
                expectedRgbaRow: createExpectedMandelbrotRgbaRow(THIRD_WIDTH, THIRD_HEIGHT, THIRD_LIMIT),
            },
        ],
        events: [
            { kind: "window", windowKind: "resized", windowId: 1, width: 40, height: 90 },
            { kind: "window", windowKind: "focused", windowId: 1, width: 40, height: 90 },
            { kind: "window", windowKind: "unfocused", windowId: 1, width: 40, height: 90 },
            { kind: "window", windowKind: "resized", windowId: 1, width: 48, height: 96 },
            { kind: "window", windowKind: "close-requested", windowId: 1, width: 48, height: 96 },
        ],
    });

    const result = await runSingle({
        id: "examples/gui_mandelbrot.nepl/video-memory-loop-test",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_mandelbrot.nepl"),
        argv: ["--video-memory-loop-test"],
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);

    return {
        ok: true,
        checks: [
            "Mandelbrot formal video memory loop keeps the window alive across multiple typed events",
            "focused and unfocused window events do not recreate the surface",
            "resize events close the current surface before opening the resized surface",
            "close-requested closes the current surface and exits without stdout or command-frame fallback",
        ],
    };
}

if (require.main === module) {
    runWebGuiMandelbrotVideoMemoryLoopHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiMandelbrotVideoMemoryLoopHarnessRegression,
};
