#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function runCase(name, contract, evidence, expectedCalls) {
    const calls = { begin: 0, run: 0, end: 0 };
    const result = await runSingle({
        id: `gui-font-web-registered-budgeted-run-driver/${name}`,
        source: `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_budgeted_run_driver_test" as * with tests

fn main %impure fn void i32 \\void:
    ${contract} unit
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_budgeted_run_driver_exhaustion.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory() {
            return {
                nepl_gui_web: {
                    compositor_tile_present_begin() { calls.begin += 1; return 0; },
                    compositor_tile_present_run() { calls.run += 1; return 0; },
                    compositor_tile_present_end() { calls.end += 1; return 0; },
                },
            };
        },
    });
    assert.equal(result.ok, true, result.error && result.error.slice(-12000));
    assert.equal(result.return_value, evidence);
    assert.deepEqual(calls, expectedCalls);
    return { evidence, calls };
}

async function run() {
    const totalExhausted = await runCase("total-exhausted", "gui_font_web_registered_budgeted_run_driver_test_total_exhausted_contract", 5, { begin: 1, run: 1, end: 0 });
    const suspendedResume = await runCase("suspended-resume", "gui_font_web_registered_budgeted_run_driver_test_suspended_resume_contract", 11, { begin: 1, run: 2, end: 0 });
    return { ok: true, totalExhausted, suspendedResume };
}

if (require.main === module) run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => { console.error(error.stack || error); process.exit(1); });

module.exports = { run };
