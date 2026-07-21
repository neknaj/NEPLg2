#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function run() {
    const calls = { begin: 0, run: 0, end: 0 };
    const result = await runSingle({
        id: "gui-font-web-registered-resumed-web-loop/success",
        source: `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_resumed_web_loop_test" as * with tests

fn main %impure fn void i32 \\void:
    gui_font_web_registered_resumed_web_loop_test_runtime_contract unit
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_resumed_web_loop.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory() {
            return {
                nepl_gui_web: {
                    compositor_tile_present_begin() { calls.begin += 1; return 0; },
                    compositor_tile_present_run() { calls.run += 1; return calls.run === 1 ? 0 : -1; },
                    compositor_tile_present_end() { calls.end += 1; return 0; },
                },
            };
        },
    });
    assert.equal(result.ok, true, result.error);
    assert.equal(result.return_value, 510);
    assert.deepEqual(calls, { begin: 2, run: 2, end: 1 });
    return { ok: true, evidence: 510, calls };
}

if (require.main === module) {
    run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => {
        console.error(error.stack || error);
        process.exit(1);
    });
}

module.exports = { run };
