#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function run() {
    const calls = { begin: 0, run: 0, end: 0 };
    const result = await runSingle({
        id: "gui-font-web-registered-budgeted-run-driver/runtime",
        source: `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_budgeted_run_driver_test" as * with tests

fn main %impure fn void i32 \\void:
    gui_font_web_registered_budgeted_run_driver_test_contract unit
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_budgeted_run_driver.nepl"),
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
    assert.equal(result.return_value, 48);
    assert.deepEqual(calls, { begin: 1, run: 1, end: 0 });
    return { ok: true, evidence: 48, calls };
}

if (require.main === module) run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => { console.error(error.stack || error); process.exit(1); });

module.exports = { run };
