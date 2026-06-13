#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createGuiVideoMemoryFakeHost } = require("./gui_video_memory_fake_host");
const { runSingle } = require("./run_test");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function expectedRgbaRow(y) {
    const bytes = [];
    for (let x = 0; x < 8; x += 1) {
        bytes.push(24 + x * 18);
        bytes.push(72 + y * 80);
        bytes.push(120 + x * 7);
        bytes.push(255);
    }
    return bytes;
}

async function runWebGuiVideoMemoryFakeHostHarnessRegression() {
    const source = readRepoFile("examples", "gui_video_memory_rows.nepl");
    assert.doesNotMatch(source, /argv: \["--contract"\][\s\S]*main[\s\S]*runtimeImportsFactory/);
    const fakeHost = createGuiVideoMemoryFakeHost({
        width: 8,
        height: 2,
        slotCount: 2,
        windowId: 1,
        title: "NEPLg2 Video Memory Rows",
        expectedRgbaRow,
    });
    const result = await runSingle({
        id: "examples/gui_video_memory_rows.nepl/fake-host-happy-path",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_video_memory_rows.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);
    return {
        ok: true,
        checks: [
            "normal NEPL/Wasm path executes without --contract",
            "shared fake nepl_gui_web host validates create/acquire/write-row/publish/present/close ordering",
            "row writer reads and checks RGBA8888 bytes from Wasm memory",
            "default run_test unsupported stubs remain opt-out from this focused harness",
        ],
    };
}

if (require.main === module) {
    runWebGuiVideoMemoryFakeHostHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiVideoMemoryFakeHostHarnessRegression,
};
