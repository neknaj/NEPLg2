#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function run() {
    const calls = { begin: 0, run: 0, end: 0, timer: 0, poll: 0 };
    let current = null;
    const events = [
        { kind: 6, windowId: 4, timerId: 7, tick: 1 },
        { kind: 6, windowId: 3, timerId: 7, tick: 1 },
    ];
    async function runCase(name, contract, requestStatus, pollStatus, expected) {
        const result = await runSingle({
        id: "gui-font-web-registered-budgeted-run-polling-loop/runtime",
        source: `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_budgeted_run_polling_loop_test" as * with tests

fn main %impure fn void i32 \\void:
    ${contract} unit
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_budgeted_run_polling_loop.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory() {
            return { nepl_gui_web: {
                compositor_tile_present_begin() { calls.begin += 1; return 0; },
                compositor_tile_present_run() { calls.run += 1; return 0; },
                compositor_tile_present_end() { calls.end += 1; return 0; },
                request_timer(windowId, timerId, intervalMs, repeating) {
                    calls.timer += 1;
                    assert.deepEqual({ windowId, timerId, intervalMs, repeating }, { windowId: 3, timerId: 7, intervalMs: 1, repeating: 0 });
                    return requestStatus;
                },
                poll_event_kind() { calls.poll += 1; if (pollStatus < 0) return pollStatus; current = events.shift(); return current.kind; },
                last_event_window_id() { return current.windowId; },
                last_event_timer_id() { return current.timerId; },
                last_event_timer_tick() { return current.tick; },
            } };
        },
        });
        assert.equal(result.ok, true, `${name}: ${result.error && result.error.slice(-12000)}`);
        assert.equal(result.return_value, expected, name);
        return result.return_value;
    }
    const scheduleFailed = await runCase("schedule-failed", "gui_font_web_registered_budgeted_run_polling_loop_test_failure_contract", -1, 0, 13);
    const pollFailed = await runCase("poll-failed", "gui_font_web_registered_budgeted_run_polling_loop_test_failure_contract", 0, -1, 14);
    const evidence = await runCase("success", "gui_font_web_registered_budgeted_run_polling_loop_test_contract", 0, 0, 12);
    assert.deepEqual(calls, { begin: 3, run: 4, end: 0, timer: 3, poll: 3 });
    return { ok: true, scheduleFailed, pollFailed, evidence, calls };
}

if (require.main === module) run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => { console.error(error.stack || error); process.exit(1); });

module.exports = { run };
