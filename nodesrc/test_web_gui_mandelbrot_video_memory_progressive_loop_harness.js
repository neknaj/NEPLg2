#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createGuiVideoMemoryFakeHost } = require("./gui_video_memory_fake_host");
const { createExpectedMandelbrotRgbaRow } = require("./mandelbrot_expected_rows");
const { runSingle } = require("./run_test");

const MANDELBROT_WIDTH = 32;
const MANDELBROT_HEIGHT = 18;
const MANDELBROT_ITERATION_LIMIT = 24;
const BATCH_HEIGHT = 4;
const TIMER_ID = 1;

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

function createExpectedProgressiveFrames() {
    const frames = [];
    for (let startY = 0; startY < MANDELBROT_HEIGHT; startY += BATCH_HEIGHT) {
        const endY = Math.min(startY + BATCH_HEIGHT, MANDELBROT_HEIGHT);
        frames.push({
            frameId: 7501 + frames.length,
            dirty: {
                kind: "rect",
                x: 0,
                y: startY,
                width: MANDELBROT_WIDTH,
                height: endY - startY,
            },
            rows: {
                start: startY,
                end: endY,
            },
        });
    }
    return frames;
}

async function runWebGuiMandelbrotVideoMemoryProgressiveLoopHarnessRegression() {
    const source = readRepoFile("examples", "gui_mandelbrot.nepl");
    assert.match(source, /--video-memory-progressive-loop-test/);
    assert.match(source, /fn mandelbrot_video_memory_progressive_event_is_tick[\s\S]*gui_web_event_timer[\s\S]*timer_event_timer_id/);
    assert.match(source, /fn mandelbrot_video_memory_progressive_loop_with_limit[\s\S]*gui_web_wait_event_result[\s\S]*mandelbrot_video_memory_render_batch_present/);
    assert.match(source, /fn mandelbrot_video_memory_progressive_loop_with_limit[\s\S]*mandelbrot_video_memory_progressive_event_requests_close/);

    const progressiveLoopSlice = sourceBetween(
        source,
        "fn mandelbrot_video_memory_progressive_event_is_tick",
        "fn mandelbrot_present_row_pixel",
    );
    assert.match(progressiveLoopSlice, /gui_web_event_timer/);
    assert.match(progressiveLoopSlice, /timer_event_timer_id/);
    assert.match(progressiveLoopSlice, /WindowEventKind::CloseRequested/);
    assert.doesNotMatch(progressiveLoopSlice, /gui_web_stdout_|mandelbrot_present_frame|presentCommands|command-frame/);
    assert.doesNotMatch(progressiveLoopSlice, /gui_web_video_memory_write_frame_bytes|gui_web_video_memory_fill_rect_rgba8888/);

    const fakeHost = createGuiVideoMemoryFakeHost({
        windowId: 1,
        title: "NEPLg2 Mandelbrot Video Memory",
        expectedEventCalls: Array.from({ length: 9 }, () => ({ name: "wait", args: [60000] })),
        surfaces: [
            {
                width: MANDELBROT_WIDTH,
                height: MANDELBROT_HEIGHT,
                slotCount: 2,
                windowId: 1,
                title: "NEPLg2 Mandelbrot Video Memory",
                surfaceId: 1501,
                expectedRgbaRow: createExpectedMandelbrotRgbaRow(
                    MANDELBROT_WIDTH,
                    MANDELBROT_HEIGHT,
                    MANDELBROT_ITERATION_LIMIT,
                ),
                frames: createExpectedProgressiveFrames(),
            },
        ],
        events: [
            { kind: "timer", timerId: 99, tick: 1 },
            { kind: "none" },
            { kind: "window", windowKind: "focused", windowId: 1, width: MANDELBROT_WIDTH, height: MANDELBROT_HEIGHT },
            { kind: "window", windowKind: "resized", windowId: 1, width: MANDELBROT_WIDTH, height: MANDELBROT_HEIGHT },
            { kind: "timer", timerId: TIMER_ID, tick: 1 },
            { kind: "timer", timerId: TIMER_ID, tick: 2 },
            { kind: "timer", timerId: TIMER_ID, tick: 3 },
            { kind: "timer", timerId: TIMER_ID, tick: 4 },
            { kind: "timer", timerId: TIMER_ID, tick: 5 },
        ],
    });
    const result = await runSingle({
        id: "examples/gui_mandelbrot.nepl/video-memory-progressive-loop-test",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_mandelbrot.nepl"),
        argv: ["--video-memory-progressive-loop-test"],
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);

    return {
        ok: true,
        checks: [
            "Mandelbrot progressive loop advances one row batch per matching timer event",
            "timer id mismatch, empty event, focused event, and resized event do not advance the row batch",
            "progressive loop closes the surface immediately after the last batch is presented",
            "progressive loop keeps formal video memory presentation separate from stdout and command-frame paths",
        ],
    };
}

if (require.main === module) {
    runWebGuiMandelbrotVideoMemoryProgressiveLoopHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiMandelbrotVideoMemoryProgressiveLoopHarnessRegression,
};
