#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createGuiVideoMemoryFakeHost } = require("./gui_video_memory_fake_host");
const { runSingle } = require("./run_test");

const MANDELBROT_WIDTH = 32;
const MANDELBROT_HEIGHT = 18;
const MANDELBROT_ITERATION_LIMIT = 24;

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

function divS(numerator, denominator) {
    return Math.trunc(numerator / denominator);
}

function mandelbrotCx(x) {
    return divS(x * 300, MANDELBROT_WIDTH - 1) - 220;
}

function mandelbrotCy(y) {
    return divS(y * 220, MANDELBROT_HEIGHT - 1) - 110;
}

function mandelbrotEscapeIteration(x, y) {
    const cx = mandelbrotCx(x);
    const cy = mandelbrotCy(y);
    let zx = 0;
    let zy = 0;
    let iter = 0;
    while (true) {
        const zx2 = zx * zx;
        const zy2 = zy * zy;
        if (zx2 + zy2 >= 40000 || iter >= MANDELBROT_ITERATION_LIMIT) {
            return iter;
        }
        const nextZx = divS(zx2, 100) - divS(zy2, 100) + cx;
        const nextZy = divS(zx * zy * 2, 100) + cy;
        zx = nextZx;
        zy = nextZy;
        iter += 1;
    }
}

function expectedMandelbrotRgbaRow(y) {
    const bytes = [];
    for (let x = 0; x < MANDELBROT_WIDTH; x += 1) {
        const iter = mandelbrotEscapeIteration(x, y);
        if (iter === MANDELBROT_ITERATION_LIMIT) {
            bytes.push(3, 7, 12, 255);
        } else {
            bytes.push(24 + iter * 3, 52 + iter * 5, 98 + iter * 4, 255);
        }
    }
    return bytes;
}

async function runWebGuiMandelbrotVideoMemoryHarnessRegression() {
    const source = readRepoFile("examples", "gui_mandelbrot.nepl");
    assert.match(source, /--video-memory-once/);
    assert.match(source, /--test-video-memory-contract/);
    assert.match(source, /fn mandelbrot_video_memory_model[\s\S]*mandelbrot_model_new 32 18 1 24 MandelbrotMode::Preview/);
    assert.match(source, /gui_web_video_memory_create_surface/);
    assert.match(source, /gui_web_video_memory_write_rgba8888_row/);
    assert.match(source, /gui_web_video_memory_publish_full/);
    assert.match(source, /gui_web_video_memory_present_surface/);
    assert.match(source, /gui_web_video_memory_close_surface/);

    const videoMemorySlice = sourceBetween(source, "fn mandelbrot_video_memory_slot_count", "fn mandelbrot_present_row_pixel");
    assert.doesNotMatch(videoMemorySlice, /gui_web_stdout_/);
    assert.doesNotMatch(videoMemorySlice, /mandelbrot_present_frame/);

    const runSlice = sourceBetween(source, "fn mandelbrot_run_video_memory_once", "fn mandelbrot_event_loop");
    assert.doesNotMatch(runSlice, /mandelbrot_run_once|mandelbrot_present_frame|gui_web_stdout_/);

    const fakeHost = createGuiVideoMemoryFakeHost({
        width: MANDELBROT_WIDTH,
        height: MANDELBROT_HEIGHT,
        slotCount: 2,
        windowId: 1,
        title: "NEPLg2 Mandelbrot Video Memory",
        expectedRgbaRow: expectedMandelbrotRgbaRow,
    });
    const result = await runSingle({
        id: "examples/gui_mandelbrot.nepl/video-memory-once",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_mandelbrot.nepl"),
        argv: ["--video-memory-once"],
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);
    return {
        ok: true,
        checks: [
            "Mandelbrot --video-memory-once executes the formal host import path",
            "Mandelbrot video memory path does not fallback to stdout transport",
            "fake host validates every RGBA8888 row from Wasm memory",
            "fake host validates publish/present/close ordering and post-close rejection",
        ],
    };
}

if (require.main === module) {
    runWebGuiMandelbrotVideoMemoryHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiMandelbrotVideoMemoryHarnessRegression,
};
