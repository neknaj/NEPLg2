#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createGuiVideoMemoryFakeHost } = require("./gui_video_memory_fake_host");
const { runSingle } = require("./run_test");

const FIRST_WIDTH = 32;
const FIRST_HEIGHT = 18;
const FIRST_LIMIT = 24;
const RESIZED_WINDOW_WIDTH = 40;
const RESIZED_WINDOW_HEIGHT = 90;
const SECOND_WIDTH = 40;
const SECOND_HEIGHT = 18;
const SECOND_LIMIT = 64;

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

function u8(value) {
    return ((value % 256) + 256) % 256;
}

function mandelbrotCx(width, x) {
    return divS(x * 300, width - 1) - 220;
}

function mandelbrotCy(height, y) {
    return divS(y * 220, height - 1) - 110;
}

function mandelbrotEscapeIteration(width, height, limit, x, y) {
    const cx = mandelbrotCx(width, x);
    const cy = mandelbrotCy(height, y);
    let zx = 0;
    let zy = 0;
    let iter = 0;
    while (true) {
        const zx2 = zx * zx;
        const zy2 = zy * zy;
        if (zx2 + zy2 >= 40000 || iter >= limit) {
            return iter;
        }
        const nextZx = divS(zx2, 100) - divS(zy2, 100) + cx;
        const nextZy = divS(zx * zy * 2, 100) + cy;
        zx = nextZx;
        zy = nextZy;
        iter += 1;
    }
}

function expectedMandelbrotRgbaRow(width, height, limit) {
    return (y) => {
        const bytes = [];
        for (let x = 0; x < width; x += 1) {
            const iter = mandelbrotEscapeIteration(width, height, limit, x, y);
            if (iter === limit) {
                bytes.push(3, 7, 12, 255);
            } else {
                bytes.push(u8(24 + iter * 3), u8(52 + iter * 5), u8(98 + iter * 4), 255);
            }
        }
        return bytes;
    };
}

async function runWebGuiMandelbrotVideoMemoryResizeHarnessRegression() {
    const source = readRepoFile("examples", "gui_mandelbrot.nepl");
    assert.match(source, /--video-memory-resize-once/);
    assert.match(source, /fn mandelbrot_video_memory_open_rendered_surface/);
    assert.match(source, /fn mandelbrot_video_memory_close_and_open_next/);
    assert.match(source, /fn mandelbrot_video_memory_resize_once[\s\S]*gui_web_wait_event_result/);
    assert.match(source, /fn mandelbrot_video_memory_resize_once_event[\s\S]*WindowEventKind::Resized[\s\S]*mandelbrot_update_window[\s\S]*WindowEventKind::Focused:[\s\S]*WindowEventKind::Unfocused:[\s\S]*WindowEventKind::CloseRequested:/);

    const resizeSlice = sourceBetween(source, "fn mandelbrot_video_memory_resize_once_event", "fn mandelbrot_present_row_pixel");
    assert.doesNotMatch(resizeSlice, /mandelbrot_present_frame|gui_web_stdout_|gui_web_stdout_frame|presentCommands|command-frame/);
    assert.doesNotMatch(resizeSlice, /gui_web_video_memory_write_frame_bytes/);

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
                surfaceId: 1201,
                frameId: 4501,
                expectedRgbaRow: expectedMandelbrotRgbaRow(FIRST_WIDTH, FIRST_HEIGHT, FIRST_LIMIT),
            },
            {
                width: SECOND_WIDTH,
                height: SECOND_HEIGHT,
                slotCount: 2,
                windowId: 1,
                title: "NEPLg2 Mandelbrot Video Memory",
                surfaceId: 1202,
                frameId: 4502,
                expectedRgbaRow: expectedMandelbrotRgbaRow(SECOND_WIDTH, SECOND_HEIGHT, SECOND_LIMIT),
            },
        ],
        events: [
            {
                kind: "window",
                windowKind: "resized",
                windowId: 1,
                width: RESIZED_WINDOW_WIDTH,
                height: RESIZED_WINDOW_HEIGHT,
            },
        ],
    });

    const result = await runSingle({
        id: "examples/gui_mandelbrot.nepl/video-memory-resize-once",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_mandelbrot.nepl"),
        argv: ["--video-memory-resize-once"],
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);

    return {
        ok: true,
        checks: [
            "Mandelbrot video memory resize path waits for a typed GuiWebEvent",
            "Mandelbrot video memory resize path closes the old surface before creating the resized surface",
            "fake host validates both initial and resized RGBA8888 row payloads from Wasm memory",
            "resize harness does not use stdout, command-frame, write_slot_bytes, or TS simulation",
        ],
    };
}

if (require.main === module) {
    runWebGuiMandelbrotVideoMemoryResizeHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiMandelbrotVideoMemoryResizeHarnessRegression,
};
