#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function runWebGuiVideoMemoryRowsExampleContractRegression() {
    const exampleSource = readRepoFile("examples", "gui_video_memory_rows.nepl");
    const webSurfaceSource = readRepoFile("stdlib", "platforms", "gui", "web", "surface.nepl");
    const specSource = readRepoFile("doc", "neplg2", "gui_standard_library_spec.md");
    const planSource = readRepoFile("doc", "neplg2", "gui_tui_implementation_plan.md");
    const sourcePolicyRunner = readRepoFile("nodesrc", "run_source_policy_regressions.js");
    const fakeHarnessSource = readRepoFile("nodesrc", "test_web_gui_video_memory_fake_host_harness.js");

    assert.match(exampleSource, /#import "alloc\/io" as \*/);
    assert.match(exampleSource, /#import "platforms\/gui\/web" as \*/);
    assert.match(exampleSource, /byte_builder_new/);
    assert.match(exampleSource, /byte_builder_push_u8/);
    assert.match(exampleSource, /byte_builder_finish/);
    assert.match(exampleSource, /byte_builder_error_free/);
    assert.match(exampleSource, /io_bytebuf_ptr_ref/);
    assert.match(exampleSource, /io_bytebuf_free/);
    assert.match(exampleSource, /gui_web_video_memory_create_surface/);
    assert.match(exampleSource, /gui_web_video_memory_acquire_write_frame/);
    assert.match(exampleSource, /gui_web_video_memory_write_rgba8888_row/);
    assert.match(exampleSource, /gui_web_video_memory_publish_full/);
    assert.match(exampleSource, /gui_web_video_memory_present_surface/);
    assert.match(exampleSource, /gui_web_video_memory_discard_write_frame/);
    assert.match(exampleSource, /gui_web_video_memory_close_surface/);
    assert.match(exampleSource, /fn vm_rows_discard_close_error[\s\S]*gui_web_video_memory_discard_write_frame/);
    assert.match(exampleSource, /match gui_web_video_memory_create_surface/);
    assert.match(exampleSource, /match gui_web_video_memory_acquire_write_frame/);
    assert.match(exampleSource, /match gui_web_video_memory_publish_full/);
    assert.match(exampleSource, /match window_id vm_rows_window_id/);
    assert.match(exampleSource, /argv: \["--contract"\]/);
    assert.match(exampleSource, /stdout: "video memory rows contract ok\\n"/);
    assert.doesNotMatch(exampleSource, /gui_web_stdout_/);
    assert.doesNotMatch(exampleSource, /NEPLG2_GUI_RGBA_ROW/);
    assert.doesNotMatch(exampleSource, /gui_web_video_memory_write_frame_bytes/);
    assert.doesNotMatch(exampleSource, /gui_web_video_memory_write_rgba8888_row_raw/);
    assert.doesNotMatch(exampleSource, /#extern/);

    assert.match(webSurfaceSource, /pub fn gui_web_video_memory_write_rgba8888_row[\s\S]*Result unit GuiError/);
    assert.match(webSurfaceSource, /stdout transport や command frame への fallback は/);
    assert.match(specSource, /gui_video_memory_rows\.nepl/);
    assert.match(specSource, /focused NEPL example/);
    assert.match(specSource, /fake positive `nepl_gui_web` host import harness/);
    assert.match(planSource, /gui_video_memory_rows\.nepl/);
    assert.match(planSource, /formal row host import/);
    assert.match(planSource, /positive fake host import harness[\s\S]*NEPL\/Wasm/);
    assert.match(sourcePolicyRunner, /nodesrc\/test_web_gui_video_memory_fake_host_harness\.js/);
    assert.match(fakeHarnessSource, /runSingle/);
    assert.match(fakeHarnessSource, /runtimeImportsFactory/);
    assert.match(fakeHarnessSource, /video_memory_write_rgba8888_row/);
    assert.match(fakeHarnessSource, /expectedRgbaRow/);
    assert.doesNotMatch(fakeHarnessSource, /argv:\s*\["--contract"\]/);

    return {
        ok: true,
        checks: [
            "focused NEPL example uses formal Web video memory row wrapper",
            "row bytes are built through ByteBuilder and passed through a borrowed ByteBuf pointer",
            "error paths discard write frames and close surfaces through Result handling",
            "example does not use stdout rgba-row, raw externs, or write-slot byte fallback",
        ],
    };
}

if (require.main === module) {
    try {
        const result = runWebGuiVideoMemoryRowsExampleContractRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runWebGuiVideoMemoryRowsExampleContractRegression,
};
