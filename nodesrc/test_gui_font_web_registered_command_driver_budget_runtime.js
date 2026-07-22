#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function run() {
    const result = await runSingle({
        id: "gui-font-web-registered-command-driver-budget/runtime",
        source: `#entry main
#indent 4
#target std
#import "core/math" as *
#import "platforms/gui/web/font_registered_command_driver_budget_test" as * with tests
#import "platforms/gui/web/font_registered_command_driver_budget_carrier_test" as * with tests

fn main %fn void i32 \\void:
    add gui_font_web_registered_command_driver_budget_test_contract unit gui_font_web_registered_command_driver_budget_carrier_test_contract unit
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_command_driver_budget.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
    });
    assert.equal(result.ok, true, result.error);
    assert.equal(result.return_value, 32);
    return { ok: true, evidence: 32 };
}

if (require.main === module) run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => { console.error(error.stack || error); process.exit(1); });

module.exports = { run };
