#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

const SOURCE = String.raw`#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_begin_frame_retry_success_phase_test" as * with tests

fn main %impure fn void i32 \void:
    gui_font_web_registered_begin_frame_retry_success_phase_test_contract unit
`;

function successHost() {
    const calls = { begin: 0, run: 0, end: 0 };
    return {
        importsFactory() {
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
        verify(result) {
            assert.equal(result.ok, true, result.error);
            assert.equal(result.return_value, 127);
            assert.equal(result.exit_code, 127);
            assert.deepEqual(calls, { begin: 1, run: 0, end: 0 });
        },
    };
}

async function run() {
    const host = successHost();
    const result = await runSingle({
        id: "gui-font-web-registered-retry-success-phase/runtime",
        source: SOURCE,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_retry_success_phase_runtime.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: host.importsFactory,
    });
    host.verify(result);
    return { ok: true, evidence: result.return_value, calls: { begin: 1, run: 0, end: 0 } };
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
