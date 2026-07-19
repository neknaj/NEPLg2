#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function runCase(id, testName, evidence, expectedCalls) {
    const calls = { begin: 0, run: 0, end: 0 };
    const result = await runSingle({
        id,
        source: `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_end_frame_command_test" as * with tests

fn main %impure fn void i32 \\void:
    ${testName} unit
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_end_frame_command.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory() {
            return {
                nepl_gui_web: {
                    compositor_tile_present_begin() {
                        calls.begin += 1;
                        return 0;
                    },
                    compositor_tile_present_run() {
                        calls.run += 1;
                        return 0;
                    },
                    compositor_tile_present_end() {
                        calls.end += 1;
                        return -1;
                    },
                },
            };
        },
    });
    assert.equal(result.ok, true, result.error);
    assert.equal(result.return_value, evidence);
    assert.equal(result.exit_code, evidence);
    assert.deepEqual(calls, expectedCalls);
    return evidence;
}

async function run() {
    const evidence = await runCase("gui-font-web-registered-end-frame-command/all", "gui_font_web_registered_end_frame_command_test_all_contract", 94, { begin: 2, run: 2, end: 0 });
    return { ok: true, evidence, calls: { begin: 2, run: 2, end: 0 } };
}

if (require.main === module) {
    run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => {
        console.error(error.stack || error);
        process.exit(1);
    });
}

module.exports = { run };
