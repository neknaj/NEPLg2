#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

const sourceFor = (contract) => String.raw`#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_begin_frame_retry_yield_scheduler_test" as * with tests

fn main %impure fn void i32 \void:
    ${contract} unit
`;

async function run() {
    const calls = { begin: 0, run: 0, end: 0 };
    const runCase = async (name, contract, evidence) => {
        const result = await runSingle({
            id: `gui-font-web-registered-retry-yield-scheduler/${name}`,
            source: sourceFor(contract),
            file: path.resolve(__dirname, "..", "tests", `gui_font_web_registered_retry_yield_scheduler_${name}.nepl`),
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
                            return -1;
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
        return result.return_value;
    };
    const resume = await runCase("resume", "gui_font_web_registered_begin_frame_retry_yield_scheduler_test_resume_contract", 511);
    const abort = await runCase("abort", "gui_font_web_registered_begin_frame_retry_yield_scheduler_test_abort_contract", 127);
    assert.deepEqual(calls, { begin: 2, run: 0, end: 0 });
    return { ok: true, evidence: resume + abort, calls };
}

if (require.main === module) {
    run()
        .then((value) => process.stdout.write(`${JSON.stringify(value)}\n`))
        .catch((error) => {
            console.error(error.stack || error);
            process.exit(1);
        });
}

module.exports = { run };
