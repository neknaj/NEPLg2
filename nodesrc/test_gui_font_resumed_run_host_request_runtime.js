#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

const sourceFor = (contract) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_host_request_test" as * with tests

fn main %impure fn void i32 \\void:
    ${contract} unit
`;

async function run() {
    const runCase = async (name, contract) => {
        const result = await runSingle({
            id: `gui-font-resumed-run-host-request/${name}`,
            source: sourceFor(contract),
            file: path.resolve(__dirname, "..", "tests", `gui_font_resumed_run_host_request_${name}.nepl`),
            distHint: path.resolve(__dirname, "..", "web", "dist"),
            forceStdlibVfs: true,
        });
        assert.equal(result.ok, true, result.error);
        assert.equal(result.return_value, 15);
        assert.equal(result.exit_code, 15);
        return result.return_value;
    };
    const success = await runCase("success", "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_host_request_test_contract");
    const failure = await runCase("failure", "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_host_request_test_failure_contract");
    return { ok: true, evidence: success + failure };
}

if (require.main === module) {
    run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => {
        console.error(error.stack || error);
        process.exit(1);
    });
}

module.exports = { run };
