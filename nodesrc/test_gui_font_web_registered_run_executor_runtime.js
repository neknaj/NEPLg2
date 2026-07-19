#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

const source = (testName) => `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_run_executor_test" as * with tests

fn main %impure fn void i32 \\void:
    ${testName} unit
`;

async function runCase(id, testName, runStatus, evidence) {
    const calls = { begin: 0, run: 0, end: 0 };
    const result = await runSingle({
        id,
        source: source(testName),
        file: path.resolve(__dirname, "..", "tests", `${id.replaceAll("/", "_")}.nepl`),
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
                        return runStatus;
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
    assert.deepEqual(calls, { begin: 1, run: 1, end: 0 });
    return result.return_value;
}

async function run() {
    const success = await runCase("gui-font-web-registered-run-executor/success", "gui_font_web_registered_run_executor_test_success_contract", 0, 1023);
    const failure = await runCase("gui-font-web-registered-run-executor/failure", "gui_font_web_registered_run_executor_test_failure_contract", -1, 15);
    return { ok: true, evidence: success + failure, calls: { begin: 2, run: 2, end: 0 } };
}

if (require.main === module) {
    run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => {
        console.error(error.stack || error);
        process.exit(1);
    });
}

module.exports = { run };
