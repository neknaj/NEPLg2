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
            frameId: 6501 + frames.length,
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

async function runWebGuiMandelbrotVideoMemoryProgressiveHarnessRegression() {
    const source = readRepoFile("examples", "gui_mandelbrot.nepl");
    assert.match(source, /--video-memory-progressive-once/);
    assert.match(source, /--video-memory-progressive-test/);
    assert.match(source, /fn mandelbrot_video_memory_progressive_batch_height[\s\S]*4/);
    assert.match(source, /fn mandelbrot_video_memory_publish_rect_present[\s\S]*gui_web_video_memory_publish_rect/);
    assert.match(source, /fn mandelbrot_video_memory_render_batch_present[\s\S]*mandelbrot_video_memory_publish_rect_present/);
    assert.match(source, /fn mandelbrot_run_video_memory_progressive_test[\s\S]*mandelbrot_run_video_memory_progressive_once/);

    const progressiveSlice = sourceBetween(
        source,
        "fn mandelbrot_video_memory_publish_rect_present",
        "fn mandelbrot_video_memory_open_rendered_surface",
    );
    assert.match(progressiveSlice, /gui_web_video_memory_publish_rect/);
    assert.doesNotMatch(progressiveSlice, /gui_web_stdout_|mandelbrot_present_frame|presentCommands|command-frame/);
    assert.doesNotMatch(progressiveSlice, /gui_web_video_memory_write_frame_bytes|gui_web_video_memory_fill_rect_rgba8888/);

    const fakeHost = createGuiVideoMemoryFakeHost({
        windowId: 1,
        title: "NEPLg2 Mandelbrot Video Memory",
        surfaces: [
            {
                width: MANDELBROT_WIDTH,
                height: MANDELBROT_HEIGHT,
                slotCount: 2,
                windowId: 1,
                title: "NEPLg2 Mandelbrot Video Memory",
                surfaceId: 1401,
                expectedRgbaRow: createExpectedMandelbrotRgbaRow(
                    MANDELBROT_WIDTH,
                    MANDELBROT_HEIGHT,
                    MANDELBROT_ITERATION_LIMIT,
                ),
                frames: createExpectedProgressiveFrames(),
            },
        ],
    });
    const result = await runSingle({
        id: "examples/gui_mandelbrot.nepl/video-memory-progressive-test",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_mandelbrot.nepl"),
        argv: ["--video-memory-progressive-test"],
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);

    return {
        ok: true,
        checks: [
            "Mandelbrot progressive video memory test uses the same implementation as the opt-in once path",
            "progressive path publishes rect dirty regions for finite row batches on one surface",
            "fake host validates that each frame writes only its expected row range",
            "last dirty rect is clamped to the remaining two rows instead of overpublishing a full batch",
        ],
    };
}

if (require.main === module) {
    runWebGuiMandelbrotVideoMemoryProgressiveHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiMandelbrotVideoMemoryProgressiveHarnessRegression,
};
